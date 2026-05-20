# Code Context Evidence

- available: true
- repo_root: <redacted-managed-repo-root>
- head_sha: bed51c0d35f392458840870401f304a157a3b005
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-20T14:35:43.413762+00:00
- selected_files: tests/wal_recovery.rs, scripts/verify, docs/cli_contract.md, docs/file_format.md, docs/v1_acceptance.md, specs/v1-transaction-wal-recovery/final_review.md, specs/v1-wal-recovery-current-sha-proof/analysis_report.md, specs/v1-wal-recovery-current-sha-proof/code_review.md, specs/v1-wal-recovery-current-sha-proof/contracts.md, specs/v1-wal-recovery-current-sha-proof/design.md, specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.exit, specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.stderr

## Omitted Reasons
- /scripts/verify: context path escapes repo root: /scripts/verify
- CLI/SQL: not_git_tracked
- checkpoint/log: not_git_tracked
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/contracts.md: not_git_tracked
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/spec.md: not_git_tracked
- rollback/uncommitted: not_git_tracked
- specs/v1-transaction-wal-current-artifact-evidence-refresh: not_git_tracked
- transaction/WAL: not_git_tracked

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
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: help printed successfully, `db exec` completed successfully, or
  `db check` passed. `db bench` also exits `0` after generating passing
  Section 14 evidence.
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

`db bench <extra>` is unsupported and reports `bench`.

## SQL Execution

Successful `db exec` writes no stderr. It writes stdout only for supported
`SELECT *` statements. Each result set prints the stored column header followed
by rows, with `|` as the field delimiter and `\n` after every output line.
Tables without a primary key scan in successful `INSERT` append order. Tables
declared with one `INT PRIMARY KEY` or `INTEGER PRIMARY KEY` scan in ascending
primary-key order. `INTEGER` is an accepted spelling alias for the existing
integer column type; it does not add SQL affinity behavior or any other type
alias.
`SELECT * FROM <table> WHERE <primary_key> = <int>;` performs exact primary-key
lookup and prints only the matching row, or only the header when the key is
missing. Multiple `SELECT` statements repeat the header with no blank line,
separator, or count line.

`CREATE INDEX <index> ON <table>(<integer_column>);` creates a durable
secondary index over an existing `INT` column. Successful `CREATE TABLE`,
`INSERT`, `CREATE INDEX`, primary-key-targeted `UPDATE`, and
primary-key-targeted `DELETE` mutations exit `0`, write empty stdout/stderr
unless a later `SELECT` writes rows, and are durable across later `db exec`
process starts for the same database path. WAL sidecar details are documented
in `docs/file_format.md`; they do not change successful `db exec` stdout,
stderr, or exit codes.

`U
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
documented duplicate-primary-key invalid-storage error:

```text
error: invalid SQL storage record: duplicate primary key for table users: 2
hint: primary key values must be unique in persisted SQL storage.
```

Other corrupt SQL logical-record data, such as unknown record tags, continues
to use the generic unknown-record-tag invalid-storage error. Because no separate
index metadata is stored, missing index metadata is not a V1 failure mode.

Secondary indexes are persisted as append-only SQL logical records above the
same page framing. Existing no-index databases containing only `C` and `R`
records remain compatible: they reopen normally, and a later `CREATE INDEX`
backfills existing rows.

`CREATE INDEX` writes all backfill `E` records first, then writes the final `X`
metadata record as the commit marker. The `build_id` in `E` and `X` is the
durable SQL logical-record count before that `CREATE INDEX` appends anything.
An `E` record without a matching
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
| `gate-v1-indexes` | `REQ-7-implement-integer-primary-key-as-9c698e08` | `tests/primary_index.rs`; `tests/sql_exec.rs`; `scripts/verify_primary_index_acceptance`; `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`; `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256` | Verified product SHA `6008189f30b8e2cd38ad6ab5994c89c373d386ca`; current evidence repair identity in `artifact_identity.sha256`; base source SHA `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`; `cargo test --test primary_index`; `cargo test --test sql_exec primary_key`; `scripts/verify_primary_index_acceptance`; `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-secondary-index-proof` | `tests/secondary_index.rs`; `src/sql.rs`; `src/index.rs`; `docs/cli_contract.md`; `docs/file_format.md` | `cargo tes
```

### specs/v1-transaction-wal-recovery/final_review.md
- excerpt_chars: 2319
- clipped: false

```text
# Final Review: v1-transaction-wal-recovery

Verdict: PASS

