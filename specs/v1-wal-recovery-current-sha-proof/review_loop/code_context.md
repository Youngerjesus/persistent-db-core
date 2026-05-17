# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 33b480cac6cf9d505a86eda4c149a4471454f11d
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T16:00:19.144902+00:00
- selected_files: tests/wal_recovery.rs, docs/file_format.md, docs/cli_contract.md, src/main.rs, src/lib.rs, src/storage.rs, specs/v1-transaction-wal-recovery/spec.md, specs/v1-transaction-wal-recovery/contracts.md, specs/v1-transaction-wal-recovery/qa_mapping.md, specs/v1-transaction-wal-recovery/impl_review.md

## Omitted Reasons
- /db-exec-wal-recovery: context path escapes repo root: /db-exec-wal-recovery
- /wal-replay-current-sha-proof: context path escapes repo root: /wal-replay-current-sha-proof
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- flow:/wal-replay-current-sha-proof: not_git_tracked
- history/progress: not_git_tracked
- route:/db-exec-wal-recovery: not_git_tracked
- sidecar/recovery: not_git_tracked
- specs/v1-transaction-wal-recovery/code_review.md: context_char_limit
- specs/v1-transaction-wal-recovery/final_review.md: context_char_limit

## File Excerpts

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
    let mut frame = Vec::with_capacity(WAL_HEADER_LEN + payload.len());
    frame.extend_from_slice(WAL_MAGIC);
    frame.extend_from_slice(&WAL_VERSION.to_le_bytes());
    frame.extend_from_slice(&frame_id.to_le_bytes());
    frame.extend_from_slice(&record_count_before.to_le_bytes());
    frame.push(WAL_STATE_COMMITTED);
    frame.push(WAL_PAYLOAD_KIND_PAGE_APPEND);
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(payload);

    let checksum = checksum(&frame);
    frame[32..36].copy_from_slice(&checksum.to_le_bytes());
    frame
}

fn checksum(frame_with_zero_checksum: &[u8]) -> u32 {
    frame_with_zero_checksum
        .iter()
        .enumerate()
        .filter(|(index, _)| !(32..36).contains(index))
        .fold(0u32, |sum, (_, byte)| sum.wrapping_add(*byte as u32))
}

