# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: a5d22e67f3f2feefae757ee12c25d6b88849a849
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T20:59:53.036846+00:00
- selected_files: docs/v1_spec.md, docs/cli_contract.md, tests/cli_contract.rs, tests/crash_matrix.rs, tests/db_check.rs, tests/differential_property.rs, tests/fixtures/crash_matrix/README.md, tests/page_storage.rs, tests/primary_index.rs

## Omitted Reasons
- AGENTS.md: context_char_limit
- autopilot/progress/launch-readiness.json: not_git_tracked
- autopilot/progress/launch-readiness.md: not_git_tracked
- autopilot/project_manager/specs: not_git_tracked
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- autopilot/ssot/current-plan.md: not_git_tracked
- benchmark/acceptance: not_git_tracked
- docs/benchmarks.md: not_git_tracked
- docs/v1_acceptance.md: not_git_tracked
- persistent-db-core_worktree/main/docs/cli_contract.md: not_git_tracked
- persistent-db-core_worktree/main/docs/v1_spec.md: not_git_tracked
- persistent-db-core_worktree/main/src/main.rs: not_git_tracked
- persistent-db-core_worktree/main/tests/cli_contract.rs: not_git_tracked
- persistent-db-core_worktree/main/work_queue/progress.md: not_git_tracked
- scripts/verify_bench_acceptance: not_git_tracked
- tests/sql_exec.rs: context_char_limit
- tests/wal_recovery.rs: context_char_limit

## File Excerpts

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

### tests/cli_contract.rs
- excerpt_chars: 3969
- clipped: false

```text
use std::process::{Command, Output};

const REQUIRED_HELP_LINES: &[&str] = &[
    "db - deterministic single-process V1 database CLI",
    "Usage:",
    "  db --help",
    "  db help",
    "  db exec <path> <sql>",
    "  db check <path>",
    "Supported commands:",
    "  help        Print this help text.",
    "  exec <path> <sql>",
    "  check <path>",
    "Reserved future commands:",
    "  open <path>",
    "  bench <path>",
    "V1 scope:",
    "  This build supports the CLI contract, page storage, and the documented minimal SQL subset.",
    "Non-goals:",
    "  No network server, multi-process concurrency, or distributed storage.",
];

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn assert_help_contract(output: &Output) {
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr={:?}",
        output.status.code(),
        stderr(output)
    );
    assert_eq!("", stderr(output), "help stderr must be empty");

    let out = stdout(output);
    let mut search_from = 0usize;
    for line in REQUIRED_HELP_LINES {
        let relative = out[search_from..].find(line).unwrap_or_else(|| {
            panic!("missing help line after byte {search_from}: {line:?}\nstdout:\n{out}")
        });
        search_from += relative + line.len();
    }
}

#[test]
fn help_flag_prints_required_contract() {
    let output = db(&["--help"]);

    assert_help_contract(&output);
}

#[test]
fn help_subcommand_matches_help_flag() {
    let help_flag = db(&["--help"]);
    let help_subcommand = db(&["help"]);

    assert_help_contract(&help_flag);
    assert_help_contract(&help_subcommand);
    assert_eq!(stdout(&help_flag), stdout(&help_subcommand));
}

#[test]
fn unsupported_argument_reports_first_token() {
    let output = db(&["--unknown"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: --unknown\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn reserved_future_command_remains_unsupported() {
    let output = db(&["open", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: open\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn bench_reserved_future_command_remains_unsupported() {
    let output = db(&["bench", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: bench\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn exec_requires_path_and_single_sql_argument() {
    let output = db(&["exec", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: exec\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn check_requires_path_argument() {
    let output = db(&["check"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: check\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}
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
- excerpt_chars: 4000
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
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn check_db(path: &Path) -> Output {
    db(&["check", path.to_str().expect("temp path should be UTF-8")])
}

fn assert_exec_ok(path: &Path, sql: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(0),
        output.status.code(),
        "exec failed; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert_eq!("", stderr(&output));
}

fn assert_check_ok(output: &Output) {
    assert_eq!(
        Some(0),
        output.status.code(),
        "valid db check should pass; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("ok: db check passed\n", stdout(output));
    assert_eq!("", stderr(output));
}

fn assert_check_failed(output: &Output, expected_label: &str) {
    assert_eq!(
        Some(1),
        output.status.code(),
        "db check should fail with exit 1; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("", stdout(output), "db check failure stdout must be empty");
    assert_eq!(
        format!("error: db check failed: {expected_label}\n"),
        stderr(output),
        "stderr should use exact check failure contract"
    );
}

fn assert_user_open_read_error(output: &Output, expected_path: &Path) {
    assert_eq!(
        Some(1),
        output.status.code(),
        "open/read error should fail with exit 1; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("", stdout(output), "open/read error stdout must be empty");
    assert_eq!(
        format!(
            "error: could not open or read database path: {}\n",
            expected_path.display()
        ),
        stderr(output),
```

