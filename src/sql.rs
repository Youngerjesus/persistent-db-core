use crate::index::{PrimaryIndex, SecondaryIndex};
use crate::storage::{self, PageStore, StorageError};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const CATALOG_RECORD: u8 = b'C';
const ROW_RECORD: u8 = b'R';
const SECONDARY_METADATA_RECORD: u8 = b'X';
const SECONDARY_ENTRY_RECORD: u8 = b'E';
const INDEXED_ROW_RECORD: u8 = b'I';
const UPDATE_ROW_RECORD: u8 = b'U';
const DELETE_ROW_RECORD: u8 = b'D';

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPath {
    PrimaryIndex {
        table: String,
        column: String,
    },
    SecondaryIndexEquality {
        table: String,
        index: String,
        column: String,
    },
    SecondaryIndexRange {
        table: String,
        index: String,
        column: String,
    },
    FullTableScan {
        table: String,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TieBreakMode {
    PrimaryKey,
    RowPosition,
}

impl TieBreakMode {
    fn to_byte(self) -> u8 {
        match self {
            TieBreakMode::PrimaryKey => b'P',
            TieBreakMode::RowPosition => b'R',
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'P' => Some(TieBreakMode::PrimaryKey),
            b'R' => Some(TieBreakMode::RowPosition),
            _ => None,
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
struct SecondaryIndexState {
    build_id: u64,
    name: String,
    indexed_column: usize,
    tie_break_mode: TieBreakMode,
    index: SecondaryIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Option<Vec<Value>>>,
    primary_key_column: Option<usize>,
    primary_index: PrimaryIndex,
    secondary_indexes: Vec<SecondaryIndexState>,
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
    CreateIndex {
        index: String,
        table: String,
        column: String,
    },
    Insert {
        table: String,
        values: Vec<Value>,
    },
    Update {
        table: String,
        set_column: String,
        value: Value,
        where_column: String,
        key: i64,
    },
    Delete {
        table: String,
        where_column: String,
        key: i64,
    },
    SelectAll {
        table: String,
    },
    SelectWhere {
        table: String,
        column: String,
        predicate: Predicate,
        raw: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Predicate {
    Equality(i64),
    Range { low: i64, high: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LogicalRecord {
    Catalog {
        table: String,
        columns: Vec<Column>,
    },
    Row {
        table: String,
        values: Vec<Value>,
    },
    SecondaryIndexEntry {
        build_id: u64,
        index_name: String,
        indexed_key: i64,
        tie_break: i64,
        row_position: u64,
    },
    SecondaryIndexMetadata {
        build_id: u64,
        index_name: String,
        table_name: String,
        indexed_column: u16,
        tie_break_mode: TieBreakMode,
    },
    IndexedRow {
        table: String,
        values: Vec<Value>,
        entries: Vec<EmbeddedIndexEntry>,
    },
    UpdateRow {
        table: String,
        row_position: u64,
        values: Vec<Value>,
        entries: Vec<EmbeddedIndexEntry>,
    },
    DeleteRow {
        table: String,
        row_position: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EmbeddedIndexEntry {
    build_id: u64,
    index_name_key: String,
    index_name: String,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
}

#[derive(Debug, Default)]
struct Database {
    tables: Vec<Table>,
}

impl Database {
    fn from_records(records: Vec<Vec<u8>>) -> Result<Self, SqlError> {
        Self::from_records_with_check_label(records).map_err(|_| SqlError::InvalidStorageRecord)
    }

    fn from_records_with_check_label(records: Vec<Vec<u8>>) -> Result<Self, &'static str> {
        let mut database = Database::default();
        let mut pending_entries: BTreeMap<(String, u64), Vec<EmbeddedIndexEntry>> = BTreeMap::new();
        let mut index_names = Vec::<String>::new();
        let mut committed_index_builds = Vec::<(String, u64)>::new();

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
                        secondary_indexes: Vec::new(),
                    });
                }
                LogicalRecord::Row { table, values } => {
                    let table = database
                        .find_table_mut(&table)
                        .ok_or("catalog/record invariant")?;
                    if !table.secondary_indexes.is_empty() {
                        return Err("secondary index");
                    }
                    validate_values_for_table(table, &values)
                        .map_err(|_| "catalog/record invariant")?;
                    validate_primary_key_available(table, &values).map_err(|_| "primary index")?;
                    append_loaded_row_after_validation(table, values)
                        .map_err(|_| "catalog/record invariant")?;
                }
                LogicalRecord::SecondaryIndexEntry {
                    build_id,
                    index_name,
                    indexed_key,
                    tie_break,
                    row_position,
                } => {
                    let entry = EmbeddedIndexEntry {
                        build_id,
                        index_name_key: normalize_identifier(&index_name),
                        index_name,
                        indexed_key,
                        tie_break,
                        row_position,
                    };
                    pending_entries
                        .entry((entry.index_name_key.clone(), build_id))
                        .or_default()
                        .push(entry);
                }
                LogicalRecord::SecondaryIndexMetadata {
                    build_id,
                    index_name,
                    table_name,
                    indexed_column,
                    tie_break_mode,
                } => {
                    let index_key = normalize_identifier(&index_name);
                    if index_names.iter().any(|existing| existing == &index_key) {
                        return Err("secondary index");
                    }
                    let table = database
                        .find_table_mut(&table_name)
                        .ok_or("secondary index")?;
                    let column_index = indexed_column as usize;
                    let Some(column) = table.columns.get(column_index) else {
                        return Err("secondary index");
                    };
                    if column.column_type != ColumnType::Int {
                        return Err("secondary index");
                    }
                    if tie_break_mode == TieBreakMode::PrimaryKey
                        && table.primary_key_column.is_none()
                    {
                        return Err("secondary index");
                    }

                    let index_build_key = (index_key.clone(), build_id);
                    let entries = pending_entries.remove(&index_build_key).unwrap_or_default();
                    let mut state = SecondaryIndexState {
                        build_id,
                        name: index_name,
                        indexed_column: column_index,
                        tie_break_mode,
                        index: SecondaryIndex::new(),
                    };
                    validate_and_insert_entries(table, &mut state, &entries)?;
                    table.secondary_indexes.push(state);
                    index_names.push(index_key);
                    committed_index_builds.push(index_build_key);
                }
                LogicalRecord::IndexedRow {
                    table,
                    values,
                    entries,
                } => {
                    let table = database
                        .find_table_mut(&table)
                        .ok_or("catalog/record invariant")?;
                    if table.secondary_indexes.is_empty() {
                        return Err("secondary index");
                    }
                    validate_values_for_table(table, &values)
                        .map_err(|_| "catalog/record invariant")?;
                    validate_primary_key_available(table, &values).map_err(|_| "primary index")?;

                    let row_position = table.rows.len();
                    let expected_entries = expected_entries_for_row(table, &values, row_position)?;
                    if canonical_entries(&entries) != canonical_entries(&expected_entries) {
                        return Err("secondary index");
                    }

                    for entry in &expected_entries {
                        let index = table
                            .secondary_indexes
                            .iter_mut()
                            .find(|index| {
                                index.build_id == entry.build_id
                                    && index.name.eq_ignore_ascii_case(&entry.index_name)
                            })
                            .ok_or("secondary index")?;
                        index
                            .index
                            .insert(entry.indexed_key, entry.tie_break, row_position)
                            .map_err(|_| "secondary index")?;
                    }
                    append_loaded_row_after_validation(table, values)
                        .map_err(|_| "catalog/record invariant")?;
                }
                LogicalRecord::UpdateRow {
                    table,
                    row_position,
                    values,
                    entries,
                } => {
                    let table = database
                        .find_table_mut(&table)
                        .ok_or("catalog/record invariant")?;
                    apply_loaded_update_after_validation(
                        table,
                        row_position as usize,
                        values,
                        entries,
                    )
                    .map_err(|_| "secondary index")?;
                }
                LogicalRecord::DeleteRow {
                    table,
                    row_position,
                } => {
                    let table = database
                        .find_table_mut(&table)
                        .ok_or("catalog/record invariant")?;
                    apply_loaded_delete_after_validation(table, row_position as usize)
                        .map_err(|_| "secondary index")?;
                }
            }
        }

        if pending_entries.keys().any(|key| {
            committed_index_builds
                .iter()
                .any(|committed| committed == key)
        }) {
            return Err("secondary index");
        }

        validate_secondary_indexes(&database)?;
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

    fn contains_index_name(&self, index_name: &str) -> bool {
        self.tables.iter().any(|table| {
            table
                .secondary_indexes
                .iter()
                .any(|index| index.name.eq_ignore_ascii_case(index_name))
        })
    }
}

pub fn execute(path: impl AsRef<Path>, sql: &str) -> Result<String, SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    validate_replayable_records_before_open(path)?;
    let mut store = PageStore::open(path)?;
    let records = store.read_records()?;
    let mut logical_record_count = records.len() as u64;
    let mut database = Database::from_records(records)?;
    let statements = parse_statements(sql)?;
    let mut stdout = String::new();

    for statement in statements {
        match statement {
            Statement::CreateTable { table, columns } => {
                execute_create_table(&mut store, &mut database, table, columns)?;
                logical_record_count += 1;
            }
            Statement::CreateIndex {
                index,
                table,
                column,
            } => {
                let appended = execute_create_index(
                    &mut store,
                    &mut database,
                    logical_record_count,
                    index,
                    table,
                    column,
                )?;
                logical_record_count += appended;
            }
            Statement::Insert { table, values } => {
                execute_insert(&mut store, &mut database, table, values)?;
                logical_record_count += 1;
            }
            Statement::Update {
                table,
                set_column,
                value,
                where_column,
                key,
            } => {
                let appended = execute_update(
                    &mut store,
                    &mut database,
                    table,
                    set_column,
                    value,
                    where_column,
                    key,
                )?;
                logical_record_count += appended;
            }
            Statement::Delete {
                table,
                where_column,
                key,
            } => {
                let appended = execute_delete(&mut store, &mut database, table, where_column, key)?;
                logical_record_count += appended;
            }
            Statement::SelectAll { table } => {
                execute_select(&database, &table, &mut stdout)?;
            }
            Statement::SelectWhere {
                table,
                column,
                predicate,
                raw,
            } => {
                execute_select_where(&database, &table, &column, predicate, &raw, &mut stdout)?;
            }
        }
    }

    Ok(stdout)
}

pub fn validate_records_for_check(records: Vec<Vec<u8>>) -> Result<(), &'static str> {
    Database::from_records_with_check_label(records)?;
    Ok(())
}

pub fn plan_query_path_for_test(path: impl AsRef<Path>, sql: &str) -> Result<QueryPath, SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    validate_replayable_records_before_open(path)?;
    let mut store = PageStore::open(path)?;
    let database = Database::from_records(store.read_records()?)?;
    let statements = parse_statements(sql)?;
    if statements.len() != 1 {
        return Err(SqlError::Malformed(sql.trim().to_string()));
    }
    plan_query_path(&database, &statements[0])
}

pub fn plan_selects_for_bench(
    path: impl AsRef<Path>,
    sql_texts: &[String],
) -> Result<Vec<QueryPath>, SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    validate_replayable_records_before_open(path)?;
    let mut store = PageStore::open(path)?;
    let database = Database::from_records(store.read_records()?)?;
    let mut paths = Vec::with_capacity(sql_texts.len());

    for sql_text in sql_texts {
        let statements = parse_statements(sql_text)?;
        if statements.len() != 1 {
            return Err(SqlError::Malformed(sql_text.trim().to_string()));
        }
        paths.push(plan_query_path(&database, &statements[0])?);
    }

    Ok(paths)
}

pub fn execute_selects_for_bench(
    path: impl AsRef<Path>,
    sql_texts: &[String],
) -> Result<Vec<String>, SqlError> {
    execute_select_batches_for_bench(path, &[sql_texts.to_vec()]).map(|mut batches| {
        let (_, outputs) = batches.remove(0);
        outputs
    })
}

pub fn execute_select_batches_for_bench(
    path: impl AsRef<Path>,
    batches: &[Vec<String>],
) -> Result<Vec<(f64, Vec<String>)>, SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    validate_replayable_records_before_open(path)?;
    let mut store = PageStore::open(path)?;
    let database = Database::from_records(store.read_records()?)?;
    let parsed_batches = batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .map(|sql_text| {
                    let statements = parse_statements(sql_text)?;
                    if statements.len() != 1 {
                        return Err(SqlError::Malformed(sql_text.trim().to_string()));
                    }
                    Ok(statements[0].clone())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut measured_batches = Vec::with_capacity(batches.len());

    for batch in parsed_batches {
        let start = Instant::now();
        let mut outputs = Vec::with_capacity(batch.len());
        for statement in batch {
            let mut stdout = String::new();
            match statement {
                Statement::SelectAll { table } => {
                    execute_select(&database, &table, &mut stdout)?;
                }
                Statement::SelectWhere {
                    table,
                    column,
                    predicate,
                    raw,
                } => {
                    execute_select_where(&database, &table, &column, predicate, &raw, &mut stdout)?;
                }
                _ => return Err(SqlError::Unsupported(String::new())),
            }
            outputs.push(stdout);
        }
        measured_batches.push((
            start.elapsed().as_secs_f64().max(0.000_001) * 1000.0,
            outputs,
        ));
    }

    Ok(measured_batches)
}

pub fn create_section14_fixture_for_bench(
    path: impl AsRef<Path>,
    rows: &[(i64, i64, String)],
    index_name: &str,
) -> Result<(), SqlError> {
    initialize_empty_sql_file(path.as_ref())?;
    let mut store = PageStore::open(path)?;
    let columns = vec![
        Column {
            name: "id".to_string(),
            column_type: ColumnType::Int,
            is_primary_key: true,
        },
        Column {
            name: "group_key".to_string(),
            column_type: ColumnType::Int,
            is_primary_key: false,
        },
        Column {
            name: "payload".to_string(),
            column_type: ColumnType::Text,
            is_primary_key: false,
        },
    ];
    store.append_record(&encode_catalog_record("bench_items", &columns))?;
    let state = SecondaryIndexState {
        build_id: 1,
        name: index_name.to_string(),
        indexed_column: 1,
        tie_break_mode: TieBreakMode::PrimaryKey,
        index: SecondaryIndex::new(),
    };
    store.append_record(&encode_secondary_metadata_record(&state, "bench_items"))?;

    for (row_position, (id, group_key, payload)) in rows.iter().enumerate() {
        let values = vec![
            Value::Int(*id),
            Value::Int(*group_key),
            Value::Text(payload.clone()),
        ];
        let entry = EmbeddedIndexEntry {
            build_id: 1,
            index_name_key: normalize_identifier(index_name),
            index_name: index_name.to_string(),
            indexed_key: *group_key,
            tie_break: *id,
            row_position: row_position as u64,
        };
        store.append_record(&encode_indexed_row_record("bench_items", &values, &[entry]))?;
    }

    Ok(())
}

pub fn create_section14_wal_recovery_fixture_for_bench(
    path: impl AsRef<Path>,
    rows: &[(i64, i64, String)],
    index_name: &str,
) -> Result<(), SqlError> {
    let path = path.as_ref();
    initialize_empty_sql_file(path)?;
    let mut store = PageStore::open(path)?;
    let columns = vec![
        Column {
            name: "id".to_string(),
            column_type: ColumnType::Int,
            is_primary_key: true,
        },
        Column {
            name: "group_key".to_string(),
            column_type: ColumnType::Int,
            is_primary_key: false,
        },
        Column {
            name: "payload".to_string(),
            column_type: ColumnType::Text,
            is_primary_key: false,
        },
    ];
    store.append_record(&encode_catalog_record("bench_items", &columns))?;
    let state = SecondaryIndexState {
        build_id: 1,
        name: index_name.to_string(),
        indexed_column: 1,
        tie_break_mode: TieBreakMode::PrimaryKey,
        index: SecondaryIndex::new(),
    };
    store.append_record(&encode_secondary_metadata_record(&state, "bench_items"))?;
    drop(store);

    let mut record_count_before = 2u64;
    for (row_position, (id, group_key, payload)) in rows.iter().enumerate() {
        let values = vec![
            Value::Int(*id),
            Value::Int(*group_key),
            Value::Text(payload.clone()),
        ];
        let entry = EmbeddedIndexEntry {
            build_id: 1,
            index_name_key: normalize_identifier(index_name),
            index_name: index_name.to_string(),
            indexed_key: *group_key,
            tie_break: *id,
            row_position: row_position as u64,
        };
        storage::append_committed_wal_record_for_bench(
            path,
            record_count_before,
            &encode_indexed_row_record("bench_items", &values, &[entry]),
        )?;
        record_count_before += 1;
    }

    Ok(())
}

fn plan_query_path(database: &Database, statement: &Statement) -> Result<QueryPath, SqlError> {
    match statement {
        Statement::SelectAll { table } => {
            let table = database
                .find_table(table)
                .ok_or_else(|| table_not_found(table))?;
            Ok(QueryPath::FullTableScan {
                table: table.name.clone(),
            })
        }
        Statement::SelectWhere {
            table,
            column,
            predicate,
            raw,
        } => {
            let table = database
                .find_table(table)
                .ok_or_else(|| table_not_found(table))?;
            if let Some(index) = table.secondary_index_for_column(column) {
                let column_name = table.columns[index.indexed_column].name.clone();
                return match predicate {
                    Predicate::Equality(_) => Ok(QueryPath::SecondaryIndexEquality {
                        table: table.name.clone(),
                        index: index.name.clone(),
                        column: column_name,
                    }),
                    Predicate::Range { .. } => Ok(QueryPath::SecondaryIndexRange {
                        table: table.name.clone(),
                        index: index.name.clone(),
                        column: column_name,
                    }),
                };
            }
            if let Some(primary_key_column) = table.primary_key_column {
                if table.columns[primary_key_column]
                    .name
                    .eq_ignore_ascii_case(column)
                    && matches!(predicate, Predicate::Equality(_))
                {
                    return Ok(QueryPath::PrimaryIndex {
                        table: table.name.clone(),
                        column: table.columns[primary_key_column].name.clone(),
                    });
                }
            }
            Err(SqlError::Unsupported(raw.clone()))
        }
        _ => Err(SqlError::Unsupported(String::new())),
    }
}

trait TableSecondaryLookup {
    fn secondary_index_for_column(&self, column: &str) -> Option<&SecondaryIndexState>;
}

impl TableSecondaryLookup for Table {
    fn secondary_index_for_column(&self, column: &str) -> Option<&SecondaryIndexState> {
        self.secondary_indexes.iter().find(|index| {
            self.columns[index.indexed_column]
                .name
                .eq_ignore_ascii_case(column)
        })
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
        secondary_indexes: Vec::new(),
    });
    Ok(())
}

fn execute_create_index(
    store: &mut PageStore,
    database: &mut Database,
    build_id: u64,
    index_name: String,
    table_name: String,
    column_name: String,
) -> Result<u64, SqlError> {
    if database.contains_index_name(&index_name) {
        return Err(SqlError::Semantic {
            message: format!("index already exists: {index_name}"),
            hint: "use a new index name for CREATE INDEX in this database.",
        });
    }

    let table = database
        .find_table_mut(&table_name)
        .ok_or_else(|| table_not_found_for_create_index(&table_name))?;
    let Some(indexed_column) = table
        .columns
        .iter()
        .position(|column| column.name.eq_ignore_ascii_case(&column_name))
    else {
        return Err(SqlError::Semantic {
            message: format!("column not found for index {index_name}: {column_name}"),
            hint: "create the index on an existing table column.",
        });
    };
    if table.columns[indexed_column].column_type != ColumnType::Int {
        return Err(SqlError::Semantic {
            message: format!("secondary index column must be INT: {column_name}"),
            hint: "this SQL slice supports secondary indexes only on INT columns.",
        });
    }

    let tie_break_mode = if table.primary_key_column.is_some() {
        TieBreakMode::PrimaryKey
    } else {
        TieBreakMode::RowPosition
    };
    let mut state = SecondaryIndexState {
        build_id,
        name: index_name,
        indexed_column,
        tie_break_mode,
        index: SecondaryIndex::new(),
    };
    let mut entries = Vec::new();
    for (row_position, row) in table.rows.iter().enumerate() {
        let Some(row) = row else {
            continue;
        };
        entries.push(
            expected_entry_for_index(&state, table, row, row_position)
                .map_err(|_| SqlError::InvalidStorageRecord)?,
        );
    }
    for entry in &entries {
        store.append_record(&encode_secondary_entry_record(entry))?;
    }
    store.append_record(&encode_secondary_metadata_record(&state, &table.name))?;
    for entry in &entries {
        state
            .index
            .insert(
                entry.indexed_key,
                entry.tie_break,
                entry.row_position as usize,
            )
            .map_err(|_| SqlError::InvalidStorageRecord)?;
    }
    table.secondary_indexes.push(state);
    Ok(entries.len() as u64 + 1)
}

fn execute_insert(
    store: &mut PageStore,
    database: &mut Database,
    table: String,
    values: Vec<Value>,
) -> Result<(), SqlError> {
    let existing = database
        .find_table_mut(&table)
        .ok_or_else(|| table_not_found(&table))?;
    validate_values_for_table(existing, &values)?;
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

    if existing.secondary_indexes.is_empty() {
        store.append_record(&encode_row_record(&existing.name, &values))?;
        append_loaded_row_after_validation(existing, values)?;
        return Ok(());
    }

    let row_position = existing.rows.len();
    let entries = expected_entries_for_row(existing, &values, row_position)
        .map_err(|_| SqlError::InvalidStorageRecord)?;
    store.append_record(&encode_indexed_row_record(
        &existing.name,
        &values,
        &entries,
    ))?;
    for entry in &entries {
        let index = existing
            .secondary_indexes
            .iter_mut()
            .find(|index| {
                index.build_id == entry.build_id
                    && index.name.eq_ignore_ascii_case(&entry.index_name)
            })
            .ok_or(SqlError::InvalidStorageRecord)?;
        index
            .index
            .insert(entry.indexed_key, entry.tie_break, row_position)
            .map_err(|_| SqlError::InvalidStorageRecord)?;
    }
    append_loaded_row_after_validation(existing, values)?;
    Ok(())
}

fn execute_update(
    store: &mut PageStore,
    database: &mut Database,
    table: String,
    set_column: String,
    value: Value,
    where_column: String,
    key: i64,
) -> Result<u64, SqlError> {
    let existing = database
        .find_table_mut(&table)
        .ok_or_else(|| table_not_found(&table))?;
    let primary_key_column = require_primary_key_predicate(existing, &where_column)?;
    let Some(set_column_index) = existing
        .columns
        .iter()
        .position(|column| column.name.eq_ignore_ascii_case(&set_column))
    else {
        return Err(SqlError::Semantic {
            message: format!("column not found for UPDATE: {set_column}"),
            hint: "UPDATE can SET an existing non-primary-key column.",
        });
    };
    if set_column_index == primary_key_column {
        return Err(SqlError::Semantic {
            message: format!("cannot update primary key column: {set_column}"),
            hint: "this SQL slice supports UPDATE of non-primary-key columns only.",
        });
    }
    if existing.columns[set_column_index].column_type != value.column_type() {
        return Err(SqlError::Semantic {
            message: format!(
                "type mismatch for column {}: expected {}, got {}",
                existing.columns[set_column_index].name,
                existing.columns[set_column_index].column_type.as_str(),
                value.column_type().as_str()
            ),
            hint: "UPDATE values must match the declared column type.",
        });
    }
    let Some(row_position) = existing.primary_index.get(key) else {
        return Ok(0);
    };

    let current = existing
        .rows
        .get(row_position)
        .and_then(Option::as_ref)
        .ok_or(SqlError::InvalidStorageRecord)?;
    let mut values = current.clone();
    values[set_column_index] = value;
    validate_values_for_table(existing, &values)?;
    let entries = expected_entries_for_row(existing, &values, row_position)
        .map_err(|_| SqlError::InvalidStorageRecord)?;
    store.append_record(&encode_update_row_record(
        &existing.name,
        row_position,
        &values,
        &entries,
    ))?;
    apply_update_to_table(existing, row_position, values, &entries)?;
    Ok(1)
}

fn execute_delete(
    store: &mut PageStore,
    database: &mut Database,
    table: String,
    where_column: String,
    key: i64,
) -> Result<u64, SqlError> {
    let existing = database
        .find_table_mut(&table)
        .ok_or_else(|| table_not_found(&table))?;
    require_primary_key_predicate(existing, &where_column)?;
    let Some(row_position) = existing.primary_index.get(key) else {
        return Ok(0);
    };
    if existing
        .rows
        .get(row_position)
        .and_then(Option::as_ref)
        .is_none()
    {
        return Err(SqlError::InvalidStorageRecord);
    }
    store.append_record(&encode_delete_row_record(&existing.name, row_position))?;
    apply_delete_to_table(existing, row_position)?;
    Ok(1)
}

fn execute_select(database: &Database, table: &str, stdout: &mut String) -> Result<(), SqlError> {
    let existing = database
        .find_table(table)
        .ok_or_else(|| table_not_found(table))?;
    write_header(existing, stdout);
    if existing.primary_key_column.is_some() {
        for row_position in existing.primary_index.ordered_positions() {
            let row = existing.rows[row_position]
                .as_ref()
                .ok_or(SqlError::InvalidStorageRecord)?;
            write_row(row, stdout);
        }
    } else {
        for row in existing.rows.iter().flatten() {
            write_row(row, stdout);
        }
    }
    Ok(())
}

fn execute_select_where(
    database: &Database,
    table: &str,
    column: &str,
    predicate: Predicate,
    raw: &str,
    stdout: &mut String,
) -> Result<(), SqlError> {
    let existing = database
        .find_table(table)
        .ok_or_else(|| table_not_found(table))?;
    write_header(existing, stdout);
    if let Some(index) = existing.secondary_index_for_column(column) {
        let positions = match predicate {
            Predicate::Equality(key) => index.index.equality_positions(key),
            Predicate::Range { low, high } => index.index.range_positions(low, high),
        };
        for row_position in positions {
            let row = existing.rows[row_position]
                .as_ref()
                .ok_or(SqlError::InvalidStorageRecord)?;
            write_row(row, stdout);
        }
        return Ok(());
    }

    if let Some(primary_key_column) = existing.primary_key_column {
        if existing.columns[primary_key_column]
            .name
            .eq_ignore_ascii_case(column)
        {
            let Predicate::Equality(key) = predicate else {
                return Err(SqlError::Unsupported(raw.to_string()));
            };
            if let Some(row_position) = existing.primary_index.get(key) {
                let row = existing.rows[row_position]
                    .as_ref()
                    .ok_or(SqlError::InvalidStorageRecord)?;
                write_row(row, stdout);
            }
            return Ok(());
        }
    }
    Err(SqlError::Unsupported(raw.to_string()))
}

fn append_loaded_row_after_validation(
    table: &mut Table,
    values: Vec<Value>,
) -> Result<(), SqlError> {
    if let Some(primary_key_column) = table.primary_key_column {
        let Value::Int(key) = values[primary_key_column] else {
            return Err(SqlError::InvalidStorageRecord);
        };
        table
            .primary_index
            .insert(key, table.rows.len())
            .map_err(|_| SqlError::InvalidStorageRecord)?;
    }
    table.rows.push(Some(values));
    Ok(())
}

fn apply_loaded_update_after_validation(
    table: &mut Table,
    row_position: usize,
    values: Vec<Value>,
    entries: Vec<EmbeddedIndexEntry>,
) -> Result<(), SqlError> {
    validate_values_for_table(table, &values).map_err(|_| SqlError::InvalidStorageRecord)?;
    let expected_entries = expected_entries_for_row(table, &values, row_position)
        .map_err(|_| SqlError::InvalidStorageRecord)?;
    if canonical_entries(&entries) != canonical_entries(&expected_entries) {
        return Err(SqlError::InvalidStorageRecord);
    }
    apply_update_to_table(table, row_position, values, &expected_entries)
}

fn apply_loaded_delete_after_validation(
    table: &mut Table,
    row_position: usize,
) -> Result<(), SqlError> {
    apply_delete_to_table(table, row_position)
}

fn apply_update_to_table(
    table: &mut Table,
    row_position: usize,
    values: Vec<Value>,
    new_entries: &[EmbeddedIndexEntry],
) -> Result<(), SqlError> {
    let old_values = table
        .rows
        .get(row_position)
        .and_then(Option::as_ref)
        .ok_or(SqlError::InvalidStorageRecord)?
        .clone();
    let old_entries = expected_entries_for_row(table, &old_values, row_position)
        .map_err(|_| SqlError::InvalidStorageRecord)?;
    if primary_key_value(table, &old_values)? != primary_key_value(table, &values)? {
        return Err(SqlError::InvalidStorageRecord);
    }

    for entry in &old_entries {
        let index = table
            .secondary_indexes
            .iter_mut()
            .find(|index| {
                index.build_id == entry.build_id
                    && index.name.eq_ignore_ascii_case(&entry.index_name)
            })
            .ok_or(SqlError::InvalidStorageRecord)?;
        if index.index.remove(entry.indexed_key, entry.tie_break) != Some(row_position) {
            return Err(SqlError::InvalidStorageRecord);
        }
    }
    for entry in new_entries {
        let index = table
            .secondary_indexes
            .iter_mut()
            .find(|index| {
                index.build_id == entry.build_id
                    && index.name.eq_ignore_ascii_case(&entry.index_name)
            })
            .ok_or(SqlError::InvalidStorageRecord)?;
        index
            .index
            .insert(entry.indexed_key, entry.tie_break, row_position)
            .map_err(|_| SqlError::InvalidStorageRecord)?;
    }
    table.rows[row_position] = Some(values);
    Ok(())
}

fn apply_delete_to_table(table: &mut Table, row_position: usize) -> Result<(), SqlError> {
    let old_values = table
        .rows
        .get(row_position)
        .and_then(Option::as_ref)
        .ok_or(SqlError::InvalidStorageRecord)?
        .clone();
    if let Some(primary_key_column) = table.primary_key_column {
        let Value::Int(key) = old_values[primary_key_column] else {
            return Err(SqlError::InvalidStorageRecord);
        };
        if table.primary_index.remove(key) != Some(row_position) {
            return Err(SqlError::InvalidStorageRecord);
        }
    }
    let old_entries = expected_entries_for_row(table, &old_values, row_position)
        .map_err(|_| SqlError::InvalidStorageRecord)?;
    for entry in &old_entries {
        let index = table
            .secondary_indexes
            .iter_mut()
            .find(|index| {
                index.build_id == entry.build_id
                    && index.name.eq_ignore_ascii_case(&entry.index_name)
            })
            .ok_or(SqlError::InvalidStorageRecord)?;
        if index.index.remove(entry.indexed_key, entry.tie_break) != Some(row_position) {
            return Err(SqlError::InvalidStorageRecord);
        }
    }
    table.rows[row_position] = None;
    Ok(())
}

fn primary_key_value(table: &Table, values: &[Value]) -> Result<Option<i64>, SqlError> {
    let Some(primary_key_column) = table.primary_key_column else {
        return Ok(None);
    };
    let Value::Int(key) = values[primary_key_column] else {
        return Err(SqlError::InvalidStorageRecord);
    };
    Ok(Some(key))
}

fn require_primary_key_predicate(table: &Table, where_column: &str) -> Result<usize, SqlError> {
    let Some(primary_key_column) = table.primary_key_column else {
        return Err(SqlError::Semantic {
            message: format!("primary key predicate required for table {}", table.name),
            hint: "UPDATE and DELETE require WHERE on the INT PRIMARY KEY column.",
        });
    };
    if !table.columns[primary_key_column]
        .name
        .eq_ignore_ascii_case(where_column)
    {
        return Err(SqlError::Semantic {
            message: format!("primary key predicate required for table {}", table.name),
            hint: "UPDATE and DELETE require WHERE on the INT PRIMARY KEY column.",
        });
    }
    Ok(primary_key_column)
}

fn validate_values_for_table(table: &Table, values: &[Value]) -> Result<(), SqlError> {
    if table.columns.len() != values.len() {
        return Err(SqlError::Semantic {
            message: format!(
                "column count mismatch for table {}: expected {} values, got {}",
                table.name,
                table.columns.len(),
                values.len()
            ),
            hint: "INSERT values must match the table schema exactly.",
        });
    }
    for (column, value) in table.columns.iter().zip(values.iter()) {
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
        validate_loaded_value(value)?;
    }
    Ok(())
}

fn validate_primary_key_available(table: &Table, values: &[Value]) -> Result<(), SqlError> {
    if let Some(primary_key_column) = table.primary_key_column {
        let Value::Int(key) = values[primary_key_column] else {
            return Err(SqlError::InvalidStorageRecord);
        };
        if table.primary_index.get(key).is_some() {
            return Err(SqlError::InvalidStorageRecord);
        }
    }
    Ok(())
}

fn validate_and_insert_entries(
    table: &Table,
    state: &mut SecondaryIndexState,
    entries: &[EmbeddedIndexEntry],
) -> Result<(), &'static str> {
    let mut expected = Vec::new();
    for (row_position, row) in table.rows.iter().enumerate() {
        let Some(row) = row else {
            continue;
        };
        expected.push(expected_entry_for_index(state, table, row, row_position)?);
    }
    if canonical_entries(entries) != canonical_entries(&expected) {
        return Err("secondary index");
    }
    for entry in entries {
        state
            .index
            .insert(
                entry.indexed_key,
                entry.tie_break,
                entry.row_position as usize,
            )
            .map_err(|_| "secondary index")?;
    }
    Ok(())
}

fn validate_secondary_indexes(database: &Database) -> Result<(), &'static str> {
    for table in &database.tables {
        for state in &table.secondary_indexes {
            let mut rebuilt = SecondaryIndex::new();
            for (row_position, row) in table.rows.iter().enumerate() {
                let Some(row) = row else {
                    continue;
                };
                let entry = expected_entry_for_index(state, table, row, row_position)?;
                rebuilt
                    .insert(entry.indexed_key, entry.tie_break, row_position)
                    .map_err(|_| "secondary index")?;
            }
            if rebuilt != state.index {
                return Err("secondary index");
            }
        }
    }
    Ok(())
}

