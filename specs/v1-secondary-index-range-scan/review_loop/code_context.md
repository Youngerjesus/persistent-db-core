# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: ddff99d9f8f89ed69aca56a436693ccd5870b4cb
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-18T16:30:29.265394+00:00
- selected_files: src/check.rs, src/index.rs, src/lib.rs, src/main.rs, src/sql.rs, src/storage.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md, work_queue/progress.md, AGENTS.md

## Omitted Reasons
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- autopilot/ssot/current-plan.md: not_git_tracked
- benchmark/docs: not_git_tracked
- differential/property: not_git_tracked
- docs/history_archives/history.md: context_char_limit
- equality/range: not_git_tracked
- metadata/storage: not_git_tracked
- persistent-db-core_worktree/main/docs/cli_contract.md: not_git_tracked
- persistent-db-core_worktree/main/docs/v1_acceptance.md: not_git_tracked
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/analysis_report.md: not_git_tracked
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/final_review.md: not_git_tracked
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/spec.md: not_git_tracked
- persistent-db-core_worktree/main/src/index.rs: not_git_tracked
- persistent-db-core_worktree/main/src/sql.rs: not_git_tracked
- persistent-db-core_worktree/main/work_queue/progress.md: not_git_tracked
- ssot/current-artifact.md: not_git_tracked
- ssot/current-plan.md: not_git_tracked
- tests/secondary_index.rs: not_git_tracked

## File Excerpts

### src/check.rs
- excerpt_chars: 1343
- clipped: false

```text
use crate::sql;
use crate::storage::{self, StorageError};
use std::path::{Path, PathBuf};

pub const SUCCESS_OUTPUT: &str = "ok: db check passed\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    OpenRead { path: PathBuf },
    Invariant { label: &'static str },
}

pub fn check_database(path: impl AsRef<Path>) -> Result<(), CheckError> {
    let path = path.as_ref();
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) | Err(_) => {
            return Err(CheckError::OpenRead {
                path: path.to_path_buf(),
            });
        }
    }

    let snapshot = storage::read_records_for_check(path).map_err(|error| match error {
        StorageError::Io => CheckError::OpenRead {
            path: path.to_path_buf(),
        },
        _ => CheckError::Invariant {
            label: "storage record readability",
        },
    })?;

    sql::validate_records_for_check(snapshot.records)
        .map_err(|label| CheckError::Invariant { label })?;

    storage::validate_wal_for_check(path, snapshot.record_count).map_err(|error| match error {
        StorageError::Io => CheckError::OpenRead {
            path: path.to_path_buf(),
        },
        _ => CheckError::Invariant {
            label: "wal replay consistency",
        },
    })?;

    Ok(())
}
```

### src/index.rs
- excerpt_chars: 981
- clipped: false

```text
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimaryIndex {
    positions_by_key: BTreeMap<i64, usize>,
}

impl PrimaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: i64, row_position: usize) -> Result<(), DuplicatePrimaryKey> {
        if self.positions_by_key.contains_key(&key) {
            return Err(DuplicatePrimaryKey);
        }
        self.positions_by_key.insert(key, row_position);
        Ok(())
    }

    pub fn get(&self, key: i64) -> Option<usize> {
        self.positions_by_key.get(&key).copied()
    }

    pub fn ordered_positions(&self) -> Vec<usize> {
        self.positions_by_key.values().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.positions_by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions_by_key.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicatePrimaryKey;
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

### src/storage.rs
- excerpt_chars: 4000
- clipped: true

```text
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const PAGE_SIZE: usize = 4096;
pub const FILE_MAGIC: &[u8; 8] = b"PDBV1\0\0\0";
pub const DATA_PAGE_MAGIC: &[u8; 4] = b"PDPG";

const FORMAT_VERSION: u16 = 1;
const FILE_HEADER_PAGE_COUNT_OFFSET: usize = 16;
const DATA_PAGE_HEADER_SIZE: usize = 16;
const DATA_PAGE_USED_OFFSET: usize = 8;
const DATA_PAGE_RECORD_COUNT_OFFSET: usize = 10;
const RECORD_LENGTH_SIZE: usize = 4;
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;
const WAL_CHECKSUM_OFFSET: usize = 32;
const WAL_CHECKSUM_END: usize = 36;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    TruncatedFile,
    TruncatedPage,
    InvalidMagic,
    UnsupportedVersion,
    RecordTooLarge,
    CorruptRecordLength,
    Io,
}