### tests/differential_property.rs
- excerpt_chars: 4000
- clipped: true

```text
use rusqlite::{params, Connection, ErrorCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DEFAULT_SEEDS: &[u64] = &[1, 42, 0x5eed_2026];
const MIN_OPERATIONS_PER_SEED: usize = 100;
const MIN_SUCCESSFUL_ROWS_PER_SEED: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    CreateTable,
    Insert { id: i64, value: String },
    DuplicateInsert { id: i64, value: String },
    SelectAll,
    SelectById { id: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    id: i64,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Observation {
    Ok { rows: Vec<Row> },
    Err { kind: ErrorKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    DuplicatePrimaryKey,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mismatch {
    operation_index: usize,
    operation: Operation,
    expected: Observation,
    actual: Observation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MismatchSignature {
    operation_class: OperationClass,
    expected_kind: ObservationKind,
    actual_kind: ObservationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationClass {
    CreateTable,
    Insert,
    DuplicateInsert,
    SelectAll,
    SelectById,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObservationKind {
    OkRows,
    DuplicatePrimaryKeyError,
    OtherError,
}

impl Operation {
    fn class(&self) -> OperationClass {
        match self {
            Operation::CreateTable => OperationClass::CreateTable,
            Operation::Insert { .. } => OperationClass::Insert,
            Operation::DuplicateInsert { .. } => OperationClass::DuplicateInsert,
            Operation::SelectAll => OperationClass::SelectAll,
            Operation::SelectById { .. } => OperationClass::SelectById,
        }
    }
}

impl Observation {
    fn kind(&self) -> ObservationKind {
        match self {
            Observation::Ok { .. } => ObservationKind::OkRows,
            Observation::Err {
                kind: ErrorKind::DuplicatePrimaryKey,
            } => ObservationKind::DuplicatePrimaryKeyError,
            Observation::Err {
                kind: ErrorKind::Other,
            } => ObservationKind::OtherError,
        }
    }
}

impl Mismatch {
    fn signature(&self) -> MismatchSignature {
        MismatchSignature {
            operation_class: self.operation.class(),
            expected_kind: self.expected.kind(),
            actual_kind: self.actual.kind(),
        }
    }
}

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
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

fn temp_db_path(seed: u64, label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_differential_property_{seed}_{label}_{}_{}",
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

#[test]
fn deterministic_sql_subset_matches_sqlite_oracle() {
    let seed_override = std::env::var("PDB_DIFF_SEED")
        .ok()
        .map(|value| value.parse::<u64>().expect("PDB_DIFF_SEED must be u64"));
    l
```

### tests/fixtures/crash_matrix/README.md
- excerpt_chars: 1868
- clipped: false

```text
# Crash Matrix Fixture Manifest

This directory is the stable fixture identity for `tests/crash_matrix.rs`.
The initial QA prep scaffold names every required seed and case. Binary WAL
bytes may be generated by deterministic Rust helpers during implementation.

| Fixture or Case | Crash Point | Evidence ID | Expected Visible Rows | Byte Source |
| --- | --- | --- | --- | --- |
| `seed_committed_one` | baseline seed | n/a | `[(1, 'seed')]` | create `items (id INT PRIMARY KEY, name TEXT)` and insert `(1, 'seed')` |
| CM-001 | `pre-wal-append` | `crash-matrix-case-CM-001` | `[(1, 'seed')]` | no row-2 WAL/data mutation |
| CM-002 | `partial-wal-frame` | `crash-matrix-case-CM-002` | `[(1, 'seed')]` | generated incomplete WAL header or payload prefix |
| CM-003 | `wal-frame-without-commit-marker` | `crash-matrix-case-CM-003` | `[(1, 'seed')]` | generated complete WAL frame using state byte `0x02` (`WAL_STATE_ROLLED_BACK`) |
| CM-004 | `committed-wal-before-data-apply` | `crash-matrix-case-CM-004` | `[(1, 'seed'), (2, 'committed_wal')]` | generated committed WAL frame for row 2 without page-file apply |
| CM-005 | `recovery-interrupted-after-first-apply` | `crash-matrix-case-CM-005` | `[(1, 'seed'), (2, 'recover_a'), (3, 'recover_b')]` | seed-only durable data file plus committed WAL rows 2/3; test sets `PDB_CRASH_AFTER_WAL_REPLAY_APPLIES=1` to interrupt after row 2 replay, then reopens twice |
| CM-006 | `corrupt-tail-after-committed-frame` | `crash-matrix-case-CM-006` | `[(1, 'seed'), (2, 'committed_before_tail')]` | generated committed WAL frame for row 2 plus incomplete/invalid-length trailing fragment |

The reopen command field in generated evidence should use:

```text
db exec <db_path> "SELECT * FROM items;"
```

`ORDER BY` is intentionally not part of this fixture contract because the
current CLI SQL subset rejects general `ORDER BY`.
```

