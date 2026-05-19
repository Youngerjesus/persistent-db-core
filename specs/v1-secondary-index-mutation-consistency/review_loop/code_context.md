# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 12731d4424d199c40d05d611c077a9be30b96ece
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-18T18:17:33.802374+00:00
- selected_files: src/main.rs, src/lib.rs, src/storage.rs, tests/secondary_index.rs, tests/sql_exec.rs, tests/db_check.rs, docs/cli_contract.md, work_queue/progress.md, docs/history_archives/history.md, src/sql.rs

## Omitted Reasons
- UPDATE/DELETE: not_git_tracked
- active/reserved: not_git_tracked
- autopilot/project_manager/specs/v1-secondary-index-range-scan/spec.md: not_git_tracked
- autopilot/project_manager/tasks/task_status_events.jsonl: not_git_tracked
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- docs/sql_subset.md: context_char_limit
- equality/range: not_git_tracked
- primary/secondary: not_git_tracked
- reopen/WAL: not_git_tracked
- src/check.rs: context_char_limit
- ssot/current-artifact.md: not_git_tracked
- update/delete: not_git_tracked

## File Excerpts

### src/main.rs
- excerpt_chars: 3474
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
            eprintln!("hint: supported SQL subset is documented in docs/sql_subset.md.");
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

### tests/secondary_index.rs
- excerpt_chars: 4000
- clipped: true

```text
use persistent_db_core::index::SecondaryIndex;
use persistent_db_core::sql::{plan_query_path_for_test, QueryPath};
use persistent_db_core::storage::PageStore;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const INVALID_SQL_STORAGE_STDERR: &str = "error: invalid SQL storage record: unknown record tag\nhint: run against a database file created by this SQL contract or restore from a valid backup.\n";
const UNSUPPORTED_SQL_HINT: &str =
    "hint: supported SQL subset is documented in docs/sql_subset.md.\n";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;

#[derive(Clone, Copy)]
struct EmbeddedIndexEntry<'a> {
    build_id: u64,
    index_name: &'a str,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedSecondaryMetadata {
    build_id: u64,
    index_name: String,
    table_name: String,
    indexed_column: u16,
    tie_break_mode: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedSecondaryEntry {
    build_id: u64,
    index_name: String,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
}

type FixtureBuilder = Box<dyn Fn(&Path)>;

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_secondary_index_{}_{}_{}",
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

fn check_db(path: &Path) -> Output {
    db(&["check", path.to_str().expect("temp path should be UTF-8")])
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.wal",
        path.to_str().expect("temp path should be UTF-8")
    ))
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

fn assert_check_ok(path: &Path) {
    let output = check_db(path);
    assert_eq!(
        Some(0),
        output.status.code(),
        "db check should pass; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("ok: db check passed\n", stdout(&output));
    assert_eq!("", stderr(&output));
}

fn assert_check_secondary_index_failure(path: &Path) {
    let output = check_db(path);
    assert_eq!(
        Some(1),
        output.status.code(),
        "db check should fail; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert_eq!("error: db check failed: secondary index\n", stderr(&output
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

### docs/cli_contract.md
- excerpt_chars: 4000
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

`CREATE INDEX <index> ON <table>(<integer_column>);` creates a durable
secondary index over an existing `INT` column. Successful `CREATE TABLE`,
`INSERT`, and `CREATE INDEX` mutations exit `0`, write empty stdout/stderr
unless a later `SELECT` writes rows, and are durable across later `db exec`
process starts for the same database path. WAL sidecar details are documented
in `docs/file_format.md`; they do not change successful `db exec` stdout,
stderr, or exit codes.

After `CREATE INDEX`, `SELECT * FROM <table> WHERE <indexed_column> = <int>;`
uses the matching secondary index. `SELECT * FROM <table> WHERE
<indexed_column> BETWEEN <low_int> AND <high_int>;` uses an inclusive bounded
secondary-index range scan. Results are ordered by secondary key ascending and
then by deterministic tie-break ascending. For tables with a primary key, the
tie-break is the primary-key value. For tables without a primary key, the
tie-break is durable row insertion order. If `low_int > high_int`, the range
query still uses the secondary range path and prints the header only.

Before
```