## Scope

Closed final execution for `task-2026-05-17-23-45-17-v1-transaction-wal-recovery` against `spec.md`, `contracts.md`, implementation verification, and code review.

Reviewed and finalized:
- `src/storage.rs`
- `tests/wal_recovery.rs`
- `docs/file_format.md`
- `docs/cli_contract.md`
- `work_queue/progress.md`
- `docs/history_archives/history.md`
- task-scoped artifacts under `specs/v1-transaction-wal-recovery/**`

## Closure Checks

- `tests/wal_recovery.rs` exists and covers CLI-visible committed replay, incomplete trailing WAL exclusion, retained WAL idempotence, incomplete-tail cleanup before later appends, and ahead-of-store deterministic corruption.
- `docs/file_format.md` documents WAL sidecar path, frame layout/framing, replay order, committed/rolled-back/incomplete handling, retained-frame behavior, and existing database compatibility.
- `docs/cli_contract.md` was updated only to remove stale WAL/recovery non-goal wording and describe durability across later `db exec` starts. Public commands, stdout, stderr, and exit codes are unchanged.
- `work_queue/progress.md` and `docs/history_archives/history.md` were synced for the shipped recovery milestone.
- No `docs/*/memory.md` files exist, so no component memory update was applicable.
- Protected `ssot/` and `policies/` areas were not modified.

## Open Items

None.

## Verification Evidence

- `cargo test --test wal_recovery`: pass, 4 tests.
- `cargo test`: pass, all current unit, integration, and doc-test targets.
- `./scripts/verify`: pass, including `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`.
- Canonical CLI smoke with a redacted temp DB path:
  - create/insert command exit `0`, stdout `""`, stderr `""`.
  - reopen select command exit `0`, stderr `""`, stdout bytes `69647c6e616d650a317c6164610a327c6265610a`, equivalent to `id|name\n1|ada\n2|bea\n`.
  - WAL sidecar present with size `202` bytes.

## Remote State

Ready for commit, push, PR creation, and merge by `finish`.

## Next Action

Create final verification manifest, commit the task scope, push the branch, open a PR against `main`, and merge after local verification evidence is attached.

## Updated At

2026-05-18 00:50:02 KST
```

### specs/v1-wal-recovery-current-sha-proof/analysis_report.md
- excerpt_chars: 2636
- clipped: false

```text
# Analysis Report: v1-wal-recovery-current-sha-proof

## Verdict
PASS

## Cross-Artifact Consistency
- `spec.md` requires current task worktree proof for committed mutation survival, uncommitted change absence, incomplete trailing WAL exclusion, `./scripts/verify`, CLI smoke output, WAL sidecar state, and explicit gate/requirement mapping.
- `contracts.md` preserves protected areas, freezes acceptance evidence, excludes browser/UX proof as acceptance evidence, and requires final transcript/report details.
- `research.md` selects an evidence-first closure path and keeps repair scope limited to verification failures.
- `plan.md` defines the exact proof layers and command set without changing canonical scope.
- `design.md` separates test, baseline, CLI, file-state, and review proof layers.
- `tasks.md` breaks the work into current-SHA identity, focused WAL test, baseline verification, CLI smoke, doc/code delta review, and acceptance mapping.

## Acceptance Coverage
| Contract Requirement | Planning Coverage |
|---|---|
| Current HEAD and dirty state recorded | `plan.md` evidence table; `tasks.md` T1 |
| `cargo test --test wal_recovery` passes | `plan.md` verification commands; `tasks.md` T2 |
| Committed mutation survives separate reopen process | `design.md` recovery proof flow; `tasks.md` T2/T4 |
| Uncommitted change absence has deterministic scenario | `research.md` proof layer; `tasks.md` T2 fixture rationale |
| Incomplete trailing WAL entry exclusion is separate | `tasks.md` T2.4 and T4 sidecar state capture |
| `./scripts/verify` passes | `tasks.md` T3 |
| CLI create/insert smoke transcript | `plan.md` CLI smoke commands; `tasks.md` T4 |
| CLI reopen/select exact stdout | `plan.md` expected output; `tasks.md` T4 |
| WAL sidecar existence and byte length at both points | `plan.md` evidence table; `tasks.md` T4 |
| Final evidence maps gap/gate/requirement IDs | `tasks.md` T6 |
| Browser/UX proof not substituted | `readiness-preflight.md`, `research.md`, `plan.md`, `tasks.md` |

