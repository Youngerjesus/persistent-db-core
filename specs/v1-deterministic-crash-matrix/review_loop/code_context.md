# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 358854464059ba41ed2f232cfc6ab17a9ce51dac
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T17:26:56.496581+00:00
- selected_files: src/index.rs, src/lib.rs, src/main.rs, src/sql.rs, src/storage.rs, docs/cli_contract.md, docs/file_format.md, docs/history_archives/history.md, docs/sql_subset.md, docs/v1_spec.md, AGENTS.md

## Omitted Reasons
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- autopilot/ssot/current-plan.md: not_git_tracked
- persistent-db-core_worktree/main/docs/file_format.md: not_git_tracked
- persistent-db-core_worktree/main/docs/v1_spec.md: not_git_tracked
- persistent-db-core_worktree/main/scripts/verify: not_git_tracked
- persistent-db-core_worktree/main/src/storage.rs: not_git_tracked
- persistent-db-core_worktree/main/tests/wal_recovery.rs: not_git_tracked
- persistent-db-core_worktree/main/work_queue/progress.md: not_git_tracked
- project_manager/specs/v1-transaction-wal-recovery/contracts.md: not_git_tracked
- project_manager/specs/v1-transaction-wal-recovery/spec.md: not_git_tracked
- project_manager/specs/v1-wal-recovery-current-sha-proof/contracts.md: not_git_tracked
- project_manager/specs/v1-wal-recovery-current-sha-proof/spec.md: not_git_tracked
- scripts/verify_crash_matrix: not_git_tracked
- test/runner: not_git_tracked
- tests/crash_matrix.rs: not_git_tracked
- tests/fixtures/crash_matrix: not_git_tracked
- work_queue/progress.md: context_char_limit
- write/WAL/commit/recovery: not_git_tracked
- write/WAL/replay: not_git_tracked

## File Excerpts

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
- excerpt_chars: 45
- clipped: false

```text
pub mod index;
pub mod sql;
pub mod storage;
```

### src/main.rs
- excerpt_chars: 2820
- clipped: false

```text
use std::env;
use std::process;

use persistent_db_core::sql::{self, SqlError};

const HELP: &str = "\
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
Reserved future commands:
  open <path>
  check <path>
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

fn append_record_to_file(path: &Path, payload: &[u8]) -> Result<(), StorageError> {
    validate_file(path)?;

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut page_count = read_page_count(&mut file)?;

    if page_count == 1 {
        append_empty_data_page(&mut file)?;
        page_count = 2;
        write_page_count(&mut file, page_count)?;
    }

    let record_size = RECORD_LENGTH_SIZE + payload.len();
    let mut page_index = page_count - 1;
    let mut page = read_page(&mut file, page_index)?;
    let mut used = data_page_used(&page)? as usize;

    if used + record_size > PAGE_SIZE {
        append_empty_data_page(&mut file)?;
        page_count += 1;
        page_index = page_count - 1;
        write_page_count(&mut file, page_count)?;
        page = empty_data_page();
        used = DATA_PAGE_HEADER_SIZE;
    }

    if used + record_size > PAGE_SIZE {
        return Err(StorageError::RecordTooLarge);
    }

    page[used..used + RECORD_LENGTH_SIZE].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    let payload_start = used + RECORD_LENGTH_SIZE;
    page[payload_start..payload_start + payload.len()].copy_from_slice(payload);

    let new_used = used + record_size;
    let new_count = data_page_record_count(&page)?
        .checked_add(1)
        .ok_or(StorageError::Io)?;
    page[DATA_PAGE_USED_OFFSET..DATA_PAGE_USED
```

### docs/cli_contract.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path and primary-key lookup path.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
db exec <path> <sql>
```

`db --help` and `db help` exit with code `0`, write no stderr, and write
identical help text to stdout.

`db exec <path> <sql>` executes one SQL argument against the database file at
`<path>`. The file is created if it does not exist. SQL from stdin, interactive
shell input, and multiple SQL argv fragments are not supported.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
Reserved future commands:
  open <path>
  check <path>
  bench <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: help printed successfully, or `db exec` completed successfully.
- `1`: storage or SQL logical-record data is invalid for this contract.
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
documented in `docs/file_format.md`; they do not add public CLI commands or
change successful stdout, stderr, or exit codes.

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

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT or SELECT.
```

```text
error: SQL semantic error: duplicate column: id
hint: column names in a table must be unique.
```

Case-variant duplicate column input reports the new input spelling, such as
`ID`.