### work_queue/progress.md
- excerpt_chars: 4000
- clipped: true

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, disk-backed secondary-index equality/range proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, `db check` invariant validation for existing database files, SQLite-backed differential/property evidence for the supported SQL subset, and repo-local benchmark/acceptance documentation evidence. The next smallest implementation handoff should target mutation-maintained secondary-index behavior or a narrower acceptance blocker on top of the SQL execution, recovery, check, differential, and benchmark baselines.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | verification_ready | Primary-key tables rebuild an in-memory B-tree index from durable row records, support exact lookup, scan in primary-key order, and preserve row-only table compatibility. |
| gap-v1-secondary-index-range-scan | verification_ready | `CREATE INDEX` creates durable secondary `INT` indexes with indexed equality/range query paths, deterministic ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant evidence. |
| gap-v1-transaction-wal-recovery | verification_ready | Current-SHA WAL sidecar replay proof covers committed mutation survival, rolled-back/uncommitted frame absence, incomplete-tail exclusion, and retained sidecar state after reopen. |
| gap-v1-deterministic-crash-matrix | verification_ready | Deterministic crash matrix covers pre-WAL append, partial WAL frame, uncommitted frame, committed replay idempotence, interrupted recovery retry, and corrupt tail cleanup evidence. |
| gap-v1-differential-property-tests | verification_ready | SQLite-backed deterministic differential/property tests cover supported SQL subset generation, duplicate-key errors, missing lookups, ordered scans, seed replay, and failure artifact reporting. |
| gap-v1-db-check-invariants | verification_ready | `db check <path>` validates existing page records, SQL catalog/row invariants, primary-key rebuildability, WAL sidecar ordering, missing paths, and directory-path open/read errors. |
| gap-v1-bench-docs-acceptance | verification_ready | `scripts/verify_bench_acceptance` records deterministic lower-bound JSON evidence, and `docs/v1_acceptance.md` maps launch gates to current evidence and explicit blockers. |

## Recent Entries

- 2026-05-19: Added disk-backed secondary-index support for `CREATE INDEX`, indexed equality and inclusive `BETWEEN` range scans, deterministic key/tie-break ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant validation.
- 2026-05-18: Added repo-local benchmark acceptance evidence with `scripts/verify_bench_acceptance`, documented lower-bound policy in `docs/benchmarks.md`, and mapped V1 launch gates in `docs/v1_acceptance.md` with explicit missing-evidence blockers.
- 2026-05-18: Added SQLite-backed deterministic differential/property evidence for the supported SQL subset, including seed replay, duplicate-key and missing-lookup coverage, ordered scan comparison, local failure artifact reporting, and `scripts/verify_differential_property`.
- 2026-05-18: Added `db check <path>` invariant validation with exact success/failure CLI contracts, deterministic corrupted fixtures for storage, catalog/record, primary
```

### docs/history_archives/history.md
- excerpt_chars: 2178
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
- Added deterministic WAL crash matrix coverage for pre-append, partial-frame, uncommitted, committed replay, interrupted recovery, and corrupt-tail boundaries.
- Added the `db check` invariant milestone: existing database files can now be validated for page readability, SQL catalog/row consistency, primary-index rebuildability, WAL replay consistency, and stable open/read error behavior.
- Added SQLite-backed differential/property coverage for the supported SQL subset with deterministic seed replay, duplicate-key and missing-lookup checks, ordered scan comparison, and task-specific verification.
- Added the V1 benchmark and acceptance documentation milestone: `scripts/verify_bench_acceptance` records deterministic lower-bound evidence, and the V1 acceptance guide maps launch gates to evidence and explicit blockers.

## 2026-05-19

- Added the secondary-index milestone: `CREATE INDEX` now persists `INT` secondary indexes, uses indexed equality and inclusive bounded range paths with deterministic ordering, survives reopen/WAL replay, and is covered by `db check` invariant validation.
```

### src/sql.rs
- excerpt_chars: 288
- clipped: true

```text
use crate::index::{PrimaryIndex, SecondaryIndex};
use crate::storage::{PageStore, StorageError};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const CATALOG_RECORD: u8 = b'C';
const ROW_RECORD: u8 = b'R';
const SECOND
```
