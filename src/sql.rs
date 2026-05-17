use crate::index::PrimaryIndex;
use crate::storage::{PageStore, StorageError};
use std::fs;
use std::path::Path;

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const CATALOG_RECORD: u8 = b'C';
const ROW_RECORD: u8 = b'R';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlError {
    Unsupported(String),
    Malformed(String),
    Semantic { message: String, hint: &'static str },
    InvalidStorageRecord,
    Storage(StorageError),
}

impl From<StorageError> for SqlError {
    fn from(error: StorageError) -> Self {
        SqlError::Storage(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnType {
    Int,
    Text,
}

impl ColumnType {
    fn as_str(self) -> &'static str {
        match self {
            ColumnType::Int => "INT",
            ColumnType::Text => "TEXT",
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'I' => Some(ColumnType::Int),
            b'T' => Some(ColumnType::Text),
            _ => None,
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            ColumnType::Int => b'I',
            ColumnType::Text => b'T',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Column {
    name: String,
    column_type: ColumnType,
    is_primary_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Vec<Value>>,
    primary_key_column: Option<usize>,
    primary_index: PrimaryIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Int(i64),
    Text(String),
}

impl Value {
    fn column_type(&self) -> ColumnType {
        match self {
            Value::Int(_) => ColumnType::Int,
            Value::Text(_) => ColumnType::Text,
        }
    }

    fn output(&self) -> String {
        match self {
            Value::Int(value) => value.to_string(),
            Value::Text(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    CreateTable {
        table: String,
        columns: Vec<Column>,
    },
    Insert {
        table: String,
        values: Vec<Value>,
    },
    SelectAll {
        table: String,
    },
    SelectPrimaryKey {
        table: String,
        column: String,
        key: i64,
        raw: String,
    },
}

#[derive(Debug, Default)]
struct Database {
    tables: Vec<Table>,
}

impl Database {
    fn from_records(records: Vec<Vec<u8>>) -> Result<Self, SqlError> {
        let mut database = Database::default();
        for record in records {
            match decode_record(&record)? {
                LogicalRecord::Catalog { table, columns } => {
                    let primary_key_column = validate_catalog_record_invariants(&table, &columns)?;
                    if database.find_table(&table).is_some() {
                        return Err(SqlError::InvalidStorageRecord);
                    }
                    database.tables.push(Table {
                        name: table,
                        columns,
                        rows: Vec::new(),
                        primary_key_column,
                        primary_index: PrimaryIndex::new(),
                    });
                }
                LogicalRecord::Row { table, values } => {
                    let Some(existing) = database.find_table_mut(&table) else {
                        return Err(SqlError::InvalidStorageRecord);
                    };
                    if existing.columns.len() != values.len() {
                        return Err(SqlError::InvalidStorageRecord);
                    }
                    for (column, value) in existing.columns.iter().zip(values.iter()) {
                        if column.column_type != value.column_type() {
                            return Err(SqlError::InvalidStorageRecord);
                        }
                        validate_loaded_value(value)?;
                    }
                    if let Some(primary_key_column) = existing.primary_key_column {
                        let Value::Int(key) = values[primary_key_column] else {
                            return Err(SqlError::InvalidStorageRecord);
                        };
                        existing
                            .primary_index
                            .insert(key, existing.rows.len())
                            .map_err(|_| SqlError::InvalidStorageRecord)?;
                    }
                    existing.rows.push(values);
                }
            }
        }
        Ok(database)
    }

    fn find_table(&self, name: &str) -> Option<&Table> {
        self.tables
            .iter()
            .find(|table| table.name.eq_ignore_ascii_case(name))
    }

    fn find_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables
            .iter_mut()
            .find(|table| table.name.eq_ignore_ascii_case(name))
    }
}

fn validate_catalog_record_invariants(
    table: &str,
    columns: &[Column],
) -> Result<Option<usize>, SqlError> {
    if !is_valid_identifier(table) || columns.is_empty() {
        return Err(SqlError::InvalidStorageRecord);
    }

    let mut primary_key_column = None;
    for (index, column) in columns.iter().enumerate() {
        if !is_valid_identifier(&column.name)
            || columns[..index]
                .iter()
                .any(|existing| existing.name.eq_ignore_ascii_case(&column.name))
        {
            return Err(SqlError::InvalidStorageRecord);
        }
        if column.is_primary_key {
            if column.column_type != ColumnType::Int || primary_key_column.is_some() {
                return Err(SqlError::InvalidStorageRecord);
            }
            primary_key_column = Some(index);
        }
    }

    Ok(primary_key_column)
}

fn validate_loaded_value(value: &Value) -> Result<(), SqlError> {
    match value {
        Value::Int(_) => Ok(()),
        Value::Text(value) if is_output_safe_text(value) => Ok(()),
        Value::Text(_) => Err(SqlError::InvalidStorageRecord),
    }
}

enum LogicalRecord {
    Catalog { table: String, columns: Vec<Column> },
    Row { table: String, values: Vec<Value> },
}

pub fn execute(path: impl AsRef<Path>, sql: &str) -> Result<String, SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    let mut store = PageStore::open(path)?;
    let mut database = Database::from_records(store.read_records()?)?;
    let statements = parse_statements(sql)?;
    let mut stdout = String::new();

    for statement in statements {
        match statement {
            Statement::CreateTable { table, columns } => {
                execute_create_table(&mut store, &mut database, table, columns)?;
            }
            Statement::Insert { table, values } => {
                execute_insert(&mut store, &mut database, table, values)?;
            }
            Statement::SelectAll { table } => {
                execute_select(&database, &table, &mut stdout)?;
            }
            Statement::SelectPrimaryKey {
                table,
                column,
                key,
                raw,
            } => {
                execute_select_primary_key(&database, &table, &column, key, &raw, &mut stdout)?;
            }
        }
    }

    Ok(stdout)
}

pub fn validate_records_for_check(records: Vec<Vec<u8>>) -> Result<(), &'static str> {
    let mut database = Database::default();
    for record in records {
        match decode_record(&record).map_err(|_| "storage record readability")? {
            LogicalRecord::Catalog { table, columns } => {
                let primary_key_column = validate_catalog_record_invariants(&table, &columns)
                    .map_err(|_| "catalog/record invariant")?;
                if database.find_table(&table).is_some() {
                    return Err("catalog/record invariant");
                }
                database.tables.push(Table {
                    name: table,
                    columns,
                    rows: Vec::new(),
                    primary_key_column,
                    primary_index: PrimaryIndex::new(),
                });
            }
            LogicalRecord::Row { table, values } => {
                let Some(existing) = database.find_table_mut(&table) else {
                    return Err("catalog/record invariant");
                };
                if existing.columns.len() != values.len() {
                    return Err("catalog/record invariant");
                }
                for (column, value) in existing.columns.iter().zip(values.iter()) {
                    if column.column_type != value.column_type() {
                        return Err("catalog/record invariant");
                    }
                    validate_loaded_value(value).map_err(|_| "catalog/record invariant")?;
                }
                if let Some(primary_key_column) = existing.primary_key_column {
                    let Value::Int(key) = values[primary_key_column] else {
                        return Err("catalog/record invariant");
                    };
                    existing
                        .primary_index
                        .insert(key, existing.rows.len())
                        .map_err(|_| "primary index")?;
                }
                existing.rows.push(values);
            }
        }
    }

    Ok(())
}

fn initialize_empty_sql_file(path: &Path) -> Result<(), SqlError> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() == 0 => {
            fs::remove_file(path).map_err(|_| SqlError::Storage(StorageError::Io))?;
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(SqlError::Storage(StorageError::Io)),
    }
}

fn execute_create_table(
    store: &mut PageStore,
    database: &mut Database,
    table: String,
    columns: Vec<Column>,
) -> Result<(), SqlError> {
    if database.find_table(&table).is_some() {
        return Err(SqlError::Semantic {
            message: format!("table already exists: {table}"),
            hint: "use a new table name for CREATE TABLE in this database.",
        });
    }

    for (index, column) in columns.iter().enumerate() {
        if columns[..index]
            .iter()
            .any(|existing| existing.name.eq_ignore_ascii_case(&column.name))
        {
            return Err(SqlError::Semantic {
                message: format!("duplicate column: {}", column.name),
                hint: "column names in a table must be unique.",
            });
        }
    }

    let mut primary_key_count = 0usize;
    for column in &columns {
        if column.is_primary_key {
            primary_key_count += 1;
            if column.column_type != ColumnType::Int {
                return Err(SqlError::Semantic {
                    message: format!("primary key column must be INT: {}", column.name),
                    hint: "this SQL slice supports one INT PRIMARY KEY column per table.",
                });
            }
        }
    }
    if primary_key_count > 1 {
        return Err(SqlError::Semantic {
            message: format!("multiple primary key columns for table {table}"),
            hint: "this SQL slice supports one INT PRIMARY KEY column per table.",
        });
    }

    store.append_record(&encode_catalog_record(&table, &columns))?;
    let primary_key_column = columns.iter().position(|column| column.is_primary_key);
    database.tables.push(Table {
        name: table,
        columns,
        rows: Vec::new(),
        primary_key_column,
        primary_index: PrimaryIndex::new(),
    });
    Ok(())
}

fn execute_insert(
    store: &mut PageStore,
    database: &mut Database,
    table: String,
    values: Vec<Value>,
) -> Result<(), SqlError> {
    let Some(existing) = database.find_table_mut(&table) else {
        return Err(table_not_found(&table));
    };

    if existing.columns.len() != values.len() {
        return Err(SqlError::Semantic {
            message: format!(
                "column count mismatch for table {}: expected {} values, got {}",
                existing.name,
                existing.columns.len(),
                values.len()
            ),
            hint: "INSERT values must match the table schema exactly.",
        });
    }

    for (column, value) in existing.columns.iter().zip(values.iter()) {
        if column.column_type != value.column_type() {
            return Err(SqlError::Semantic {
                message: format!(
                    "type mismatch for column {}: expected {}, got {}",
                    column.name,
                    column.column_type.as_str(),
                    value.column_type().as_str()
                ),
                hint: "INSERT values must match the declared column types.",
            });
        }
    }

    if let Some(primary_key_column) = existing.primary_key_column {
        let Value::Int(key) = values[primary_key_column] else {
            return Err(SqlError::InvalidStorageRecord);
        };
        if existing.primary_index.get(key).is_some() {
            return Err(SqlError::Semantic {
                message: format!("duplicate primary key for table {}: {key}", existing.name),
                hint: "primary key values must be unique.",
            });
        }
    }

    store.append_record(&encode_row_record(&existing.name, &values))?;
    if let Some(primary_key_column) = existing.primary_key_column {
        let Value::Int(key) = values[primary_key_column] else {
            return Err(SqlError::InvalidStorageRecord);
        };
        existing
            .primary_index
            .insert(key, existing.rows.len())
            .map_err(|_| SqlError::InvalidStorageRecord)?;
    }
    existing.rows.push(values);
    Ok(())
}

fn execute_select(database: &Database, table: &str, stdout: &mut String) -> Result<(), SqlError> {
    let Some(existing) = database.find_table(table) else {
        return Err(table_not_found(table));
    };

    write_header(existing, stdout);

    if existing.primary_key_column.is_some() {
        for row_position in existing.primary_index.ordered_positions() {
            write_row(&existing.rows[row_position], stdout);
        }
    } else {
        for row in &existing.rows {
            write_row(row, stdout);
        }
    }

    Ok(())
}

fn execute_select_primary_key(
    database: &Database,
    table: &str,
    column: &str,
    key: i64,
    raw: &str,
    stdout: &mut String,
) -> Result<(), SqlError> {
    let Some(existing) = database.find_table(table) else {
        return Err(table_not_found(table));
    };
    let Some(primary_key_column) = existing.primary_key_column else {
        return Err(SqlError::Unsupported(raw.to_string()));
    };
    if !existing.columns[primary_key_column]
        .name
        .eq_ignore_ascii_case(column)
    {
        return Err(SqlError::Unsupported(raw.to_string()));
    }

    write_header(existing, stdout);
    if let Some(row_position) = existing.primary_index.get(key) {
        write_row(&existing.rows[row_position], stdout);
    }
    Ok(())
}

fn write_header(table: &Table, stdout: &mut String) {
    for (index, column) in table.columns.iter().enumerate() {
        if index > 0 {
            stdout.push('|');
        }
        stdout.push_str(&column.name);
    }
    stdout.push('\n');
}

fn write_row(row: &[Value], stdout: &mut String) {
    for (index, value) in row.iter().enumerate() {
        if index > 0 {
            stdout.push('|');
        }
        stdout.push_str(&value.output());
    }
    stdout.push('\n');
}

fn table_not_found(table: &str) -> SqlError {
    SqlError::Semantic {
        message: format!("table not found: {table}"),
        hint: "create the table before INSERT or SELECT.",
    }
}

fn parse_statements(sql: &str) -> Result<Vec<Statement>, SqlError> {
    let raw_statements = split_raw_statements(sql)?;
    raw_statements
        .into_iter()
        .map(|raw| parse_statement(&raw))
        .collect()
}

fn split_raw_statements(sql: &str) -> Result<Vec<String>, SqlError> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut in_text = false;

    for (index, ch) in sql.char_indices() {
        if ch == '\'' {
            in_text = !in_text;
        }
        if ch == ';' && !in_text {
            let raw = sql[start..=index].trim();
            if raw.is_empty() || raw == ";" {
                return Err(SqlError::Malformed(raw.to_string()));
            }
            statements.push(raw.to_string());
            start = index + ch.len_utf8();
        }
    }

    let trailing = sql[start..].trim();
    if in_text || !trailing.is_empty() || statements.is_empty() {
        return Err(SqlError::Malformed(sql.trim().to_string()));
    }

    Ok(statements)
}

fn parse_statement(raw: &str) -> Result<Statement, SqlError> {
    let body = raw
        .strip_suffix(';')
        .expect("split_raw_statements keeps statement delimiter")
        .trim();

    if starts_with_keyword_sequence(body, &["CREATE", "TABLE"]) {
        parse_create_table(body).map_err(|()| SqlError::Malformed(raw.to_string()))
    } else if starts_with_keyword_sequence(body, &["INSERT", "INTO"]) {
        parse_insert(body).map_err(|()| {
            if is_insert_with_column_list(body) {
                SqlError::Unsupported(raw.to_string())
            } else {
                SqlError::Malformed(raw.to_string())
            }
        })
    } else if starts_with_keyword_sequence(body, &["SELECT"]) {
        parse_select(body, raw).map_err(|()| {
            if is_malformed_select_shape(body) {
                SqlError::Malformed(raw.to_string())
            } else {
                SqlError::Unsupported(raw.to_string())
            }
        })
    } else {
        Err(SqlError::Unsupported(raw.to_string()))
    }
}

fn is_malformed_select_shape(body: &str) -> bool {
    let tokens: Vec<&str> = body.split_whitespace().collect();
    if tokens.len() < 2 {
        return true;
    }
    if tokens[1] != "*" {
        return false;
    }
    if tokens.len() < 4 {
        return true;
    }
    if !tokens[2].eq_ignore_ascii_case("FROM") {
        return true;
    }
    if !is_valid_identifier(tokens[3]) {
        return true;
    }
    tokens.len() > 4 && !starts_unsupported_select_clause(tokens[4])
}

fn starts_unsupported_select_clause(token: &str) -> bool {
    token.eq_ignore_ascii_case("WHERE")
        || token.eq_ignore_ascii_case("ORDER")
        || token.eq_ignore_ascii_case("JOIN")
}

fn is_insert_with_column_list(body: &str) -> bool {
    let mut parser = Parser::new(body);
    if parser.consume_keyword("INSERT").is_err()
        || parser.require_space().is_err()
        || parser.consume_keyword("INTO").is_err()
        || parser.require_space().is_err()
        || parser.consume_identifier().is_err()
    {
        return false;
    }
    parser.skip_space();
    parser.peek_char() == Some('(')
}

fn parse_create_table(body: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("CREATE")?;
    parser.require_space()?;
    parser.consume_keyword("TABLE")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char('(')?;
    let columns_body = parser.consume_until_matching_close_paren()?;
    parser.skip_space();
    parser.finish()?;

    let mut columns = Vec::new();
    for item in split_comma_list(columns_body)? {
        let mut column_parser = Parser::new(item);
        let name = column_parser.consume_identifier()?;
        column_parser.require_space()?;
        let column_type = if column_parser.consume_keyword("INT").is_ok() {
            ColumnType::Int
        } else if column_parser.consume_keyword("TEXT").is_ok() {
            ColumnType::Text
        } else {
            return Err(());
        };
        column_parser.skip_space();
        let is_primary_key = if column_parser.consume_keyword("PRIMARY").is_ok() {
            column_parser.require_space()?;
            column_parser.consume_keyword("KEY")?;
            column_parser.skip_space();
            true
        } else {
            false
        };
        column_parser.finish()?;
        columns.push(Column {
            name,
            column_type,
            is_primary_key,
        });
    }

    if columns.is_empty() {
        return Err(());
    }

    Ok(Statement::CreateTable { table, columns })
}

fn parse_insert(body: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("INSERT")?;
    parser.require_space()?;
    parser.consume_keyword("INTO")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.require_space()?;
    parser.consume_keyword("VALUES")?;
    parser.skip_space();
    parser.consume_char('(')?;
    let values_body = parser.consume_until_matching_close_paren()?;
    parser.skip_space();
    parser.finish()?;

    let mut values = Vec::new();
    for item in split_value_list(values_body)? {
        values.push(parse_value(item)?);
    }

    if values.is_empty() {
        return Err(());
    }

    Ok(Statement::Insert { table, values })
}

fn parse_select(body: &str, raw: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("SELECT")?;
    parser.require_space()?;
    parser.consume_char('*')?;
    parser.require_space()?;
    parser.consume_keyword("FROM")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.skip_space();
    if parser.consume_keyword("WHERE").is_ok() {
        parser.require_space()?;
        let column = parser.consume_identifier()?;
        parser.skip_space();
        parser.consume_char('=')?;
        parser.skip_space();
        let key = parser.consume_int_literal()?;
        parser.skip_space();
        parser.finish()?;
        return Ok(Statement::SelectPrimaryKey {
            table,
            column,
            key,
            raw: raw.to_string(),
        });
    }
    parser.finish()?;
    Ok(Statement::SelectAll { table })
}

fn parse_value(raw: &str) -> Result<Value, ()> {
    let value = raw.trim();
    if value.starts_with('\'') {
        let Some(inner) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')) else {
            return Err(());
        };
        if !is_output_safe_text(inner) {
            return Err(());
        }
        return Ok(Value::Text(inner.to_string()));
    }

    if !is_signed_decimal_literal(value) {
        return Err(());
    }
    let parsed = value.parse::<i64>().map_err(|_| ())?;
    Ok(Value::Int(parsed))
}

fn is_signed_decimal_literal(value: &str) -> bool {
    let digits = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit())
}

fn is_output_safe_text(value: &str) -> bool {
    !value.contains('|') && !value.contains('\n') && !value.contains('\r') && !value.contains('\'')
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn split_comma_list(input: &str) -> Result<Vec<&str>, ()> {
    let items: Vec<&str> = input.split(',').map(str::trim).collect();
    if items.iter().any(|item| item.is_empty()) {
        return Err(());
    }
    Ok(items)
}

fn split_value_list(input: &str) -> Result<Vec<&str>, ()> {
    let mut items = Vec::new();
    let mut start = 0usize;
    let mut in_text = false;

    for (index, ch) in input.char_indices() {
        if ch == '\'' {
            in_text = !in_text;
        }
        if ch == ',' && !in_text {
            let item = input[start..index].trim();
            if item.is_empty() {
                return Err(());
            }
            items.push(item);
            start = index + ch.len_utf8();
        }
    }

    if in_text {
        return Err(());
    }

    let item = input[start..].trim();
    if item.is_empty() {
        return Err(());
    }
    items.push(item);
    Ok(items)
}

fn starts_with_keyword_sequence(input: &str, keywords: &[&str]) -> bool {
    let mut parser = Parser::new(input);
    for (index, keyword) in keywords.iter().enumerate() {
        if index > 0 && parser.require_space().is_err() {
            return false;
        }
        if parser.consume_keyword(keyword).is_err() {
            return false;
        }
    }
    true
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }

    fn skip_space(&mut self) {
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_whitespace() {
                break;
            }
            self.offset += ch.len_utf8();
        }
    }

    fn require_space(&mut self) -> Result<(), ()> {
        let mut consumed = false;
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_whitespace() {
                break;
            }
            consumed = true;
            self.offset += ch.len_utf8();
        }
        consumed.then_some(()).ok_or(())
    }

    fn consume_keyword(&mut self, keyword: &str) -> Result<(), ()> {
        let end = self.offset.checked_add(keyword.len()).ok_or(())?;
        let candidate = self.input.get(self.offset..end).ok_or(())?;
        if !candidate.eq_ignore_ascii_case(keyword) {
            return Err(());
        }
        if self
            .input
            .get(end..)
            .and_then(|remaining| remaining.chars().next())
            .is_some_and(is_identifier_continue)
        {
            return Err(());
        }
        self.offset = end;
        Ok(())
    }

    fn consume_identifier(&mut self) -> Result<String, ()> {
        let remaining = self.input.get(self.offset..).ok_or(())?;
        let mut chars = remaining.char_indices();
        let Some((_, first)) = chars.next() else {
            return Err(());
        };
        if !is_identifier_start(first) {
            return Err(());
        }

        let mut end = self.offset + first.len_utf8();
        for (relative, ch) in chars {
            if !is_identifier_continue(ch) {
                break;
            }
            end = self.offset + relative + ch.len_utf8();
        }

        let identifier = self.input[self.offset..end].to_string();
        self.offset = end;
        Ok(identifier)
    }

    fn consume_int_literal(&mut self) -> Result<i64, ()> {
        let remaining = self.input.get(self.offset..).ok_or(())?;
        let mut end = self.offset;
        let mut chars = remaining.char_indices();
        if let Some((_, ch)) = chars.next() {
            if ch == '+' || ch == '-' {
                end += ch.len_utf8();
            }
        }

        let digit_start = end;
        for (relative, ch) in self.input[digit_start..].char_indices() {
            if !ch.is_ascii_digit() {
                break;
            }
            end = digit_start + relative + ch.len_utf8();
        }

        if end == digit_start {
            return Err(());
        }
        let raw = &self.input[self.offset..end];
        self.offset = end;
        raw.parse::<i64>().map_err(|_| ())
    }

    fn consume_char(&mut self, expected: char) -> Result<(), ()> {
        if self.peek_char() != Some(expected) {
            return Err(());
        }
        self.offset += expected.len_utf8();
        Ok(())
    }

    fn consume_until_matching_close_paren(&mut self) -> Result<&'a str, ()> {
        let start = self.offset;
        let mut in_text = false;
        for (relative, ch) in self.input[self.offset..].char_indices() {
            if ch == '\'' {
                in_text = !in_text;
            }
            if ch == ')' && !in_text {
                let end = self.offset + relative;
                self.offset = end + ch.len_utf8();
                return Ok(&self.input[start..end]);
            }
        }
        Err(())
    }

    fn finish(&self) -> Result<(), ()> {
        (self.offset == self.input.len()).then_some(()).ok_or(())
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.offset..)?.chars().next()
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn encode_catalog_record(table: &str, columns: &[Column]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(CATALOG_RECORD);
    write_string_u16(&mut record, table);
    write_u16(&mut record, columns.len() as u16);
    for column in columns {
        write_string_u16(&mut record, &column.name);
        record.push(column.column_type.to_byte());
    }
    if let Some(primary_key_column) = columns.iter().position(|column| column.is_primary_key) {
        record.push(b'P');
        write_u16(&mut record, primary_key_column as u16);
    }
    record
}