`
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

### docs/history_archives/history.md
- excerpt_chars: 1078
- clipped: false

```text
# Persistent DB Core History

## 2026-05-15

- Created `persistent-db-core` as a V1 managed repo for CAO Autopilot.
- Initial product boundary is a Rust CLI binary named `db`.
- No V1 implementation gaps have verified completion evidence yet.

## 2026-05-17

- Added the minimal SQL schema/execute milestone: `db exec <path> <sql>` now supports the documented `CREATE TABLE`, `INSERT`, and `SELECT *` path with deterministic persistence and error-contract coverage.
- Added the primary-key index milestone: `db exec` now supports single `INT PRIMARY KEY` tables with duplicate-key rejection, exact lookup, key-ordered scans, and reopen-safe in-memory index rebuild from durable row records.

## 2026-05-18

- Added the minimal WAL recovery milestone: committed `db exec` mutations are replay-safe across reopen, incomplete trailing WAL entries are excluded, and the retained sidecar format is documented.
- Reverified the WAL recovery milestone at the current task SHA with separate committed, rolled-back/uncommitted, incomplete-tail, CLI smoke, and retained sidecar evidence.
```

### docs/sql_subset.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 SQL Subset

`db exec <path> <sql>` accepts one SQL argument. Statement delimiter is `;`,
and every statement must end with `;`. Multiple statements in one command run in
order.

## Supported Grammar

```text
CREATE TABLE <table_name> (<column_name> INT|TEXT[, <column_name> INT|TEXT]*);
CREATE TABLE <table_name> (<column_name> INT PRIMARY KEY[, <column_name> INT|TEXT]*);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
SELECT * FROM <table_name>;
SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;
```

Keywords compare ASCII case-insensitively. Identifiers must match
`[A-Za-z_][A-Za-z0-9_]*`. Table and column equality is ASCII
case-insensitive, while stored catalog spelling is preserved for headers and
errors. Types are `INT` and `TEXT`. `INT` values are signed 64-bit decimal
integers. `TEXT` values are UTF-8 strings inside single quotes; escape
sequences, embedded single quotes, `|`, newline, and carriage return are not
supported.

This slice supports at most one `INT PRIMARY KEY` column per table. `TEXT
PRIMARY KEY`, multiple primary-key columns, non-primary-key predicates, range
predicates, and non-integer predicate values are rejected.

Projection, general `WHERE`, `ORDER BY`, `JOIN`, `UPDATE`, `DELETE`, defaults,
`NULL`, quoted identifiers, and transactions are out of scope.

## Output

`SELECT * FROM <table_name>;` prints the catalog column order as a header. For
tables without a primary key, rows print in successful `INSERT` append order.
For tables with an `INT PRIMARY KEY`, rows print in ascending primary-key order.
`SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;` prints
the header and the matching row, or only the header when the key is missing.
Fields are delimited with `|`, and each output line ends with `\n`. Empty tables
print only the header. Multiple `SELECT` statements repeat headers without blank
lines, separators, or count lines.

If any statement in a command fails, command stdout is empty even if an earlier
statement produced a result set. This task does not provide command-level
atomicity: successful statements before the failure remain durable, the failing
statement appends no partial SQL record, and later statements are not executed.

## Error Contract

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

Semantic errors occur inside the supported grammar and exit `2` with empty
stdout:

```text
error: SQL semantic error: table already exists: users
hint: use a new table name for CREATE TABLE in this database.
```

Case-variant duplicate table input reports the new input spelling, such as
`Users`.

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT or SELECT.
```

```text
error: SQL semantic error: duplicate column: id
hint: column names in a table must be unique.
```

Case-variant duplicate column input reports the new input spelling, such as
`ID`.

```text
error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1
hint: INSERT values must match the table schema exactly.
```

```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

```text
error: SQL semantic error: duplicate primary key for table users: 2
hint: primary key values must be unique.
```

```text
error: SQL semantic error: primary key column must be INT: id
hint: this SQL slice supports one INT PRIMARY KEY column per table.
```

Invalid SQL logical records exit `1`, write empty stdout, and use th
```

### docs/v1_spec.md
- excerpt_chars: 4000
- clipped: true

