# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 2a330122edc833f214ab1727e677afbea0236f56
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T19:51:15.011145+00:00
- selected_files: docs/cli_contract.md, Cargo.toml, scripts/verify, src/sql.rs, src/main.rs, src/lib.rs, tests/sql_exec.rs, tests/primary_index.rs, tests/wal_recovery.rs, tests/crash_matrix.rs, tests/db_check.rs

## Omitted Reasons
- autopilot/project_manager/specs: not_git_tracked
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- differential/property: not_git_tracked
- docs/sql_subset.md: context_char_limit
- docs/testing.md: not_git_tracked
- persistent-db-core_worktree/main/specs: not_git_tracked
- scripts/verify_differential_property: not_git_tracked
- seed/failing-case: not_git_tracked
- tests/differential_property.rs: not_git_tracked

## File Excerpts

### docs/cli_contract.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path, primary-key lookup path, and database
check path.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
db exec <path> <sql>
db check <path>
```

`db --help` and `db help` exit with code `0`, write no stderr, and write
identical help text to stdout.

`db exec <path> <sql>` executes one SQL argument against the database file at
`<path>`. The file is created if it does not exist. SQL from stdin, interactive
shell input, and multiple SQL argv fragments are not supported.

`db check <path>` validates an existing database file and WAL sidecar without
repairing or mutating them. The file must already exist and must be a regular
file.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
Reserved future commands:
  open <path>
  bench <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: help printed successfully, `db exec` completed successfully, or
  `db check` passed.
- `1`: storage, SQL logical-record data, or `db check` invariants are invalid
  for this contract. `db check` open/read failures also use exit code `1`.
- `2`: the first argument was unsupported, or no supported command was provided.
  SQL syntax, unsupported SQL, and SQL semantic errors also use exit code `2`.

## Unsupported Input

Unsupported arguments and subcommands exit with code `2`, write no stdout, and write this stderr format:

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

`<token>` is the first unsupported token supplied by the user. For example, `db --unknown` reports `--unknown`, `db open demo.db` reports `open`, and `db exec demo.db` reports `exec`.

## SQL Execution

Successful `db exec` writes no stderr. It writes stdout only for supported
`SELECT *` statements. Each result set prints the stored column header followed
by rows, with `|` as the field delimiter and `\n` after every output line.
Tables without a primary key scan in successful `INSERT` append order. Tables
declared with one `INT PRIMARY KEY` scan in ascending primary-key order.
`SELECT * FROM <table> WHERE <primary_key> = <int>;` performs exact primary-key
lookup and prints only the matching row, or only the header when the key is
missing. Multiple `SELECT` statements repeat the header with no blank line,
separator, or count line.

Successful `CREATE TABLE` and `INSERT` mutations are durable across later
`db exec` process starts for the same database path. WAL sidecar details are
documented in `docs/file_format.md`; they do not change successful `db exec`
stdout, stderr, or exit codes.

The supported SQL subset is documented in `docs/sql_subset.md`.

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ..., SELECT * FROM ... WHERE <primary_key> = <int>;
```

Malformed SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: malformed SQL statement: CREATE TABLE users id INT);
hint: terminate each statement with ';' and use the documented SQL subset.
```

SQL semantic errors exit `2`, write empty stdout, and use the exact strings below:

```text
error: SQL semantic error: table already exists: users
hint: use a new table name for CREATE TABLE in this database.
```

Case-variant duplicate table input reports the new input spelling, such as
`Users`.
```

### Cargo.toml
- excerpt_chars: 131
- clipped: false

```text
[package]
name = "persistent-db-core"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "db"
path = "src/main.rs"

[dependencies]
```

### scripts/verify
- excerpt_chars: 217
- clipped: false

```text
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin db -- --help
```

### src/sql.rs
- excerpt_chars: 4000
- clipped: true

```text
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
                    if let Some(primary_key_column) = existing.prim
```

### src/main.rs
- excerpt_chars: 3572
- clipped: false

