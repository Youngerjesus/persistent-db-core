# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-20T10:57:09.675867+00:00
- selected_files: tests/primary_index.rs, tests/sql_exec.rs, docs/v1_acceptance.md, scripts/verify, specs/v1-primary-btree-index/final_review.md, src/index.rs, src/sql.rs, docs/sql_subset.md, docs/file_format.md, docs/cli_contract.md

## Omitted Reasons
- /scripts/verify: context path escapes repo root: /scripts/verify
- active/reserved: not_git_tracked
- digest/current: not_git_tracked
- digest/current-SHA: not_git_tracked
- evidence/review: not_git_tracked
- scripts/verify_primary_index_acceptance: not_git_tracked
- specs/v1-primary-btree-index/code_review.md: context_char_limit
- specs/v1-primary-btree-index/impl_review.md: context_char_limit
- specs/v1-primary-index-current-artifact-evidence-refresh: not_git_tracked
- specs/v1-primary-index-current-artifact-evidence-refresh/**: not_git_tracked

## File Excerpts

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

### docs/v1_acceptance.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 Acceptance Guide

Evidence id: `evidence-v1-acceptance-docs`

Gate source at task handoff: `autopilot/ssot/current-artifact.md`, specifically the Launch Gate Evidence Contract and Evidence Requirements sections. This guide maps that source to current repo evidence without treating progress projection as proof.

## Gate Evidence Map

| Gate id | Requirement id | Evidence path | Verification command or manual review evidence | Current status |
| --- | --- | --- | --- | --- |
| `gate-v1-cli-smoke` | `req-v1-cli-help-smoke` | `docs/cli_contract.md`; `src/main.rs`; `tests/cli_contract.rs` | `scripts/verify`; `cargo run --bin db -- --help`; `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-cli-smoke` | `req-v1-cli-dispatch-tests` | `tests/cli_contract.rs` | `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-page-storage-restart` | `src/storage.rs`; `tests/page_storage.rs` | `cargo test --test page_storage`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `REQ-6-store-data-in-a-disk-ad3ffc4e` | `tests/page_storage.rs`; `docs/file_format.md`; `scripts/verify_page_storage_acceptance` | `cargo test --test page_storage`; `scripts/verify_page_storage_acceptance`; included in `scripts/verify`; manual review of page header/data page byte inspection | `verified_current_run` |
| `gate-v1-disk-page-storage` | `REQ-6-data-must-survive-process-restart-0471a233` | `tests/page_storage.rs`; `docs/file_format.md`; `scripts/verify_page_storage_acceptance` | `cargo test --test page_storage`; `scripts/verify_page_storage_acceptance`; included in `scripts/verify`; deterministic same-path drop/reopen test evidence | `verified_current_run` |
| `gate-v1-disk-page-storage` | `FAIL-6-reject-memory-only-dump-at-fd82a296` | `tests/page_storage.rs`; `docs/file_format.md`; `scripts/verify_page_storage_acceptance` | `cargo test --test page_storage`; `scripts/verify_page_storage_acceptance`; included in `scripts/verify`; live-store page file inspection before drop | `verified_current_run` |
| `gate-v1-disk-page-storage` | `FAIL-6-reject-whole-database-file-rewrite-bebf73bb` | `tests/page_storage.rs`; `src/storage.rs`; `docs/file_format.md`; `scripts/verify_page_storage_acceptance` | `cargo test --test page_storage`; `scripts/verify_page_storage_acceptance`; included in `scripts/verify`; bounded active-page mutation test plus implementation-level write-range audit and source review of page-write helpers | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-record-format-doc` | `docs/file_format.md` | Manual review of documented page, SQL logical record, and WAL sidecar compatibility notes | `verified_current_run` |
| `gate-v1-sql-schema-exec` | `req-v1-sql-exec-examples` | `docs/sql_subset.md`; `tests/sql_exec.rs` | `cargo test --test sql_exec`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-primary-index-proof` | `tests/primary_index.rs`; `src/index.rs`; `docs/sql_subset.md` | `cargo test --test primary_index`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-secondary-index-proof` | `tests/secondary_index.rs`; `src/sql.rs`; `src/index.rs`; `docs/cli_contract.md`; `docs/file_format.md` | `cargo test --test secondary_index -- --nocapture`; included in `scripts/verify`; manual review of persisted `E`/`X`/`I` record docs and `db check` invariant coverage | `verified_current_run` |
| `gate-v1-transactions-wal-recovery` | `req-v1-wal-recovery-proof` | `tests/wal_recovery.rs`; `docs/file_format.md` | `cargo test --test wal_recovery`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-crash-testing` | `req-v1-crash-matrix-output` | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md`; `target/crash_matrix/` when generated | `scripts/verify_crash_matrix` when crash-matrix evidence is required; crash tests are also covered by `scripts/verify` if
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

### specs/v1-primary-btree-index/final_review.md
- excerpt_chars: 1664
- clipped: false

```text
Verdict: PASS

## Scope
- Phase: Final Execution.
- Task: `task-2026-05-17-22-43-31-v1-primary-btree-index`.
- Reviewed implementation, tests, durable docs, progress/history records, and prior PASS reports for primary-key indexed lookup and ordered scan.
- Non-visual CLI/storage task: browser, DOM, screenshot, and UX design-review evidence were not required or used.

## Closure Checks
- Implementation exists for `PrimaryIndex`, primary-key catalog metadata, rebuild from durable row records, duplicate-key rejection, exact lookup, and key-ordered scans.
- Existing row-only SQL catalog records remain compatible as non-primary-key tables.
- Durable docs describe grammar, output behavior, no separate persisted index metadata, rebuild-on-open model, invalid SQL storage record handling for corrupt persisted rows, and no missing-index-metadata failure mode.
- Finish documentation sync updated `work_queue/progress.md` and `docs/history_archives/history.md`.
- Component memory files were not changed because no `docs/**/memory.md` files exist in this repo.

## Open Items
- None.

## Verification Evidence
- `cargo test --test primary_index`: PASS, 7 passed.
- `cargo test --test sql_exec primary_key`: PASS, 11 passed, 17 filtered out.
- `./scripts/verify`: PASS, including `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, doc tests, and `db --help` smoke.

