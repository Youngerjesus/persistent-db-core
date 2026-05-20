# Implementation Plan: v1-transaction-wal-current-artifact-evidence-refresh

## Objective
Refresh current-artifact evidence for transaction/WAL recovery requirement rows so `gate-v1-transactions-wal-recovery` can be evaluated from this package's own requirement IDs, command outputs, current repo SHA, WAL sidecar evidence, and crash-matrix evidence.

## Non-Goals
- No new CLI commands, SQL grammar, transaction syntax, network service, daemon, or remote dependency.
- No scheduler/control-plane status substitution for artifact evidence.
- No visual, DOM, screenshot, or UX design-review evidence.
- No edits to `spec.md`, `contracts.md`, protected `ssot/`, or `policies/`.

## Product Contract Affected
- Persisted data compatibility and WAL sidecar evidence.
- CLI-visible reopen/recovery behavior through `db exec`.
- Verification evidence and durable task-scoped artifact mapping.
- `docs/file_format.md`, `docs/cli_contract.md`, and `docs/v1_acceptance.md` only if implementation finds stale or missing user-facing recovery documentation.

## Implementation Strategy
1. Reconfirm live state: current HEAD, dirty state, and presence of `tests/wal_recovery.rs`, `tests/crash_matrix.rs`, `scripts/verify`, and `scripts/verify_crash_matrix`.
2. Run and capture the full required command set:
   - `scripts/verify`
   - `cargo test --test wal_recovery`
   - `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli`
   - `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`
   - `cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`
   - `cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable`
   - `cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically`
   - `scripts/verify_crash_matrix`
3. Generate fresh WAL sidecar/reopen smoke evidence with a temporary DB path:
   - create table and insert committed rows through `db exec`
   - record exit code, stdout, stderr
   - record `<db-path>.wal` existence and byte length after mutation
   - run a separate reopen/select `db exec`
   - record exact selected rows, exit code, stdout, stderr, and WAL sidecar state after reopen
4. Write current-artifact evidence files:
   - `evidence/current-repo-sha.txt`
   - `evidence/command-log.md`
   - `evidence/requirement-evidence.md`
   - `evidence/wal-sidecar-smoke.md`
   - `evidence/crash-matrix-log.md`
   - `final_review.md`
5. Map every requirement ID exactly:
   - `REQ-8-begin-commit-and-rollback-provide-44e7901f`
   - `REQ-8-committed-writes-survive-crash-and-35caf667`
   - `REQ-9-provide-wal-or-equivalent-write-80297892`
   - `REQ-9-recovery-must-be-idempotent-and-300531dc`
   - `REQ-9-checkpoint-or-log-truncation-must-d633d286`
6. If verification proves behavior without code/doc changes, keep the delta to evidence artifacts and final review.
7. If a required test or script is absent or fails, make only the minimum in-scope test/script/doc repair needed by the contract, then rerun and capture evidence. If `scripts/verify_crash_matrix` cannot directly prove checkpoint/log-truncation safety, stop with a human-required blocker.

## Evidence File Responsibilities
| Artifact | Required contents |
|---|---|
| `evidence/current-repo-sha.txt` | `git rev-parse HEAD`, dirty-state summary, command used to obtain identity. |
| `evidence/command-log.md` | Required command list, exit status, stdout/stderr summaries or transcript refs, current SHA, run timestamp. |
| `evidence/requirement-evidence.md` | Requirement ID matrix linking command, expected behavior, observed result, artifact refs, and blocker status. |
| `evidence/wal-sidecar-smoke.md` | Fresh CLI smoke command, exact stdout/stderr/exit, WAL path identity with temp path redacted, sidecar existence and byte lengths. |
| `evidence/crash-matrix-log.md` | `scripts/verify_crash_matrix` output summary plus copied or referenced `target/crash_matrix/crash_matrix_report.md` case evidence. |
| `final_review.md` | Current verdict, requirement IDs, command evidence, artifact paths, SHA, non-visual evidence note, and any blockers. |

## Verification Gate
Implementation may report success only when all contract-required commands pass and all required evidence artifacts are present and internally consistent. Otherwise it must report `blocking` with the exact failed command or missing proof.