```text
use std::env;
use std::process;

use persistent_db_core::check::{self, CheckError};
use persistent_db_core::sql::{self, SqlError};

const HELP: &str = "\
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
Reserved future commands:
  open <path>
  bench <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [arg] if arg == "--help" || arg == "help" => {
            print!("{HELP}");
        }
        [command, path, sql_text] if command == "exec" => match sql::execute(path, sql_text) {
            Ok(stdout) => {
                print!("{stdout}");
            }
            Err(error) => exit_with_sql_error(error),
        },
        [command, path] if command == "check" => match check::check_database(path) {
            Ok(()) => {
                print!("{}", check::SUCCESS_OUTPUT);
            }
            Err(error) => exit_with_check_error(error),
        },
        [token, ..] => {
            eprintln!("error: unsupported argument or command: {token}");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
        [] => {
            eprintln!("error: unsupported argument or command: <none>");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
    }
}

fn exit_with_check_error(error: CheckError) -> ! {
    match error {
        CheckError::OpenRead { path } => {
            eprintln!(
                "error: could not open or read database path: {}",
                path.display()
            );
            process::exit(1);
        }
        CheckError::Invariant { label } => {
            eprintln!("error: db check failed: {label}");
            process::exit(1);
        }
    }
}

fn exit_with_sql_error(error: SqlError) -> ! {
    match error {
        SqlError::Unsupported(statement) => {
            eprintln!("error: unsupported SQL statement: {statement}");
            eprintln!(
                "hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ..., SELECT * FROM ... WHERE <primary_key> = <int>;"
            );
            process::exit(2);
        }
        SqlError::Malformed(statement) => {
            eprintln!("error: malformed SQL statement: {statement}");
            eprintln!("hint: terminate each statement with ';' and use the documented SQL subset.");
            process::exit(2);
        }
        SqlError::Semantic { message, hint } => {
            eprintln!("error: SQL semantic error: {message}");
            eprintln!("hint: {hint}");
            process::exit(2);
        }
        SqlError::InvalidStorageRecord => {
            eprintln!("error: invalid SQL storage record: unknown record tag");
            eprintln!(
                "hint: run against a database file created by this SQL contract or restore from a valid backup."
            );
            process::exit(1);
        }
        SqlError::Storage(error) => {
            eprintln!("error: storage error: {error:?}");
            eprintln!("hint: database file must use the documented V1 page format.");
            process::exit(1);
        }
    }
}
```

### src/lib.rs
- excerpt_chars: 60
- clipped: false

```text
pub mod check;
pub mod index;
pub mod sql;
pub mod storage;
```

### tests/sql_exec.rs
- excerpt_chars: 4000
- clipped: true

```text
use persistent_db_core::storage::PageStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const INVALID_SQL_STORAGE_STDERR: &str = "error: invalid SQL storage record: unknown record tag\nhint: run against a database file created by this SQL contract or restore from a valid backup.\n";

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_sql_exec_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn assert_exec(path: &Path, sql: &str, code: i32, expected_stdout: &str, expected_stderr: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(code),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(expected_stdout, stdout(&output));
    assert_eq!(expected_stderr, stderr(&output));
}

fn assert_rejected_without_stdout(path: &Path, sql: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(2),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert!(
        stderr(&output).starts_with("error: "),
        "stderr should contain a deterministic user-facing error, got {:?}",
        stderr(&output)
    );
}

fn append_fixture_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture database should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn catalog_record(table: &str, columns: &[(&str, u8)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'C');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(columns.len() as u16).to_le_bytes());
    for (name, column_type) in columns {
        write_string_u16(&mut record, name);
        record.push(*column_type);
    }
    record
}

fn row_record(table: &str, values: &[(u8, &str)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'R');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(values.len() as u16).to_le_bytes());
    for (value_type, value) in values {
        record.push(*value_type);
        record.extend_from_slice(&(value.len() as u32).to_le_bytes());
        record.extend_from_slice(value.as_bytes());
    }
    record
}

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

#[test]
fn happy_path_creates_inserts_and_selects_rows_in_insert_order() {
    let path = temp_db_path("happy_path_creates_inserts_and_selects_rows_in_insert_order");

    assert_exec(
        &path,
```