## Remote State
- Pending finish commit, push, PR creation, and merge at the time this final-family SSOT was written.

## Next Action
- Commit, push, open PR, merge, and write scheduler final verification manifest/result.

## Updated At
- 2026-05-17T23:36:15+0900
```

### src/index.rs
- excerpt_chars: 2579
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

    pub fn remove(&mut self, key: i64) -> Option<usize> {
        self.positions_by_key.remove(&key)
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecondaryIndex {
    positions_by_key_and_tie_break: BTreeMap<(i64, i64), usize>,
}

impl SecondaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        key: i64,
        tie_break: i64,
        row_position: usize,
    ) -> Result<(), DuplicateSecondaryIndexEntry> {
        let entry_key = (key, tie_break);
        if self.positions_by_key_and_tie_break.contains_key(&entry_key) {
            return Err(DuplicateSecondaryIndexEntry);
        }
        self.positions_by_key_and_tie_break
            .insert(entry_key, row_position);
        Ok(())
    }

    pub fn equality_positions(&self, key: i64) -> Vec<usize> {
        self.positions_by_key_and_tie_break
            .range((key, i64::MIN)..=(key, i64::MAX))
            .map(|(_, row_position)| *row_position)
            .collect()
    }

    pub fn range_positions(&self, low: i64, high: i64) -> Vec<usize> {
        if low > high {
            return Vec::new();
        }
        self.positions_by_key_and_tie_break
            .range((low, i64::MIN)..=(high, i64::MAX))
            .map(|(_, row_position)| *row_position)
            .collect()
    }

    pub fn remove(&mut self, key: i64, tie_break: i64) -> Option<usize> {
        self.positions_by_key_and_tie_break
            .remove(&(key, tie_break))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateSecondaryIndexEntry;
```

### src/sql.rs
- excerpt_chars: 4000
- clipped: true

```text
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
CREATE TABLE <table_name> (<column_name> INT|INTEGER|TEXT[, <column_name> INT|INTEGER|TEXT]*);
CREATE TABLE <table_name> (<column_name> INT|INTEGER PRIMARY KEY[, <column_name> INT|INTEGER|TEXT]*);
CREATE INDEX <index_name> ON <table_name>(<integer_column>);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
UPDATE <table_name> SET <non_primary_key_column> = <value> WHERE <primary_key_column> = <int_value>;
DELETE FROM <table_name> WHERE <primary_key_column> = <int_value>;
SELECT * FROM <table_name>;
SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;
SELECT * FROM <table_name> WHERE <indexed_int_column> = <int_value>;
SELECT * FROM <table_name> WHERE <indexed_int_column> BETWEEN <low_int> AND <high_int>;
```

Keywords compare ASCII case-insensitively. Identifiers must match
`[A-Za-z_][A-Za-z0-9_]*`. Table and column equality is ASCII
case-insensitive, while stored catalog spelling is preserved for headers and
errors. Types are `INT`, `INTEGER`, and `TEXT`. `INTEGER` is a spelling alias
for the existing `INT` type, not a separate affinity system. `INT` values are
signed 64-bit decimal integers. `TEXT` values are UTF-8 strings inside single quotes; escape
sequences, embedded single quotes, `|`, newline, and carriage return are not
supported.

