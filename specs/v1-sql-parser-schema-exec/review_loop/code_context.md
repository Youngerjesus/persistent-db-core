# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: 8aea6208d2a42d51a78306ccd57dbbc5e7aad6a4
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-17T10:41:01.272604+00:00
- selected_files: src/main.rs, src/lib.rs, src/storage.rs, tests/cli_contract.rs, docs/cli_contract.md, AGENTS.md, work_queue/progress.md, docs/history_archives/history.md, .codex/agents/decision-brake-readiness-reviewer.toml, .codex/agents/project-reviewer.toml, .codex/agents/task-master.toml, .codex/agents/task-reviewer.toml

## Omitted Reasons
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md: not_git_tracked
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md: not_git_tracked
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md: not_git_tracked
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md: not_git_tracked
- autopilot/project_manager/tasks/tasks.json: not_git_tracked
- autopilot/ssot/current-artifact.md: not_git_tracked
- catalog/executor: not_git_tracked
- differential/property: not_git_tracked
- docs/sql_subset.md: not_git_tracked
- flow:create-table-insert-select: not_git_tracked
- parser/schema: not_git_tracked
- persistent-db-core_worktree/main/docs/cli_contract.md: not_git_tracked
- persistent-db-core_worktree/main/docs/file_format.md: not_git_tracked
- persistent-db-core_worktree/main/docs/v1_spec.md: not_git_tracked
- persistent-db-core_worktree/main/src/lib.rs: not_git_tracked
- persistent-db-core_worktree/main/src/main.rs: not_git_tracked
- persistent-db-core_worktree/main/src/storage.rs: not_git_tracked
- persistent-db-core_worktree/main/tests/cli_contract.rs: not_git_tracked
- persistent-db-core_worktree/main/tests/page_storage.rs: not_git_tracked
- persistent-db-core_worktree/main/work_queue/progress.md: not_git_tracked
- route:db-sql-exec: not_git_tracked
- schema/execute: not_git_tracked
- src/sql.rs: not_git_tracked
- tests/sql_exec.rs: not_git_tracked

## File Excerpts

### src/main.rs
- excerpt_chars: 1195
- clipped: false

```text
use std::env;
use std::process;

const HELP: &str = "\
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
Supported commands:
  help        Print this help text.
Reserved future commands:
  open <path>
  exec <path> <sql>
  check <path>
  bench <path>
V1 bootstrap scope:
  This build only defines the CLI contract and smoke baseline.
  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [arg] if arg == "--help" || arg == "help" => {
            print!("{HELP}");
        }
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
```

### src/lib.rs
- excerpt_chars: 17
- clipped: false

```text
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

### tests/cli_contract.rs
- excerpt_chars: 2859
- clipped: false

```text
use std::process::{Command, Output};

const REQUIRED_HELP_LINES: &[&str] = &[
    "db - deterministic single-process V1 database CLI",
    "Usage:",
    "  db --help",
    "  db help",
    "Supported commands:",
    "  help        Print this help text.",
    "Reserved future commands:",
    "  open <path>",
    "  exec <path> <sql>",
    "  check <path>",
    "  bench <path>",
    "V1 bootstrap scope:",
    "  This build only defines the CLI contract and smoke baseline.",
    "  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.",
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
```

### docs/cli_contract.md
- excerpt_chars: 1931
- clipped: false

```text
# V1 `db` CLI Contract

This slice defines only the deterministic command-line contract and smoke baseline for the `db` binary.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
```

Both commands exit with code `0`, write no stderr, and write identical help text to stdout.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
Supported commands:
  help        Print this help text.
Reserved future commands:
  open <path>
  exec <path> <sql>
  check <path>
  bench <path>
V1 bootstrap scope:
  This build only defines the CLI contract and smoke baseline.
  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: `db --help` or `db help` printed the help contract successfully.
- `2`: the first argument was unsupported, or no supported command was provided.

## Unsupported Input

Unsupported arguments and subcommands exit with code `2`, write no stdout, and write this stderr format:

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

`<token>` is the first unsupported token supplied by the user. For example, `db --unknown` reports `--unknown`, and `db open demo.db` reports `open`.

## Reserved Future Commands

The following names are reserved for later V1 work but are not executable in this slice:

```text
open <path>
exec <path> <sql>
check <path>
bench <path>
```

Invoking any reserved command currently follows the unsupported input behavior.

## Non-Goals

This slice does not implement storage pages, SQL execution, indexes, transactions, WAL, recovery, networking, multi-process concurrency, or distributed storage.
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
- excerpt_chars: 1318
- clipped: false

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` is bootstrapped as a Rust CLI skeleton. The next smallest implementation handoff should target `gap-v1-bootstrap-cli-contract`.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | missing_evidence | No SQL parser, schema catalog, or executor yet. |
| gap-v1-primary-btree-index | missing_evidence | No primary B-tree index yet. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | missing_evidence | No transaction or WAL recovery path yet. |
| gap-v1-deterministic-crash-matrix | missing_evidence | No deterministic crash matrix yet. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | missing_evidence | No `db check` invariant command yet. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |
```

### docs/history_archives/history.md
- excerpt_chars: 243
- clipped: false

```text
# Persistent DB Core History

## 2026-05-15

- Created `persistent-db-core` as a V1 managed repo for CAO Autopilot.
- Initial product boundary is a Rust CLI binary named `db`.
- No V1 implementation gaps have verified completion evidence yet.
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

### .codex/agents/task-reviewer.toml
- excerpt_chars: 192
- clipped: false

```text
model = "gpt-5.4"
model_reasoning_effort = "high"
developer_instructions = """
You review task breakdowns to make sure they are decision-complete, technically constrained, and verifiable.
"""
```