### tests/primary_index.rs
- excerpt_chars: 4000
- clipped: true

```text
use persistent_db_core::index::PrimaryIndex;
use persistent_db_core::storage::PageStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const INVALID_SQL_STORAGE_STDERR: &str = "error: invalid SQL storage record: unknown record tag\nhint: run against a database file created by this SQL contract or restore from a valid backup.\n";

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_primary_index_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn assert_exec(path: &Path, sql: &str, code: i32, expected_stdout: &str, expected_stderr: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(code),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(expected_stdout, stdout(&output));
    assert_eq!(expected_stderr, stderr(&output));
}

fn append_fixture_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture database should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn catalog_record(table: &str, columns: &[(&str, u8)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'C');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(columns.len() as u16).to_le_bytes());
    for (name, column_type) in columns {
        write_string_u16(&mut record, name);
        record.push(*column_type);
    }
    record
}

fn row_record(table: &str, values: &[(u8, &str)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'R');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(values.len() as u16).to_le_bytes());
    for (value_type, value) in values {
        record.push(*value_type);
        record.extend_from_slice(&(value.len() as u32).to_le_bytes());
        record.extend_from_slice(value.as_bytes());
    }
    record
}

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

#[test]
fn primary_index_insert_find_missing_duplicate_and_len() {
    let mut index = PrimaryIndex::new();

    assert!(index.is_empty());
    assert_eq!(None, index.get(9));

    index.insert(2, 0).expect("first key should insert");
    index.insert(1, 1).expect("second key should insert");

    assert_eq!(2, index.len());
    assert_eq!(Some(0), index.get(2));
    assert_eq!(Some(1), index.get(1));
    assert_eq!(None, index.get(3));
    assert!(index.insert(2, 99).is_err());
    assert_eq!(Some(0), index.get(2), "duplicate insert must not overwrite");
}

#[test]
fn primary_index_ordered_positions_are_ascending_by_key()
```

### tests/wal_recovery.rs
- excerpt_chars: 4000
- clipped: true

```text
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_wal_recovery_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn assert_exec(path: &Path, sql: &str, code: i32, expected_stdout: &str, expected_stderr: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(code),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(expected_stdout, stdout(&output));
    assert_eq!(expected_stderr, stderr(&output));
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.wal",
        path.to_str().expect("temp path should be UTF-8")
    ))
}

fn row_record(table: &str, values: &[(u8, &str)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'R');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(values.len() as u16).to_le_bytes());
    for (value_type, value) in values {
        record.push(*value_type);
        record.extend_from_slice(&(value.len() as u32).to_le_bytes());
        record.extend_from_slice(value.as_bytes());
    }
    record
}

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

fn committed_wal_frame(frame_id: u64, record_count_before: u64, payload: &[u8]) -> Vec<u8> {
    wal_frame(
        frame_id,
        record_count_before,
        WAL_STATE_COMMITTED,
        WAL_PAYLOAD_KIND_PAGE_APPEND,
        payload,
    )
}

fn rolled_back_wal_frame(frame_id: u64, record_count_before: u64, payload: &[u8]) -> Vec<u8> {
    wal_frame(
        frame_id,
        record_count_before,
        WAL_STATE_ROLLED_BACK,
        WAL_PAYLOAD_KIND_PAGE_APPEND,
        payload,
    )
}

fn wal_frame(
    frame_id: u64,
    record_count_before: u64,
    state: u8,
    payload_kind: u8,
    payload: &[u8],
) -> Vec<u8> {
    let mut frame = Vec::with_capacity(WAL_HEADER_LEN + payload.len());
    frame.extend_from_slice(WAL_MAGIC);
    frame.extend_from_slice(&WAL_VERSION.to_le_bytes());
    frame.extend_from_slice(&frame_id.to_le_bytes());
    frame.extend_from_slice(&record_count_before.to_le_bytes());
    frame.push(state);
    frame.push(payload_kind);
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(payload);

    let checksum = c
```