This slice supports at most one `INT PRIMARY KEY` or `INTEGER PRIMARY KEY`
column per table. Secondary indexes are explicit and support only integer
columns. `TEXT PRIMARY KEY`,
multiple primary-key columns, non-indexed non-primary-key predicates, range
predicates before `CREATE INDEX`, and non-integer predicate values are rejected.

Projection, general `WHERE`, `ORDER BY`, `JOIN`, non-primary-key-targeted
mutations, primary-key updates, defaults, `NULL`, quoted identifiers, and
transactions are out of scope.

## Output

`SELECT * FROM <table_name>;` prints the catalog column order as a header. For
tables without a primary key, rows print in successful `INSERT` append order.
For tables with an `INT PRIMARY KEY` or `INTEGER PRIMARY KEY`, rows print in ascending primary-key order.
`SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;` prints
the header and the matching row, or only the header when the key is missing.
After `CREATE INDEX`, equality and `BETWEEN` predicates on the indexed `INT`
column use the secondary index. `BETWEEN` boundaries are inclusive. Secondary
results are ordered by secondary key ascending, then by primary-key value for
primary-key tables or durable row insertion order for tables without a primary
key. A range with `low > high` prints the header only through the secondary
range path.
Fields are delimited with `|`, and each output line ends with `\n`. Empty tables
print only the header. Multiple `SELECT` statements repeat headers without blank
lines, separators, or count lines.

`UPDATE` and `DELETE` are mutation statements and write no stdout on success.
They require an equality predicate on the table's integer primary-key column.
`UPDATE` may set one existing non-primary-key column to a value matching that
column's declared type. `DELETE` makes the matching row invisible to table
scans, primary-key lookup, and secondary-index equality/range scans. Missing
primary-key targets are successful no-ops.

If any statement in a command fails, command stdout is empty even if an earlier
statement produced a result set. This task does not provide command-level
atomicity: successful statements before the failure remain durable, the failing
statement appends no partial SQL record, and later statements are not executed.

## Error Contract

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset is documented in docs/sql_subset.md.
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
The byte after the prefix is the SQL logical record kind: `C` for catalog, `R`
for row, `E` for secondary-index backfill entry, `X` for committed
secondary-index metadata, and `I` for one atomic post-index row plus its
embedded secondary-index entries. `U` updates one existing row slot and `D`
tombstones one existing row slot. Catalog records include table name and ordered
column metadata. Row records include table name and ordered typed values.
Arbitrary records without the SQL prefix are valid page-storage payloads, but
they are not valid SQL database records and are rejected by `db exec` with the
documented invalid SQL storage record error.

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

Secondary indexes are persisted as append-only SQL logical records above the
same page framing. Existing no-index databases containing only `C` and `R`
records remain compatible: they reopen normally, and a later `CREATE INDEX`
backfills existing rows.

`CREATE INDEX` writes all backfill `E` records first, then writes the final `X`
metadata record as the commit marker. The `build_id` in `E` and `X` is the
durable SQL logical-record count before that `CREATE INDEX` appends anything.
An `E` record without a matching committed `X(build_id, index_name)` is an
orphan interrupted build entry; `db exec` and `db check` ignore it. Retrying the
same index name after an interrupted build writes a fresh build id and fresh
`E` records before the final `X`.

Committed secondary-index metadata record:

```text
PDBSQL1\0
X
u64 buil
```

### docs/cli_contract.md
- excerpt_chars: 1540
- clipped: true

```text
# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path, primary-key lookup path, secondary
index lookup/range path, and database check path.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
db exec <path> <sql>
db check <path>
db bench
```

`db --help` and `db help` exit with code `0`, write no stderr, and write
identical help text to stdout.

`db exec <path> <sql>` executes one SQL argument against the database file at
`<path>`. The file is created if it does not exist. SQL from stdin, interactive
shell input, and multiple SQL argv fragments are not supported.

`db check <path>` validates an existing database file and WAL sidecar without
repairing or mutating them. The file must already exist and must be a regular
file.

`db bench` runs the fixed Section 14 benchmark acceptance workload and writes
machine-readable evidence to
`target/bench_acceptance/section14-benchmark-acceptance.json`.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
  db bench
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
  bench       Run the fixed Section 14 benchmark acceptance workload.
Reserved future commands:
  open <path>
V1 scope:
  This build supports the CLI contract, page storage, and the d
```