### tests/page_storage.rs
- excerpt_chars: 4000
- clipped: true

```text
use persistent_db_core::storage::{PageStore, StorageError};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const PAGE_SIZE: usize = 4096;
const FILE_MAGIC: &[u8; 8] = b"PDBV1\0\0\0";
const PAGE_MAGIC: &[u8; 4] = b"PDPG";
const FORMAT_VERSION_OFFSET: usize = 8;
const PAGE_COUNT_OFFSET: usize = 16;
const DATA_PAGE_HEADER_SIZE: usize = 16;
const FIRST_RECORD_LENGTH_OFFSET: u64 = (PAGE_SIZE + DATA_PAGE_HEADER_SIZE) as u64;

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_page_storage_{}_{}_{}",
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

fn assert_storage_error<T: std::fmt::Debug>(
    result: Result<T, StorageError>,
    expected: StorageError,
) {
    let actual = result.expect_err("operation should return a deterministic storage error");
    assert_eq!(expected, actual);
}

fn write_header(path: &PathBuf, version: u16, page_count: u64) {
    let mut page = vec![0u8; PAGE_SIZE];
    page[0..8].copy_from_slice(FILE_MAGIC);
    page[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&version.to_le_bytes());
    page[10..12].copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
    page[12..16].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    page[PAGE_COUNT_OFFSET..PAGE_COUNT_OFFSET + 8].copy_from_slice(&page_count.to_le_bytes());
    fs::write(path, page).expect("header fixture should be written");
}

fn write_header_and_data_page(path: &PathBuf) {
    write_header(path, 1, 2);

    let mut page = vec![0u8; PAGE_SIZE];
    page[0..4].copy_from_slice(PAGE_MAGIC);
    page[4..6].copy_from_slice(&1u16.to_le_bytes());
    page[6..8].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[8..10].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[10..12].copy_from_slice(&0u16.to_le_bytes());

    OpenOptions::new()
        .append(true)
        .open(path)
        .expect("fixture should open")
        .write_all(&page)
        .expect("data page fixture should be written");
}

#[test]
fn append_read_preserves_order_and_bytes() {
    let path = temp_db_path("append_read_preserves_order_and_bytes");

    let result = (|| {
        let mut store = PageStore::open(&path)?;
        store.append_record(b"alpha")?;
        store.append_record(b"beta")?;

        let records = store.read_records()?;
        assert_eq!(vec![b"alpha".to_vec(), b"beta".to_vec()], records);

        let bytes = fs::read(&path).expect("page file should be readable");
        assert_eq!(
            0,
            bytes.len() % PAGE_SIZE,
            "file length must be page aligned"
        );
        assert_eq!(FILE_MAGIC, &bytes[0..8]);
        assert_eq!(
            &1u16.to_le_bytes(),
            &bytes[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        );
        assert_eq!(PAGE_MAGIC, &bytes[PAGE_SIZE..PAGE_SIZE + 4]);
        assert_eq!(
            &5u32.to_le_bytes(),
            &bytes[FIRST_RECORD_LENGTH_OFFSET as usize..FIRST_RECORD_LENGTH_OFFSET as usize + 4]
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("append/read should succeed");
}

#[test]
fn reopen_reads_previously_appended_records() {
    let path = temp_db_path("reopen_reads_previously_appended_records");

    let result = (|| {
        {
            let mut store = PageStore::open(&path)?;
            store.append_record(b"alpha")?;
            store.append_record(b"beta")?;
        }
```

### tests/primary_index.rs
- excerpt_chars: 163
- clipped: true

```text
use persistent_db_core::index::PrimaryIndex;
use persistent_db_core::storage::PageStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, O
```
