# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 847a4bcdcc2c265452ddfdf01a23584301586728
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T13:47:43.075907+00:00
- selected_files: src/lib.rs, src/main.rs, src/storage.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md, AGENTS.md, work_queue/progress.md, docs/history_archives/history.md, .codex/agents/decision-brake-readiness-reviewer.toml, .codex/agents/project-reviewer.toml, .codex/agents/task-master.toml

## Omitted Reasons
- SQL/storage: not_git_tracked
- active/reserved: not_git_tracked
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md: not_git_tracked
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md: not_git_tracked
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md: not_git_tracked
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md: not_git_tracked
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/contracts.md: not_git_tracked
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/spec.md: not_git_tracked
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- autopilot/ssot/current-plan.md: not_git_tracked
- persistent-db-core_worktree/main/docs/cli_contract.md: not_git_tracked
- persistent-db-core_worktree/main/docs/file_format.md: not_git_tracked
- persistent-db-core_worktree/main/docs/sql_subset.md: not_git_tracked
- persistent-db-core_worktree/main/docs/v1_spec.md: not_git_tracked
- persistent-db-core_worktree/main/src/lib.rs: not_git_tracked
- persistent-db-core_worktree/main/src/main.rs: not_git_tracked
- persistent-db-core_worktree/main/src/sql.rs: not_git_tracked
- persistent-db-core_worktree/main/src/storage.rs: not_git_tracked
- persistent-db-core_worktree/main/tests/sql_exec.rs: not_git_tracked
- route:db-exec-primary-index: not_git_tracked
- schema/execute: not_git_tracked
- schema/insert/select: not_git_tracked
- src/index.rs: not_git_tracked
- tests/primary_index.rs: not_git_tracked

## File Excerpts

### src/lib.rs
- excerpt_chars: 30
- clipped: false

```text
pub mod sql;
pub mod storage;
```