fn encode_row_record(table: &str, values: &[Value]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(ROW_RECORD);
    write_string_u16(&mut record, table);
    write_u16(&mut record, values.len() as u16);
    for value in values {
        record.push(value.column_type().to_byte());
        match value {
            Value::Int(value) => write_string_u32(&mut record, &value.to_string()),
            Value::Text(value) => write_string_u32(&mut record, value),
        }
    }
    record
}

fn decode_record(record: &[u8]) -> Result<LogicalRecord, SqlError> {
    if !record.starts_with(SQL_RECORD_PREFIX) {
        return Err(SqlError::InvalidStorageRecord);
    }

    let mut reader = RecordReader::new(&record[SQL_RECORD_PREFIX.len()..]);
    let kind = reader.read_u8()?;
    match kind {
        CATALOG_RECORD => {
            let table = reader.read_string_u16()?;
            let column_count = reader.read_u16()? as usize;
            let mut columns = Vec::with_capacity(column_count);
            for _ in 0..column_count {
                let name = reader.read_string_u16()?;
                let column_type = ColumnType::from_byte(reader.read_u8()?)
                    .ok_or(SqlError::InvalidStorageRecord)?;
                columns.push(Column {
                    name,
                    column_type,
                    is_primary_key: false,
                });
            }
            if !reader.is_finished() {
                let extension = reader.read_u8()?;
                if extension != b'P' {
                    return Err(SqlError::InvalidStorageRecord);
                }
                let primary_key_column = reader.read_u16()? as usize;
                let Some(column) = columns.get_mut(primary_key_column) else {
                    return Err(SqlError::InvalidStorageRecord);
                };
                column.is_primary_key = true;
            }
            reader.finish()?;
            Ok(LogicalRecord::Catalog { table, columns })
        }
        ROW_RECORD => {
            let table = reader.read_string_u16()?;
            let value_count = reader.read_u16()? as usize;
            let mut values = Vec::with_capacity(value_count);
            for _ in 0..value_count {
                let value_type = ColumnType::from_byte(reader.read_u8()?)
                    .ok_or(SqlError::InvalidStorageRecord)?;
                let raw_value = reader.read_string_u32()?;
                let value = match value_type {
                    ColumnType::Int => {
                        let parsed = raw_value
                            .parse::<i64>()
                            .map_err(|_| SqlError::InvalidStorageRecord)?;
                        if raw_value != parsed.to_string() {
                            return Err(SqlError::InvalidStorageRecord);
                        }
                        Value::Int(parsed)
                    }
                    ColumnType::Text => Value::Text(raw_value),
                };
                values.push(value);
            }
            reader.finish()?;
            Ok(LogicalRecord::Row { table, values })
        }
        _ => Err(SqlError::InvalidStorageRecord),
    }
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_string_u16(output: &mut Vec<u8>, value: &str) {
    write_u16(output, value.len() as u16);
    output.extend_from_slice(value.as_bytes());
}

fn write_string_u32(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u32).to_le_bytes());
    output.extend_from_slice(value.as_bytes());
}