fn expected_entries_for_row(
    table: &Table,
    row: &[Value],
    row_position: usize,
) -> Result<Vec<EmbeddedIndexEntry>, &'static str> {
    let mut entries = Vec::new();
    for state in &table.secondary_indexes {
        entries.push(expected_entry_for_index(state, table, row, row_position)?);
    }
    Ok(entries)
}

fn expected_entry_for_index(
    state: &SecondaryIndexState,
    table: &Table,
    row: &[Value],
    row_position: usize,
) -> Result<EmbeddedIndexEntry, &'static str> {
    let Value::Int(indexed_key) = row[state.indexed_column] else {
        return Err("secondary index");
    };
    let tie_break = match state.tie_break_mode {
        TieBreakMode::PrimaryKey => {
            let primary_key_column = table.primary_key_column.ok_or("secondary index")?;
            let Value::Int(primary_key) = row[primary_key_column] else {
                return Err("secondary index");
            };
            primary_key
        }
        TieBreakMode::RowPosition => row_position as i64,
    };
    Ok(EmbeddedIndexEntry {
        build_id: state.build_id,
        index_name_key: normalize_identifier(&state.name),
        index_name: state.name.clone(),
        indexed_key,
        tie_break,
        row_position: row_position as u64,
    })
}

fn canonical_entries(entries: &[EmbeddedIndexEntry]) -> Vec<(u64, String, i64, i64, u64)> {
    let mut canonical: Vec<_> = entries
        .iter()
        .map(|entry| {
            (
                entry.build_id,
                entry.index_name_key.clone(),
                entry.indexed_key,
                entry.tie_break,
                entry.row_position,
            )
        })
        .collect();
    canonical.sort();
    canonical
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

fn validate_replayable_records_before_open(path: &Path) -> Result<(), SqlError> {
    if !path.exists() {
        return Ok(());
    }
    let snapshot = storage::read_records_for_check(path)?;
    let mut records = snapshot.records;
    records.extend(storage::read_committed_wal_records_for_check(
        path,
        snapshot.record_count,
    )?);
    Database::from_records(records)?;
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

fn table_not_found_for_create_index(table: &str) -> SqlError {
    SqlError::Semantic {
        message: format!("table not found: {table}"),
        hint: "create the table before INSERT, SELECT, or CREATE INDEX.",
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
    if starts_with_keyword_sequence(body, &["CREATE", "INDEX"]) {
        parse_create_index(body).map_err(|()| SqlError::Malformed(raw.to_string()))
    } else if starts_with_keyword_sequence(body, &["CREATE", "TABLE"]) {
        parse_create_table(body).map_err(|()| SqlError::Malformed(raw.to_string()))
    } else if starts_with_keyword_sequence(body, &["INSERT", "INTO"]) {
        parse_insert(body).map_err(|()| {
            if is_insert_with_column_list(body) {
                SqlError::Unsupported(raw.to_string())
            } else {
                SqlError::Malformed(raw.to_string())
            }
        })
    } else if starts_with_keyword_sequence(body, &["UPDATE"]) {
        parse_update(body).map_err(|()| SqlError::Unsupported(raw.to_string()))
    } else if starts_with_keyword_sequence(body, &["DELETE", "FROM"]) {
        parse_delete(body).map_err(|()| SqlError::Unsupported(raw.to_string()))
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

fn parse_create_index(body: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("CREATE")?;
    parser.require_space()?;
    parser.consume_keyword("INDEX")?;
    parser.require_space()?;
    let index = parser.consume_identifier()?;
    parser.require_space()?;
    parser.consume_keyword("ON")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char('(')?;
    parser.skip_space();
    let column = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char(')')?;
    parser.skip_space();
    parser.finish()?;
    Ok(Statement::CreateIndex {
        index,
        table,
        column,
    })
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
        let column_type = if column_parser.consume_keyword("INTEGER").is_ok()
            || column_parser.consume_keyword("INT").is_ok()
        {
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

fn parse_update(body: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("UPDATE")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.require_space()?;
    parser.consume_keyword("SET")?;
    parser.require_space()?;
    let set_column = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char('=')?;
    parser.skip_space();
    let value = parser.consume_value_literal()?;
    parser.require_space()?;
    parser.consume_keyword("WHERE")?;
    parser.require_space()?;
    let where_column = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char('=')?;
    parser.skip_space();
    let key = parser.consume_int_literal()?;
    parser.skip_space();
    parser.finish()?;
    Ok(Statement::Update {
        table,
        set_column,
        value,
        where_column,
        key,
    })
}

fn parse_delete(body: &str) -> Result<Statement, ()> {
    let mut parser = Parser::new(body);
    parser.consume_keyword("DELETE")?;
    parser.require_space()?;
    parser.consume_keyword("FROM")?;
    parser.require_space()?;
    let table = parser.consume_identifier()?;
    parser.require_space()?;
    parser.consume_keyword("WHERE")?;
    parser.require_space()?;
    let where_column = parser.consume_identifier()?;
    parser.skip_space();
    parser.consume_char('=')?;
    parser.skip_space();
    let key = parser.consume_int_literal()?;
    parser.skip_space();
    parser.finish()?;
    Ok(Statement::Delete {
        table,
        where_column,
        key,
    })
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
        if parser.consume_keyword("BETWEEN").is_ok() {
            parser.require_space()?;
            let low = parser.consume_int_literal()?;
            parser.require_space()?;
            parser.consume_keyword("AND")?;
            parser.require_space()?;
            let high = parser.consume_int_literal()?;
            parser.skip_space();
            parser.finish()?;
            return Ok(Statement::SelectWhere {
                table,
                column,
                predicate: Predicate::Range { low, high },
                raw: raw.to_string(),
            });
        }
        parser.consume_char('=')?;
        parser.skip_space();
        let key = parser.consume_int_literal()?;
        parser.skip_space();
        parser.finish()?;
        return Ok(Statement::SelectWhere {
            table,
            column,
            predicate: Predicate::Equality(key),
            raw: raw.to_string(),
        });
    }
    parser.finish()?;
    Ok(Statement::SelectAll { table })
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

fn normalize_identifier(value: &str) -> String {
    value.to_ascii_lowercase()
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

    fn consume_value_literal(&mut self) -> Result<Value, ()> {
        if self.peek_char() == Some('\'') {
            let start = self.offset;
            self.offset += '\''.len_utf8();
            while let Some(ch) = self.peek_char() {
                self.offset += ch.len_utf8();
                if ch == '\'' {
                    return parse_value(&self.input[start..self.offset]);
                }
            }
            return Err(());
        }
        let start = self.offset;
        self.consume_int_literal()?;
        parse_value(&self.input[start..self.offset])
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
    encode_row_body(&mut record, table, values);
    record
}

fn encode_secondary_metadata_record(state: &SecondaryIndexState, table_name: &str) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(SECONDARY_METADATA_RECORD);
    write_u64(&mut record, state.build_id);
    write_string_u16(&mut record, &state.name);
    write_string_u16(&mut record, table_name);
    write_u16(&mut record, state.indexed_column as u16);
    record.push(state.tie_break_mode.to_byte());
    record
}

fn encode_secondary_entry_record(entry: &EmbeddedIndexEntry) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(SECONDARY_ENTRY_RECORD);
    encode_embedded_entry(&mut record, entry);
    record
}

fn encode_indexed_row_record(
    table: &str,
    values: &[Value],
    entries: &[EmbeddedIndexEntry],
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(INDEXED_ROW_RECORD);
    encode_row_body(&mut record, table, values);
    write_u16(&mut record, entries.len() as u16);
    for entry in entries {
        encode_embedded_entry(&mut record, entry);
    }
    record
}

fn encode_update_row_record(
    table: &str,
    row_position: usize,
    values: &[Value],
    entries: &[EmbeddedIndexEntry],
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(UPDATE_ROW_RECORD);
    write_u64(&mut record, row_position as u64);
    encode_row_body(&mut record, table, values);
    write_u16(&mut record, entries.len() as u16);
    for entry in entries {
        encode_embedded_entry(&mut record, entry);
    }
    record
}

fn encode_delete_row_record(table: &str, row_position: usize) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(DELETE_ROW_RECORD);
    write_string_u16(&mut record, table);
    write_u64(&mut record, row_position as u64);
    record
}

fn encode_row_body(record: &mut Vec<u8>, table: &str, values: &[Value]) {
    write_string_u16(record, table);
    write_u16(record, values.len() as u16);
    for value in values {
        record.push(value.column_type().to_byte());
        match value {
            Value::Int(value) => write_string_u32(record, &value.to_string()),
            Value::Text(value) => write_string_u32(record, value),
        }
    }
}

fn encode_embedded_entry(record: &mut Vec<u8>, entry: &EmbeddedIndexEntry) {
    write_u64(record, entry.build_id);
    write_string_u16(record, &entry.index_name);
    write_i64(record, entry.indexed_key);
    write_i64(record, entry.tie_break);
    write_u64(record, entry.row_position);
}

fn decode_record(record: &[u8]) -> Result<LogicalRecord, SqlError> {
    if !record.starts_with(SQL_RECORD_PREFIX) {
        return Err(SqlError::InvalidStorageRecord);
    }
    let mut reader = RecordReader::new(&record[SQL_RECORD_PREFIX.len()..]);
    let kind = reader.read_u8()?;
    let decoded = match kind {
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
            LogicalRecord::Catalog { table, columns }
        }
        ROW_RECORD => {
            let (table, values) = reader.read_row_body()?;
            LogicalRecord::Row { table, values }
        }
        SECONDARY_ENTRY_RECORD => {
            let entry = reader.read_embedded_entry()?;
            LogicalRecord::SecondaryIndexEntry {
                build_id: entry.build_id,
                index_name: entry.index_name,
                indexed_key: entry.indexed_key,
                tie_break: entry.tie_break,
                row_position: entry.row_position,
            }
        }
        SECONDARY_METADATA_RECORD => {
            let build_id = reader.read_u64()?;
            let index_name = reader.read_string_u16()?;
            let table_name = reader.read_string_u16()?;
            let indexed_column = reader.read_u16()?;
            let tie_break_mode =
                TieBreakMode::from_byte(reader.read_u8()?).ok_or(SqlError::InvalidStorageRecord)?;
            LogicalRecord::SecondaryIndexMetadata {
                build_id,
                index_name,
                table_name,
                indexed_column,
                tie_break_mode,
            }
        }
        INDEXED_ROW_RECORD => {
            let (table, values) = reader.read_row_body()?;
            let entry_count = reader.read_u16()? as usize;
            let mut entries = Vec::with_capacity(entry_count);
            for _ in 0..entry_count {
                entries.push(reader.read_embedded_entry()?);
            }
            LogicalRecord::IndexedRow {
                table,
                values,
                entries,
            }
        }
        UPDATE_ROW_RECORD => {
            let row_position = reader.read_u64()?;
            let (table, values) = reader.read_row_body()?;
            let entry_count = reader.read_u16()? as usize;
            let mut entries = Vec::with_capacity(entry_count);
            for _ in 0..entry_count {
                entries.push(reader.read_embedded_entry()?);
            }
            LogicalRecord::UpdateRow {
                table,
                row_position,
                values,
                entries,
            }
        }
        DELETE_ROW_RECORD => {
            let table = reader.read_string_u16()?;
            let row_position = reader.read_u64()?;
            LogicalRecord::DeleteRow {
                table,
                row_position,
            }
        }
        _ => return Err(SqlError::InvalidStorageRecord),
    };
    reader.finish()?;
    Ok(decoded)
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_i64(output: &mut Vec<u8>, value: i64) {
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

    fn read_u64(&mut self) -> Result<u64, SqlError> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| SqlError::InvalidStorageRecord)?,
        ))
    }

    fn read_i64(&mut self) -> Result<i64, SqlError> {
        let bytes = self.read_exact(8)?;
        Ok(i64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| SqlError::InvalidStorageRecord)?,
        ))
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

    fn read_row_body(&mut self) -> Result<(String, Vec<Value>), SqlError> {
        let table = self.read_string_u16()?;
        let value_count = self.read_u16()? as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            let value_type =
                ColumnType::from_byte(self.read_u8()?).ok_or(SqlError::InvalidStorageRecord)?;
            let raw_value = self.read_string_u32()?;
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
        Ok((table, values))
    }

    fn read_embedded_entry(&mut self) -> Result<EmbeddedIndexEntry, SqlError> {
        let build_id = self.read_u64()?;
        let index_name = self.read_string_u16()?;
        let indexed_key = self.read_i64()?;
        let tie_break = self.read_i64()?;
        let row_position = self.read_u64()?;
        Ok(EmbeddedIndexEntry {
            build_id,
            index_name_key: normalize_identifier(&index_name),
            index_name,
            indexed_key,
            tie_break,
            row_position,
        })
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