impl From<std::io::Error> for StorageError {
    fn from(_: std::io::Error) -> Self {
        StorageError::Io
    }
}

#[derive(Debug)]
pub struct PageStore {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckStorageSnapshot {
    pub records: Vec<Vec<u8>>,
    pub record_count: u64,
}

impl PageStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            let mut file = OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)?;
            write_file_header(&mut file, 1)?;
            file.flush()?;
        } else {
            validate_file(&path)?;
        }

        replay_wal(&path)?;

        Ok(Self { path })
    }

    pub fn append_record(&mut self, payload: &[u8]) -> Result<(), StorageError> {
        if payload.len() > max_record_payload_len() {
            return Err(StorageError::RecordTooLarge);
        }

        let record_count_before = total_record_count(&self.path)?;
        append_wal_frame(&wal_path(&self.path), record_count_before, payload)?;
        append_record_to_file(&self.path, payload)?;
        Ok(())
    }

    pub fn read_records(&mut self) -> Result<Vec<Vec<u8>>, StorageError> {
        validate_file(&self.path)?;

        let mut file = OpenOptions::new().read(true).open(&self.path)?;
        let page_count = read_page_count(&mut file)?;
        let mut records = Vec::new();

        for page_index in 1..page_count {
            let page = read_page(&mut file, page_index)?;
            read_data_page_records(&page, &mut records)?;
        }

        Ok(records)
    }
}

pub fn read_records_for_check(
    path: impl AsRef<Path>,
) -> Result<CheckStorageSnapshot, StorageError> {
    let path = path.as_ref();
    validate_file(path)?;

    let mut file = OpenOptions::new().read(true).open(path)?;
    let page_count = read_page_count(&mut file)?;
    let mut records = Vec::new();

    for page_index in 1..page_count {
        let page = read_page(&mut file, page_index)?;
        read_data_page_records(&page, &mut records)?;
    }

    let record_count = u64::try_from(records.len()).map_err(|_| StorageError::Io)?;
    Ok(CheckStorageSnapshot {
        records,
        record_count,
    })
}