struct RecordReader<'a> {
    record: &'a [u8],
    offset: usize,
}

impl<'a> RecordReader<'a> {
    fn new(record: &'a [u8]) -> Self {
        Self { record, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, SqlError> {
        let byte = *self
            .record
            .get(self.offset)
            .ok_or(SqlError::InvalidStorageRecord)?;
        self.offset += 1;
        Ok(byte)
    }

    fn read_u16(&mut self) -> Result<u16, SqlError> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_string_u16(&mut self) -> Result<String, SqlError> {
        let len = self.read_u16()? as usize;
        self.read_string(len)
    }

    fn read_string_u32(&mut self) -> Result<String, SqlError> {
        let bytes = self.read_exact(4)?;
        let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        self.read_string(len)
    }

    fn read_string(&mut self, len: usize) -> Result<String, SqlError> {
        let bytes = self.read_exact(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| SqlError::InvalidStorageRecord)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], SqlError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(SqlError::InvalidStorageRecord)?;
        let bytes = self
            .record
            .get(self.offset..end)
            .ok_or(SqlError::InvalidStorageRecord)?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(&self) -> Result<(), SqlError> {
        (self.offset == self.record.len())
            .then_some(())
            .ok_or(SqlError::InvalidStorageRecord)
    }

    fn is_finished(&self) -> bool {
        self.offset == self.record.len()
    }
}