## Implementation Risks To Carry Forward
- Evidence transcript must be created even if no source changes are needed.
- `git status --short` may include planning artifacts from this phase; implementation evidence should distinguish planned artifacts from unrelated dirt.
- If command output is summarized instead of fully pasted, the report must still include exact required stdout/stderr for CLI smoke.
- Cleanup of the temp DB before WAL length recording would invalidate required file-state evidence.
- Any verifier rejection followed by a needed second recovery attempt must escalate per contract.

## Blockers
None.
```

### specs/v1-wal-recovery-current-sha-proof/code_review.md
- excerpt_chars: 2402
- clipped: false

```text
Verdict: PASS

## Scope

- Phase: Code Review Verification for `task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof`.
- Verification target: all changes relative to `main`, including committed branch delta and current worktree delta.
- `git log --oneline main..HEAD`: no commits.
- `git diff --stat main...HEAD`: no committed diff.
- Current worktree reviewed: ` M tests/wal_recovery.rs` and `?? specs/v1-wal-recovery-current-sha-proof/`.
- Product delta reviewed: `tests/wal_recovery.rs` adds a shared WAL frame fixture builder and `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`.
- Evidence/report delta reviewed: task-scoped spec package under `specs/v1-wal-recovery-current-sha-proof/`, including `qa_mapping.md`, `contracts.md`, `final_report.md`, `verify_evidence_contract.sh`, and implementation evidence transcripts.

## Findings

- None.

## Must Fix Now

- None.

## Residual Risks

- The complete rolled-back frame is injected directly because V1 has no public rollback or incomplete transaction CLI. This remains contract-aligned: `docs/file_format.md` documents state `0x02` as rolled back, and `src/storage.rs` skips that state during replay.
- The proof package remains intentionally dirty/untracked in the task worktree. `final_report.md` records this status and `verify_evidence_contract.sh` validates it against live `git status --short`.
- Python-specific checks such as `pytest`, `ruff`, and `mypy` are not applicable in this Rust CLI repo: no tracked Python files or Python tool config files were found. Rust static analysis is covered by `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.

## Verification

- `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`: exit `0`, stdout `evidence contract shape ok`.
- `cargo test --test wal_recovery`: exit `0`, 5 tests passed.
- `./scripts/verify`: exit `0`, covering `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`.
- `cargo fmt --check`: exit `0`.
- `cargo clippy --all-targets -- -D warnings`: exit `0`.
- Python applicability scan for `pyproject.toml`, `mypy.ini`, `.mypy.ini`, `.ruff.toml`, `ruff.toml`, `pytest.ini`, `requirements*.txt`, `tox.ini`, and tracked `*.py`: no matches.

## Next Action

- Route to the next phase. No code-review retry is required.

## Updated At

2026-05-18T01:52:03+09:00
```

### specs/v1-wal-recovery-current-sha-proof/contracts.md
- excerpt_chars: 3023
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
- 생성 대상 코드 또는 문서: 현재 SHA 기준 WAL 복구 증거 재검증에 대한 closure.
- 생성 대상 테스트 또는 verification output: scheduler terminal result와 supporting evidence.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria 항목은 test output, command output, WAL file-state evidence, manual review evidence 또는 explicit blocker에 연결되어야 합니다.
- spec hardening 중 candidate acceptance criteria를 generic completion wording으로 약화하거나 병합하거나 대체하지 않습니다.
- Browser evidence, DOM capture, screenshot artifacts, rendered route state, UX design review는 이 Rust CLI WAL recovery task의 acceptance evidence가 아닙니다.
- `cargo test --test wal_recovery`가 현재 task worktree HEAD에서 통과하고, committed mutation이 별도 `db exec` process의 reopen/replay 뒤 살아남는 케이스를 검증합니다.
- uncommitted change absence는 별도 deterministic scenario로 검증해야 합니다. 공개 CLI에 rollback 또는 uncommitted transaction command가 없으면 WAL fixture를 직접 작성할 수 있지만, evidence는 해당 fixture가 V1에서 관찰 가능한 uncommitted WAL state를 대표하는 이유를 기록해야 합니다.
- incomplete trailing WAL entry exclusion은 별도 deterministic scenario로 검증해야 합니다. evidence는 incomplete tail의 ghost row가 recovery 후 결과 rows 또는 storage row set에 나타나지 않고, WAL sidecar가 향후 replay 가능한 상태로 유지되거나 cleanup되었음을 기록해야 합니다.
- `./scripts/verify`가 통과해 fmt, clippy, full test suite, `db --help` smoke를 함께 보장합니다.
- CLI smoke transcript는 temp DB path를 생성한 뒤 `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`를 실행하고, exit code `0`, stdout `""`, stderr `""`를 기록해야 합니다.
- CLI smoke transcript는 별도 process에서 `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"`를 실행하고, exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 기록해야 합니다.
- CLI smoke transcript는 create/insert 후 `$DB_PATH.wal` 존재 여부와 byte length, reopen select 후 `$DB_PATH.wal` 존재 여부와 byte length를 기록해야 합니다. 구현이 complete WAL frames를 retained sidecar로 유지한다면 sidecar는 존재하고 non-empty여야 합니다.
- 최종 report 또는 evidence transcript는 `git rev-parse HEAD`, `git status --short`, 실행한 `cargo test --test wal_recovery`, `./scripts/verify`, CLI smoke command의 실행 명령과 stdout/stderr 또는 transcript path를 포함해야 합니다.
- 최종 evidence는 `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, `req-v1-wal-recovery-proof`에 명시적으로 매핑되어야 합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
```