```text
# Autopilot V1 Spec: Persistent DB Core

## 1. Summary

V1 is an implementation test for a small SQLite-like database core.

The system must implement a persistent, page-based, disk-backed database with ordered indexes, single-process transactions, WAL-based recovery, deterministic crash simulation, invariant checking, differential/property-based tests, and basic performance constraints.

V1 is not an in-memory toy database. It is a test of whether Autopilot can design, implement, debug, and verify a complex persistent stateful system end to end.

## 2. Capability Being Tested

Passing V1 demonstrates:

- complex persistent stateful system implementation
- disk-backed data structure implementation
- transaction atomicity
- crash and recovery reasoning
- table/index consistency maintenance
- invariant-driven validation
- differential and property-based testing
- performance awareness sufficient to reject toy implementations

## 3. Required SQL Subset

### 3.1 Required Statements

The implementation must support the following SQL forms:

```sql
CREATE TABLE table_name (
  id INTEGER PRIMARY KEY,
  col1 INTEGER,
  col2 TEXT
);

CREATE INDEX index_name ON table_name (col1);

INSERT INTO table_name (id, col1, col2) VALUES (1, 10, 'hello');

SELECT * FROM table_name WHERE id = 1;

SELECT col1, col2 FROM table_name WHERE col1 = 10;

UPDATE table_name SET col1 = 20 WHERE id = 1;

DELETE FROM table_name WHERE id = 1;

BEGIN;
COMMIT;
ROLLBACK;
```

The parser may support only this subset, but unsupported SQL must fail with a clear error rather than crashing.

### 3.2 Required Predicates

The following `WHERE` predicates must be supported for primary key and indexed integer columns:

```sql
WHERE column = value
WHERE column < value
WHERE column <= value
WHERE column > value
WHERE column >= value
WHERE column BETWEEN a AND b
```

### 3.3 Explicitly Excluded SQL Features

The following are out of scope for V1:

- `JOIN`
- `GROUP BY`
- `HAVING`
- aggregation
- subqueries
- foreign keys
- `NULL`
- floating-point types
- concurrent transactions

## 4. Data Types

The implementation must support:

- `INTEGER`: signed 64-bit integer
- `TEXT`: UTF-8 string, maximum 1024 bytes

`NULL` is not supported.

## 5. Error Behavior

The system must return clear errors for:

- duplicate primary key insert
- access to a missing table
- access to a missing column
- unsupported SQL
- syntax errors
- type errors
- malformed database or WAL metadata found during `db check` or `db recover`

Syntax errors and unsupported SQL must not crash the process.

## 6. Storage Requirements

The database must use disk-backed persistent storage.

Required properties:

- Data must be stored in a disk file.
- Storage must be page-based.
- Default page size must be 4096 bytes.
- Data must survive process restart.
- The implementation must not keep all data only in memory and dump it at the end.
- The implementation must not rewrite the entire database file on every operation.
- Reads and writes must operate through page-level storage abstractions.

The following internal details are intentionally left to Autopilot:

- page header format
- record layout
- free page management
- overflow page design
- buffer/cache design
- page allocation policy
- compaction strategy

## 7. Index Requirements

The database must use disk-backed ordered indexes.

Required properties:

- `INTEGER PRIMARY KEY` must be implemented as a disk-backed ordered index.
- The ordered index must be a B+Tree or an equivalent ordered page-based structure.
- `CREATE INDEX` must create a secondary index.
- Equality lookup and range scan must use indexes where applicable.
- Insert, update, and delete must keep table rows and indexes consistent.
- Indexes must survive process restart.

Required invariants:

- primary key uniqueness
- B+Tree ordering
- correct leaf scan order
- secondary index entry matches the referenced table row
- no dangling index pointer
- no visible row missing from the required index

## 8. T
```

### AGENTS.md
- excerpt_chars: 1076
- clipped: true

```text
# AGENTS (persistent-db-core)

Primary audience: coding agents and maintainers working inside this product repo.

## Product Direction

Build a small, deterministic Rust CLI database binary named `db`. V1 should grow toward durable single-process database behavior while keeping the documented CLI contract stable.

## Engineering Rules

- Keep changes scoped to the active task and its spec package.
- Prefer deterministic behavior, deterministic tests, and explicit persisted-data fixtures over implicit state.
- Treat persisted data compatibility, CLI output, exit codes, and documented error behavior as stable contracts once introduced.
- Make failure modes explicit. Avoid panics for user-facing CLI errors and persisted-data handling unless the invariant violation is unrecoverable programmer error.
- Do not broaden dependencies without a task-level reason. For V1, standard library implementations are preferred unless the spec explicitly calls for a crate.
- Do not add network services, background daemons, or remote-service requirements for V1.
- Keep CLI output s
```