#[test]
fn committed_wal_replay_survives_reopen_via_cli() {
    let path = temp_db_path("committed_wal_replay_survives_reopen_via_cli");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TE
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

### src/lib.rs
- excerpt_chars: 45
- clipped: false

```text
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

### specs/v1-transaction-wal-recovery/spec.md
- excerpt_chars: 4000
- clipped: true

```text
# 트랜잭션 WAL 복구 최소 증거 추가

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-17-23-45-17
- Task ID: task-2026-05-17-23-45-17-v1-transaction-wal-recovery
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 트랜잭션 WAL 복구 최소 증거 추가
- Artifact: v1-transaction-wal-recovery

## 목표
- 현재 `persistent-db-core`는 durable storage, SQL 실행, index 증거는 갖췄지만, committed mutation만 재시작 후 살아남고 rollback 또는 partial mutation은 사라진다는 복구 계약 증거가 없습니다.

## 지금 해야 하는 이유
- Root Progress Projection에서 CLI, storage, SQL, indexes gate는 `projected_complete`이고, 남은 recovery 계열 중 crash matrix는 WAL 복구 의미가 먼저 고정되어야 검증 가능합니다. 따라서 WAL replay의 최소 commit/rollback slice가 다음 의존성 병목입니다.

## 기대 산출물 변화
- Rust CLI `db exec` 경로에서 committed mutation이 재시작 후 조회되는 최소 WAL replay 증거를 추가합니다.
- 공개 CLI에 transaction rollback 명령이 없으면 rollback 또는 incomplete WAL entry 부재 증거는 storage-level deterministic fixture test로 고정합니다.
- WAL 파일 위치, record framing, replay 의미, rollback/incomplete 처리, 기존 database file 호환 동작을 `docs/file_format.md`에 compatibility note로 문서화합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/
- docs/file_format.md
- docs/cli_contract.md
- route:db-exec
- flow:wal-replay-recovery

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 8adb253bac616445f6e389e29048e4296f0ce85b
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, src/storage.rs, tests/cli_contract.rs, tests/page_storage.rs, tests/primary_index.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `gate-v1-transactions-wal-recovery` 상태가 `open`이고 missing requirement가 `req-v1-wal-recovery-proof`입니다.
- Current Plan: `gap-v1-transaction-wal-recovery`는 high priority이며 next candidate hint가 commit/rollback WAL replay입니다.
- Current Artifact: `gate-v1-transactions-wal-recovery`는 WAL replay tests, commit/rollback tests, recovery transcript, file-state evidence를 요구합니다.
- Managed repo progress: SQL schema/execute path와 primary-key indexed lookup/ordered scan proof가 이미 존재해 recovery layer를 검증할 실행 baseline이 있습니다.
- Queue Snapshot: 현재 active 또는 reserved task가 없습니다.
- persistent-db-core_worktree/main/src/sql.rs
- persistent-db-core_worktree/main/src/storage.rs
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/tests/sql_exec.rs
- persistent-db-core_worktree/main/tests/primary_index.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/sql_subset.md
- persistent-db-core_worktree/main/docs/file_format.md
- persistent-db-core_worktree/main/work_queue/progress.md
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- sql::execute
- Database::from_records
- Statement::CreateTable
- Statement::Insert
- Statement::SelectAll
- Statement::SelectPrimaryKey
- PageStore::open
- PageStore::append_record
- PageStore::read_records
- PrimaryIndex
- autopilot/project_manager/tasks/tasks.json#task-2026-05-15-16-06-54-v1-bootstrap-cli-contract:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-16-13-58-47-v1-page-storage-record-format:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-19-38-21-v1-sql-parser-schema-exec:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-22-43-31-v1-primary-btree-index:SUCCESS
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/spec.md
- autopilot/project_manager/spe
```

### specs/v1-transaction-wal-recovery/contracts.md
- excerpt_chars: 2825
- clipped: false

```text
# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: 트랜잭션 WAL 복구 최소 증거 추가에 대한 closure.
- 생성 대상 테스트: `tests/wal_recovery.rs`.
- 생성 대상 문서: WAL compatibility note가 포함된 `docs/file_format.md`.
- 조건부 생성 대상 문서: public CLI output, exit code, stderr contract가 변경될 때의 `docs/cli_contract.md`.
- 생성 대상 verification output: `cargo test`, `cargo test --test wal_recovery`, `./scripts/verify`, canonical CLI smoke command output.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `cargo test`가 통과합니다.
- `./scripts/verify`가 통과합니다.
- `cargo test --test wal_recovery`가 통과합니다.
- CLI-visible Scenario A는 temp database path에서 `CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');` 실행 후 새 `db exec` process로 `SELECT * FROM users;`를 수행합니다.
- Scenario A의 create/insert command는 exit code `0`, stdout `""`, stderr `""`를 증거로 남깁니다.
- Scenario A의 select command는 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 증거로 남깁니다.
- Scenario B는 committed row `1|ada`와 rollback 또는 incomplete row `9|ghost`를 포함한 deterministic WAL fixture를 reopen/replay한 뒤 `9|ghost`가 조회 결과나 storage row set에 없음을 증명합니다.
- Scenario B가 CLI fixture로 검증되면 select command는 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n`를 증거로 남깁니다.
- Scenario B가 storage-level fixture로만 검증되면 test는 row set이 `[(1, "ada")]`와 동등함을 assert하고, CLI fixture가 아닌 이유를 test 이름 또는 주석에 남깁니다.
- `docs/file_format.md`는 WAL 파일명 또는 위치, record layout 또는 framing, replay 순서, committed/rollback/incomplete entry 처리, 기존 database 파일을 열 때의 기대 동작을 모두 포함합니다.
- `docs/cli_contract.md`는 public CLI behavior가 달라질 때만 갱신하며, 달라지지 않으면 final report에 변경 없음 이유를 남깁니다.
- final report 또는 phase result는 `tests/wal_recovery.rs`, `docs/file_format.md`, 조건부 `docs/cli_contract.md`, verification command output, WAL file-state evidence summary를 연결합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
```

### specs/v1-transaction-wal-recovery/qa_mapping.md
- excerpt_chars: 4000
- clipped: true

```text
# QA Mapping: v1-transaction-wal-recovery

## Scope
- Phase: QA Prep Execution
- Current run id source: task metadata `active_run_id=qa_prep_exec_fresh_20260518_000602_594462_3e6c0674`
- Frozen inputs: `spec.md`, `contracts.md`, `tasks.md`, `plan.md`, `design.md`, `research.md`
- Implementation files intentionally untouched in this phase.

## Scenario Expansion Lens
| Scenario ID | Path | Pressure Applied | QA Consequence |
|---|---|---|---|
| WAL-A | committed mutation reopen | Separate writer and reader `db exec` processes; exact stdout/stderr/exit assertions | `committed_wal_replay_survives_reopen_via_cli` must assert create/insert exits `0` with empty streams, WAL sidecar exists, and reopen select returns exactly `id\|name\n1\|ada\n2\|bea\n`. |
| WAL-B | incomplete or rollback mutation absence | Public CLI has no rollback/incomplete command; fixture authors WAL bytes directly after CLI-created catalog | `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` must write one committed `1|ada` frame plus incomplete `9|ghost` frame, then assert CLI select shows only `1|ada`. |
| WAL-C | duplicate replay / already done | Retained WAL frames are replayed across repeated opens | Scenario B repeats the select after first replay; result must stay exactly one row and must not duplicate `1|ada`. |
| WAL-D | partial state / incomplete tail | Short payload after a syntactically valid frame prefix | Scenario B truncates the ghost frame to force incomplete-tail handling and absence of `9|ghost`. |
| WAL-E | dependency failure / semantic rejection | Failed SQL statements must not become committed WAL records | Covered by T3 mapping to existing `mid_command_failure_keeps_prior_successes_and_skips_later_statements` and primary-key duplicate tests; implementation may add/adjust only if routing changes create a gap. |
| WAL-F | old file compatibility | Existing database files without WAL must remain openable | Covered by existing SQL restart and storage tests plus T4 doc review; no new CLI output contract expected. |
| WAL-G | permission / trust boundary | WAL sidecar is local filesystem state derived from user-supplied DB path | Use temp directories only; no network, daemon, or cross-process concurrency assumptions in tests. |
| WAL-H | retry/re-entry | Fresh verification pass must not reuse previous launch evidence as current proof | Provenance Contract below requires current-run regeneration for implementation evidence. |

## Provenance Contract
This task is evidence-heavy because acceptance depends on generated command evidence, a task-scoped WAL fixture, smoke output, WAL file-state evidence, and current-run provenance beyond static checks.

- Evidence root: current implementation or verification phase report under the active run directory, plus inline command evidence referenced from that report.
- Required artifact list: `tests/wal_recovery.rs`, `docs/file_format.md`, optional `docs/cli_contract.md` only if public CLI behavior changes, verification command output for `cargo test`, `cargo test --test wal_recovery`, `./scripts/verify`, canonical CLI smoke output, and WAL file-state evidence summary.
- Scenario/evidence IDs: WAL-A committed CLI reopen, WAL-B incomplete ghost absence, WAL-C retained-frame idempotence, WAL-F old-file compatibility, DOC-WAL file-format compatibility note, VERIFY-BASE required command suite, SMOKE-WAL canonical smoke commands.
- Current-run id source: task metadata `active_run_id` and the scheduler result path for the current run.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: smoke outputs, WAL file-state summaries, and verification logs from previous runs are invalid as current proof even if command text matches.
- Writer/validator separation expectation: implementation
```

### specs/v1-transaction-wal-recovery/impl_review.md
- excerpt_chars: 310
- clipped: true

```text
# Implementation Verification Review: v1-transaction-wal-recovery

Verdict: PASS

## Scope

Verified the current worktree implementation for task `task-2026-05-17-23-45-17-v1-transaction-wal-recovery` against `spec.md`, `contracts.md`, `tasks.md`, `qa_mapping.md`, and the latest `impl_brake_review.md`.

Imple
```
