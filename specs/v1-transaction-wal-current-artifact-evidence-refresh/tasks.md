# Tasks: v1-transaction-wal-current-artifact-evidence-refresh

## Execution Rules
- Keep `spec.md` and `contracts.md` frozen.
- Do not edit protected `ssot/` or `policies/`.
- Prefer evidence-only changes. Touch production code, tests, scripts, or durable docs only when a contract-required proof is missing or failing.
- A failed or insufficient `scripts/verify_crash_matrix` is a blocker unless an in-scope repair makes it directly prove the contract.

## Task List

### T1. Confirm Live Repo State
- Record `git rev-parse HEAD`.
- Record `git status --short`.
- Confirm the presence of `tests/wal_recovery.rs`, `tests/crash_matrix.rs`, `scripts/verify`, and `scripts/verify_crash_matrix`.
- Output target: `evidence/current-repo-sha.txt`.

### T2. Run Baseline Verification
- Run `scripts/verify` from the repo root.
- Capture exit status and enough stdout/stderr to prove the baseline commands ran.
- Output target: `evidence/command-log.md`.
- Requirement links: `REQ-9-provide-wal-or-equivalent-write-80297892`.

### T3. Run Focused WAL Recovery Test Suite
- Run `cargo test --test wal_recovery`.
- Run each focused test named in the contract:
  - `committed_wal_replay_survives_reopen_via_cli`
  - `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`
  - `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`
  - `committed_frame_after_incomplete_tail_cleanup_remains_replayable`
  - `committed_wal_frame_ahead_of_page_store_fails_deterministically`
- Capture exit status and output in `evidence/command-log.md`.
- Update `evidence/requirement-evidence.md` with per-test requirement mapping.

### T4. Generate WAL Sidecar/Reopen Smoke
- Create a fresh temporary DB path.
- Run `db exec <path> "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`.
- Record exit code, stdout, stderr, and `<path>.wal` existence/byte length.
- Run `db exec <path> "SELECT * FROM users;"` in a separate process.
- Record exact stdout `id|name\n1|ada\n2|bea\n`, exit code, stderr, and post-reopen WAL sidecar state.
- Redact the machine-specific temp path while preserving sidecar identity.
- Output target: `evidence/wal-sidecar-smoke.md`.
- Requirement links: `REQ-8-committed-writes-survive-crash-and-35caf667`, `REQ-9-provide-wal-or-equivalent-write-80297892`.

### T5. Run Crash Matrix Verification
- Run `scripts/verify_crash_matrix`.
- Confirm `target/crash_matrix/crash_matrix_report.md` exists for the current run.
- Capture the validator outcome and summarize case IDs `CM-001` through `CM-006`.
- Output target: `evidence/crash-matrix-log.md`.
- Requirement links: `REQ-9-checkpoint-or-log-truncation-must-d633d286`, with supporting links for idempotence and rollback rows where applicable.

### T6. Write Requirement Evidence Matrix
- Create `evidence/requirement-evidence.md`.
- For every `REQ-8-*` and `REQ-9-*` ID, include:
  - requirement ID exactly as written in `contracts.md`
  - command(s)
  - expected behavior
  - observed result
  - artifact path(s)
  - blocker status
- Do not replace specific rows with generic "verification passed" language.

### T7. Review Durable Docs For Drift
- Review `docs/file_format.md`, `docs/cli_contract.md`, and `docs/v1_acceptance.md` against observed evidence.
- If docs already describe current WAL sidecar, replay, retained-frame, rollback/incomplete, and crash-matrix behavior accurately, do not edit them.
- If a user-facing or compatibility statement is stale, make the smallest doc update and rerun `scripts/verify`.

### T8. Final Review Artifact
- Write `specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md`.
- Include current SHA, command evidence, requirement IDs, artifact refs, non-visual evidence note, and verdict.
- Use `PASS` only if all required commands pass and evidence artifacts exist.
- Use blocker language if the crash matrix requirement or any other contract row remains unproven.

## Subtasks For High-Risk Rows

### Checkpoint/Log-Truncation Row
- Verify `scripts/verify_crash_matrix` proves all expected crash cases, not just that `cargo test --test crash_matrix` passed.
- Confirm generated report includes current run id, command provenance, expected rows, actual rows, WAL/file-format assertion result, and exit status for each case.
- Confirm `CM-004` repeated reopen and `CM-005` interrupted recovery details are present.
- If any assertion is missing, return `blocking` unless repaired in scope.

### Idempotence Row
- Keep incomplete-tail exclusion and repeated recovery evidence distinct in `requirement-evidence.md`.
- Link both focused WAL tests required by the contract.
- Use crash matrix `CM-004`/`CM-005` as supporting evidence, not a replacement for required focused tests.

### WAL Sidecar Row
- Record the sidecar as persisted evidence: path form, existence, byte length after mutation, and byte length after reopen.
- Confirm the smoke uses separate `db exec` invocations so it proves reopen/recovery rather than a single in-memory process.