### tests/crash_matrix.rs
- excerpt_chars: 4000
- clipped: true

```text
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, Once};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;
const REPORT_DIR: &str = "target/crash_matrix";

static REPORT_INIT: Once = Once::new();
static REPORT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
struct CrashCase {
    case_id: &'static str,
    crash_point: &'static str,
    evidence_id: &'static str,
    test_name: &'static str,
    expected_rows: &'static str,
    wal_assertion: &'static str,
}

const CM_001: CrashCase = CrashCase {
    case_id: "CM-001",
    crash_point: "pre-wal-append",
    evidence_id: "crash-matrix-case-CM-001",
    test_name: "cm_001_pre_wal_append_seed_only_visible",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "WAL sidecar absent or empty; file header/version and seed data unchanged",
};

const CM_002: CrashCase = CrashCase {
    case_id: "CM-002",
    crash_point: "partial-wal-frame",
    evidence_id: "crash-matrix-case-CM-002",
    test_name: "cm_002_partial_wal_frame_is_ignored",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "incomplete WAL header or payload tail is ignored/truncated without panic",
};

const CM_003: CrashCase = CrashCase {
    case_id: "CM-003",
    crash_point: "wal-frame-without-commit-marker",
    evidence_id: "crash-matrix-case-CM-003",
    test_name: "cm_003_wal_frame_without_commit_marker_is_not_visible",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "commit marker absent maps to WAL_STATE_ROLLED_BACK 0x02 and is not replayed",
};

const CM_004: CrashCase = CrashCase {
    case_id: "CM-004",
    crash_point: "committed-wal-before-data-apply",
    evidence_id: "crash-matrix-case-CM-004",
    test_name: "cm_004_committed_wal_before_data_apply_is_idempotent",
    expected_rows: "id|name\n1|seed\n2|committed_wal\n",
    wal_assertion: "committed WAL replay is idempotent across first and second reopen",
};

const CM_005: CrashCase = CrashCase {
    case_id: "CM-005",
    crash_point: "recovery-interrupted-after-first-apply",
    evidence_id: "crash-matrix-case-CM-005",
    test_name: "cm_005_recovery_interrupted_after_first_apply_replays_remaining_once",
    expected_rows: "id|name\n1|seed\n2|recover_a\n3|recover_b\n",
    wal_assertion: "interrupted recovery re-entry applies every committed frame exactly once",
};

const CM_006: CrashCase = CrashCase {
    case_id: "CM-006",
    crash_point: "corrupt-tail-after-committed-frame",
    evidence_id: "crash-matrix-case-CM-006",
    test_name: "cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix",
    expected_rows: "id|name\n1|seed\n2|committed_before_tail\n",
    wal_assertion: "committed WAL prefix is replayed and incomplete/invalid-length tail is ignored without CLI output change",
};

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn db_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_db"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_crash_matrix_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as
```

### tests/db_check.rs
- excerpt_chars: 2020
- clipped: true

```text
use persistent_db_core::storage::PageStore;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PAGE_SIZE: usize = 4096;
const DATA_PAGE_HEADER_SIZE: u64 = 16;
const RECORD_LENGTH_SIZE: usize = 4;
const MAX_RECORD_PAYLOAD_LEN: usize =
    PAGE_SIZE - DATA_PAGE_HEADER_SIZE as usize - RECORD_LENGTH_SIZE;
const FIRST_RECORD_LENGTH_OFFSET: u64 = PAGE_SIZE as u64 + DATA_PAGE_HEADER_SIZE;
const FIRST_RECORD_PAYLOAD_OFFSET: u64 = FIRST_RECORD_LENGTH_OFFSET + 4;
const FIRST_SQL_KIND_OFFSET: u64 = FIRST_RECORD_PAYLOAD_OFFSET + SQL_RECORD_PREFIX.len() as u64;
const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_db_check_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(
```