pub fn validate_wal_for_check(
    path: impl AsRef<Path>,
    durable_record_count: u64,
) -> Result<(), StorageError> {
    let wal_path = wal_path(path.as_ref());
    if !wal_path.exists() {
        return Ok(());
    }

    let bytes = std::fs::read(&wal_path)?;
    let mut offset = 0usize;
    let mut virtual_record_count = durable_record_count;
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < WAL_HEADER_LEN {
            return Ok(());
        }

        let header = &bytes[offset..offset + WAL_HEADER_LEN];
        if &header[0..8] != WAL_MAGIC {
            return
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

### docs/file_format.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 Page File Format

## Page Size And Numbering

V1 page files use fixed 4096-byte pages. Page `0` is the file header page. Data pages start at page `1` and continue in append order. The file length must always be an exact multiple of 4096 bytes.

## File Header Page

All multi-byte integer fields are little-endian.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 8 | File magic `PDBV1\0\0\0` |
| 8 | 2 | Format version, currently `1` |
| 10 | 2 | Page size as `u16`, currently `4096` |
| 12 | 4 | Page size as `u32`, currently `4096` |
| 16 | 8 | Total page count, including the file header page |
| 24 | 4072 | Reserved, zero-filled in new files |

## Data Page Layout

Each data page stores opaque byte records in append order.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | Data page magic `PDPG` |
| 4 | 2 | Page format version, currently `1` |
| 6 | 2 | Data page header size, currently `16` |
| 8 | 2 | Used byte offset from the start of the page |
| 10 | 2 | Record count in this page |
| 12 | 4 | Reserved, zero-filled in new pages |
| 16 | variable | Record stream |

## Record Encoding

Records are encoded as `u32 little-endian length` followed by exactly that many payload bytes. Payloads are opaque bytes; empty payloads, UTF-8 text, and arbitrary binary bytes are all valid. A single record must fit in one data page after the 16-byte page header and 4-byte length prefix. Overflow pages are not part of V1.

## SQL Logical Records

The SQL executor does not change the page header, data page header, or opaque
record framing. SQL catalog and row data live above `PageStore` as opaque record
payloads documented in `docs/sql_subset.md`.

SQL payloads are UTF-8 compatible and start with the prefix `PDBSQL1\0`.
The byte after the prefix is the SQL logical record kind: `C` for catalog and
`R` for row. Catalog records include table name and ordered column metadata.
Row records include table name and ordered typed values. Arbitrary records
without the SQL prefix are valid page-storage payloads, but they are not valid
SQL database records and are rejected by `db exec` with the documented invalid
SQL storage record error.

Catalog records may include an optional primary-key extension after the ordered
column metadata: byte tag `P` followed by a little-endian `u16` zero-based
column index. The referenced column must be `INT`. Catalog records without this
extension are valid row-only SQL catalogs and load as tables without a primary
key.

Primary indexes are not persisted as separate page records, sidecar files, or
background metadata. `db exec` rebuilds the in-memory primary index from durable
row records when the database is opened. A primary-key table with duplicate
persisted key values is treated as corrupt SQL logical data and fails with the
existing invalid SQL storage record error. Because no separate index metadata is
stored, missing index metadata is not a V1 failure mode.

## Validation Errors

Opening or reading a file validates the header and every declared data page. Short files return a truncated-file error. Non-page-aligned files or missing declared pages return a truncated-page error. Invalid file or data page magic returns an invalid-magic error. Unsupported format versions return an unsupported-version error. Record lengths that exceed the page used bytes or page capacity return a corrupt-record-length error. Oversized appends return a record-too-large error.

## Compatibility Note

V1 is pre-launch and does not guarantee backward compatibility for existing user data. After this page and record format is introduced, format changes must not be made implicitly: the documentation and deterministic tests must be updated together with any intentional format change. SQL logical-record evolution must preserve the lower-level page framing unless a future task explicitly changes the page format contract. The primary-key catalog extension is optional so existing row-only SQL database files remain readable as non-primary
```

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

### work_queue/progress.md
- excerpt_chars: 4000
- clipped: true

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, `db check` invariant validation for existing database files, SQLite-backed differential/property evidence for the supported SQL subset, and repo-local benchmark/acceptance documentation evidence. The next smallest implementation handoff should target secondary indexes or a narrower acceptance blocker on top of the SQL execution, recovery, check, differential, and benchmark baselines.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | verification_ready | Primary-key tables rebuild an in-memory B-tree index from durable row records, support exact lookup, scan in primary-key order, and preserve row-only table compatibility. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | verification_ready | Current-SHA WAL sidecar replay proof covers committed mutation survival, rolled-back/uncommitted frame absence, incomplete-tail exclusion, and retained sidecar state after reopen. |
| gap-v1-deterministic-crash-matrix | verification_ready | Deterministic crash matrix covers pre-WAL append, partial WAL frame, uncommitted frame, committed replay idempotence, interrupted recovery retry, and corrupt tail cleanup evidence. |
| gap-v1-differential-property-tests | verification_ready | SQLite-backed deterministic differential/property tests cover supported SQL subset generation, duplicate-key errors, missing lookups, ordered scans, seed replay, and failure artifact reporting. |
| gap-v1-db-check-invariants | verification_ready | `db check <path>` validates existing page records, SQL catalog/row invariants, primary-key rebuildability, WAL sidecar ordering, missing paths, and directory-path open/read errors. |
| gap-v1-bench-docs-acceptance | verification_ready | `scripts/verify_bench_acceptance` records deterministic lower-bound JSON evidence, and `docs/v1_acceptance.md` maps launch gates to current evidence and explicit blockers. |

## Recent Entries

- 2026-05-18: Added repo-local benchmark acceptance evidence with `scripts/verify_bench_acceptance`, documented lower-bound policy in `docs/benchmarks.md`, and mapped V1 launch gates in `docs/v1_acceptance.md` with explicit missing-evidence blockers.
- 2026-05-18: Added SQLite-backed deterministic differential/property evidence for the supported SQL subset, including seed replay, duplicate-key and missing-lookup coverage, ordered scan comparison, local failure artifact reporting, and `scripts/verify_differential_property`.
- 2026-05-18: Added `db check <path>` invariant validation with exact success/failure CLI contracts, deterministic corrupted fixtures for storage, catalog/record, primary-index, and WAL replay consistency failures, plus focused `cargo test --test db_check` coverage.
- 2026-05-18: Added deterministic WAL crash matrix evidence for six recovery boundaries, including partial/corrupt tails, uncommitted frame exclusion, committed replay idempotence, and interrupted recovery retry with `scripts/verify_crash_matrix`.
- 2026-05-18: Reverified WAL recovery at current SHA with focused WAL tests, baseline `scripts/verify`, CLI reopen smoke, retained WAL sidecar byte evidence, and explic
```

### AGENTS.md
- excerpt_chars: 44
- clipped: true

```text
# AGENTS (persistent-db-core)

Primary audie
```
