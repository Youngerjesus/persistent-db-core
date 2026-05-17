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