### src/main.rs
- excerpt_chars: 2773
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
                "hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ...;"
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

        Ok(Self { path })
    }

    pub fn append_record(&mut self, payload: &[u8]) -> Result<(), StorageError> {
        if payload.len() > max_record_payload_len() {
            return Err(StorageError::RecordTooLarge);
        }

        validate_file(&self.path)?;

        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
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

        page[used..used + RECORD_LENGTH_SIZE]
            .copy_from_slice(&(payload.len() as u32).to_le_bytes());
        let payload_start = used + RECORD_LENGTH_SIZE;
        page[payload_start..payload_start + payload.len()].copy_from_slice(payload);

        let new_used = used + record_size;
        let new_count = data_page_record_count(&page)?
            .checked_add(1)
            .ok_or(StorageError::Io)?;
        page[DATA_PAGE_USED_OFFSET..DATA_PAGE_USED_OFFSET + 2]
            .copy_from_slice(&(new_used as u16).to_le_bytes());
        page[DATA_PAGE_RECORD_COUNT_OFFSET..DATA_PAGE_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&new_count.to_le_bytes());

        write_page(&mut file, page_index, &page)?;
        file.flush()?;
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

fn validate_file(path: &Path) -> Result<(), StorageError> {
    let metadata = std::fs::metadata(path)?;
    let len = metadata.len();
    if len < PAGE_SIZE as u64 {
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
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn signed_decimal_int_literals_accept_noncanonical_zero_spelling() {
    let path = temp_db_path("signed_decimal_int_literals_accept_noncanonical_zero_spelling");

    assert_exec(
        &path,
        "CREATE TABLE nums (n INT); INSERT
```

### docs/file_format.md
- excerpt_chars: 3111
- clipped: false

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

## Validation Errors

Opening or reading a file validates the header and every declared data page. Short files return a truncated-file error. Non-page-aligned files or missing declared pages return a truncated-page error. Invalid file or data page magic returns an invalid-magic error. Unsupported format versions return an unsupported-version error. Record lengths that exceed the page used bytes or page capacity return a corrupt-record-length error. Oversized appends return a record-too-large error.

## Compatibility Note

V1 is pre-launch and does not guarantee backward compatibility for existing user data. After this page and record format is introduced, format changes must not be made implicitly: the documentation and deterministic tests must be updated together with any intentional format change. SQL logical-record evolution must preserve the lower-level page framing unless a future task explicitly changes the page format contract.
```

### docs/cli_contract.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path.

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

Successful `db exec` writes no stderr. It writes stdout only for `SELECT * FROM`
statements. Each result set prints the stored column header followed by rows in
successful `INSERT` append order, with `|` as the field delimiter and `\n` after
every output line. Multiple `SELECT` statements repeat the header with no blank
line, separator, or count line.

The supported SQL subset is documented in `docs/sql_subset.md`.

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ...;
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

```text
error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1
hint: INSERT values must match the table schema exactly.
```

```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

Invalid SQL logical records exit `1`, write empty stdout, and use this stderr:

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## Reserved Future Commands

The following names are reserved for later V1 work but a
```

### AGENTS.md
- excerpt_chars: 4000
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
- Keep CLI output stable once documented in `docs/cli_contract.md` or covered by integration tests.

## SDD Workflow

- Start each implementation from the task-provided `spec.md` and `contracts.md`.
- Treat `contracts.md` as the completion contract: every listed behavior, non-goal, required check, and acceptance artifact must be satisfied or explicitly blocked.
- Use repo-local `specs/**` as product history, templates, or local execution artifacts unless the active task explicitly selects one as the current spec.
- Before editing, identify the affected product contract: CLI behavior, persisted data compatibility, documented errors, tests, or durable docs.
- Preserve existing public behavior unless the active spec explicitly changes it.
- Add or update focused tests with behavior changes, including negative and edge cases for persisted data, recovery, indexing, and CLI contract work.
- Update durable docs only when the user-facing or compatibility contract changes.
- If the spec, contract, and repo reality conflict, stop and report the conflict instead of silently changing scope.
- Treat SDD pipeline `result_*.md` files as phase status reports. Their status line or `PM_RESULT:` sentinel expresses next owner and readiness; it is not a substitute for contract evidence.
- Treat latest review/report files as verifier or reviewer SSOT: `qa_prep_review.md`, `impl_review.md`, `impl_brake_review.md`, `code_review.md`, and `final_review.md`.
- Execution and repair phases may read the latest review/report files as input, but must not check off, delete, or overwrite reviewer findings unless the phase explicitly owns that review/report.
- On retry or repair re-entry, read the latest review/report first and repair only the actionable open items such as `Repair Targets`, `Must Fix Now`, `Next Action`, or `Verify Risks`.
- Preserve previous review/report contents in the matching `.history.md` file when the owning verifier or reviewer refreshes the latest SSOT; the latest file should contain only current open findings and verdict context.
- Use `development_state.md` only as high-density implementation state between passes. It supports handoff, but completion still depends on the active contract, latest review/report state, and required verification evidence.
- Task completion is blocked until `scripts/verify` and any contract-required checks pass.

## Architecture Boundary

Current structure is intentionally small:

- `src/main.rs`: CLI entrypoint and current implementation surface.
- `tests/`: integration and behavior tests; prefer black-box CLI behavior here.
- `docs/`: durable product documentation and contract references.
- `docs/history_archives/history.md`: append-only product history.
- `work_queue/progress.md`: current product progress view.
- `specs/`: product history or local execution artifacts from completed work.

No deeper module boundary map exists yet. When meaningful layers or modules are
```

### work_queue/progress.md
- excerpt_chars: 1848
- clipped: false

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, and the minimal SQL schema/execute path for `CREATE TABLE`, `INSERT INTO ... VALUES`, and `SELECT * FROM ...`. The next smallest implementation handoff should target indexing, recovery, or validation gaps on top of the SQL execution baseline.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | missing_evidence | No primary B-tree index yet. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | missing_evidence | No transaction or WAL recovery path yet. |
| gap-v1-deterministic-crash-matrix | missing_evidence | No deterministic crash matrix yet. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | missing_evidence | No `db check` invariant command yet. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |

## Recent Entries

- 2026-05-17: Implemented the minimal SQL schema/execute path for `db exec <path> <sql>`, including parser/executor, SQL logical records over `PageStore`, exact CLI error contracts, restart and mid-command failure coverage, and SQL/file-format docs.
```

### docs/history_archives/history.md
- excerpt_chars: 466
- clipped: false

```text
# Persistent DB Core History

## 2026-05-15

- Created `persistent-db-core` as a V1 managed repo for CAO Autopilot.
- Initial product boundary is a Rust CLI binary named `db`.
- No V1 implementation gaps have verified completion evidence yet.

## 2026-05-17

- Added the minimal SQL schema/execute milestone: `db exec <path> <sql>` now supports the documented `CREATE TABLE`, `INSERT`, and `SELECT *` path with deterministic persistence and error-contract coverage.
```

### .codex/agents/decision-brake-readiness-reviewer.toml
- excerpt_chars: 1537
- clipped: false

```text
name = "Decision Brake Readiness Reviewer"
description = "Reviews whether a decision is executable and handoff-ready, including missing inputs, evidence gaps, and required followups."
model = "gpt-5.4"
model_reasoning_effort = "high"
sandbox_mode = "read-only"
developer_instructions = '''
당신은 중요한 의사결정의 실행 준비도와 handoff 가능성을 검토하는 decision-brake lens-owner 입니다. 역할은 방향이 매력적인지 평가하는 것이 아니라, 지금 실제로 실행 가능한 상태인지와 구현자에게 새 결정이 새지 않는지 판단하는 것입니다.

## Scope

- 아이디어, 프로젝트 방향, 스펙, 아키텍처, 운영 방식, 우선순위 결정 등 무엇이든 검토 대상이 될 수 있습니다.
- CAO candidate review 에서는 handoff impact, missing evidence, required followups 를 특히 선명하게 합니다.
- 직접 구현하거나 스캐폴딩하지 않습니다.

## Readiness Standard

1. required inputs, owner, acceptance criteria, verification method 가 닫혀 있는지 확인합니다.
2. 구현자가 제품/기술 결정을 새로 내려야만 진행할 수 있는 빈칸을 찾습니다.
3. missing evidence, protected area, human input, conflicting evidence 가 실행 가능성을 막는지 봅니다.
4. handoff impact 를 ready, changes-needed, clarification-needed, escalation-needed 중 하나로 제안합니다.
5. 최종 brake level 은 내리지 않습니다. 메인 decision-brake 가 verdict 와 handoff 정책을 정하는 데 필요한 재료만 제공합니다.

## Required Focus

- required execution inputs
- acceptance criteria clarity
- verification method clarity
- unresolved decisions
- missing evidence or human input
- handoff impact and required followups

## Output Shape

아래 순서로 답하십시오.

1. Decision under review
2. Execution inputs present
3. Missing inputs or evidence
4. Decisions leaking to the implementer
5. Handoff impact
6. Required followups before execution

질문이 필요하면, handoff 가능성을 실제로 바꾸는 질문만 최소 개수로 남깁니다.
'''
```

### .codex/agents/project-reviewer.toml
- excerpt_chars: 268
- clipped: false

```text
model = "gpt-5.4"
model_reasoning_effort = "high"
developer_instructions = """
You are the project reviewer. Critically assess plans, architecture, tradeoffs, and risk. Prefer identifying logical gaps, hidden costs, and stronger alternatives over broad agreement.
"""
```

### .codex/agents/task-master.toml
- excerpt_chars: 198
- clipped: false

```text
model = "gpt-5.4"
model_reasoning_effort = "high"
developer_instructions = """
You break accepted specs into execution-ready tasks with explicit dependencies, ordering, and completion criteria.
"""
```