### specs/v1-wal-recovery-current-sha-proof/design.md
- excerpt_chars: 3251
- clipped: false

```text
# Design: v1-wal-recovery-current-sha-proof

## Architecture Intent
This task is a proof-closure slice. It should not redesign WAL recovery unless current-SHA verification exposes a defect. The implementation worker should treat current WAL recovery behavior as the candidate implementation and produce current-SHA acceptance evidence.

## Recovery Proof Flow
1. Confirm repository identity:
   - capture current HEAD;
   - capture dirty state;
   - confirm no conflicts.
2. Verify deterministic recovery tests:
   - committed replay through separate `db exec` processes;
   - incomplete trailing WAL entry does not create `9|ghost`;
   - incomplete-tail cleanup leaves retained WAL reachable for future replay;
   - ahead-of-page-store committed frame fails deterministically.
3. Verify full baseline:
   - `./scripts/verify` covers formatting, clippy, full test suite, and `db --help` smoke.
4. Capture CLI smoke:
   - create table and two inserts in one process;
   - reopen select in another process;
   - record exact stdout/stderr/exit code for both.
5. Capture sidecar state:
   - inspect `$DB_PATH.wal` immediately after create/insert;
   - inspect `$DB_PATH.wal` immediately after reopen/select.
6. Write final evidence report:
   - include command transcripts or precise summary;
   - include WAL byte lengths;
   - include fixture rationale for uncommitted/incomplete state;
   - map evidence to gap, gate, and requirement IDs.

## Proof Layers
| Layer | Purpose | Artifact |
|---|---|---|
| Test proof | deterministic Rust integration coverage | `cargo test --test wal_recovery` output |
| Baseline proof | repo-wide verification contract | `./scripts/verify` output |
| CLI proof | public process/reopen behavior | smoke transcript |
| File-state proof | retained WAL sidecar behavior | sidecar exists/byte length entries |
| Review proof | acceptance mapping and fixture rationale | final report/transcript |

## Existing Code Boundary
- `src/storage.rs` owns page file validation, WAL sidecar path, frame append, replay, checksum, idempotence, and incomplete-tail cleanup.
- `tests/wal_recovery.rs` owns black-box CLI recovery tests and direct WAL fixture construction for states not reachable through public CLI.
- `docs/file_format.md` owns WAL sidecar format and replay semantics.
- `docs/cli_contract.md` owns public CLI behavior. It should change only if current documentation is stale or contradictory.
- `src/main.rs` and `src/lib.rs` are not expected to change for an evidence-only closure.

## Evidence Report Schema
The implementation report may be Markdown but should use stable headings:

```markdown
# WAL Recovery Current-SHA Evidence

## Identity
- command: git rev-parse HEAD
- exit_code:
- stdout:
- stderr:
- command: git status --short
- exit_code:
- stdout:
- stderr:

## Focused Verification
...

## Baseline Verification
...

## CLI Smoke Transcript
...

## WAL Sidecar State
...

## Fixture Rationale
...

## Acceptance Mapping
...
```

## Failure Handling
If a required command fails, the worker should record the failed command output, repair the smallest scoped defect, and rerun the affected proof. If a second recovery attempt would be needed after verifier rejection, stop and escalate per `contracts.md`.
```

### specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.exit
- excerpt_chars: 2
- clipped: false

```text
0
```

### specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.stderr
- excerpt_chars: 0
- clipped: false

```text

```
