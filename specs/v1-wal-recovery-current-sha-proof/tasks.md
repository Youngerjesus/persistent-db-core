# Tasks: v1-wal-recovery-current-sha-proof

## Execution Rules
- Treat `spec.md` and `contracts.md` as frozen canonical inputs.
- Do not edit `ssot/` or `policies/`.
- In the plan phase, do not edit production code, tests, docs, runtime config, or final evidence artifacts.
- In the implementation phase, keep changes scoped to current-SHA WAL recovery proof closure.
- Preserve public CLI stdout, stderr, exit codes, and command surface unless blocked and escalated.
- Browser, DOM, screenshot, rendered-route, and UX design-review artifacts are not acceptance evidence for this task.

## Task List

### T1. Revalidate current worktree identity
Status: ready

Files:
- final evidence transcript/report under `specs/v1-wal-recovery-current-sha-proof/`

Details:
- Capture `git rev-parse HEAD`.
- Capture `git status --short`.
- Confirm whether only task-scoped files are dirty before implementation evidence is finalized.
- Read current `tests/wal_recovery.rs`, `docs/file_format.md`, `docs/cli_contract.md`, `src/storage.rs`, `src/main.rs`, and `src/lib.rs` before deciding whether edits are needed.

Subtasks:
- T1.1 Record current SHA command, exit code, stdout, stderr.
- T1.2 Record dirty-state command, exit code, stdout, stderr.
- T1.3 Note whether implementation starts from current observed SHA `33b480cac6cf9d505a86eda4c149a4471454f11d` or a newer task worktree SHA.

Acceptance evidence:
- Identity section in final evidence transcript/report.

### T2. Run focused WAL recovery proof
Status: ready

Files:
- `tests/wal_recovery.rs`
- final evidence transcript/report

Details:
- Run `cargo test --test wal_recovery`.
- Verify the test suite includes separate coverage for committed replay survival, uncommitted/incomplete absence, incomplete-tail cleanup, future replayability, and deterministic ahead-of-store failure.
- If coverage is missing or the test fails, repair only the scoped test or implementation defect and rerun.

Subtasks:
- T2.1 Capture command, exit code, stdout, stderr.
- T2.2 Map committed process-reopen survival to the relevant test name.
- T2.3 Map uncommitted absence to the direct WAL fixture and explain why this represents V1 observable state.
- T2.4 Map incomplete trailing entry exclusion separately from uncommitted absence.

Acceptance evidence:
- Passing `cargo test --test wal_recovery` output and mapping text.

### T3. Run baseline repository verification
Status: ready

Files:
- final evidence transcript/report

Details:
- Run `./scripts/verify` from the repo root.
- Record exit code and stdout/stderr or a precise transcript path.
- Do not silently skip missing tools or failed checks.

Subtasks:
- T3.1 Capture command line.
- T3.2 Capture exit code.
- T3.3 Summarize `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help` evidence from script output.

Acceptance evidence:
- Passing `./scripts/verify` output.

### T4. Capture required CLI smoke and WAL sidecar state
Status: ready

Files:
- final evidence transcript/report

Details:
- Create a temp `DB_PATH`.
- Run:
  - `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`
  - `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"`
- Record exact exit code, stdout, and stderr for both commands.
- Record `$DB_PATH.wal` existence and byte length after create/insert and after reopen/select.

Subtasks:
- T4.1 Capture create/insert command transcript; expected exit `0`, stdout `""`, stderr `""`.
- T4.2 Capture WAL sidecar state immediately after create/insert.
- T4.3 Capture reopen/select command transcript; expected exit `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`.
- T4.4 Capture WAL sidecar state immediately after reopen/select.
- T4.5 Clean up temp files only after all evidence is captured.

Acceptance evidence:
- CLI smoke transcript and WAL sidecar byte-length entries.

### T5. Review docs and code delta necessity
Status: ready

Files:
- `docs/file_format.md`
- `docs/cli_contract.md`
- `src/main.rs`
- `src/lib.rs`
- `src/storage.rs`
- final evidence transcript/report

Details:
- If current docs already describe WAL sidecar location, frame layout, replay order, incomplete-tail behavior, retained frames, and old-file compatibility, do not churn docs.
- If current CLI docs already preserve command/output behavior while acknowledging durable reopen behavior, do not churn docs.
- If code changes are not needed, explicitly state that closure is evidence-only at current SHA.

Subtasks:
- T5.1 Record doc review outcome for `docs/file_format.md`.
- T5.2 Record doc review outcome for `docs/cli_contract.md`.
- T5.3 Record whether any code/test/doc edits were made and why.

Acceptance evidence:
- Artifact delta summary in final report.

### T6. Write acceptance mapping and phase result
Status: ready

Files:
- final evidence transcript/report
- scheduler result file for the active implementation run

Details:
- Map every Candidate Acceptance Criteria item to concrete evidence.
- Explicitly map final evidence to:
  - `gap-v1-transaction-wal-recovery`;
  - `gate-v1-transactions-wal-recovery`;
  - `req-v1-wal-recovery-proof`.
- Use `success` only when the phase is complete and verifier-ready.
- Use `continue` only if real same-phase implementation work remains after concrete progress and evidence.
- Use `blocking` only for a real blocker that cannot be resolved inside the phase.

Subtasks:
- T6.1 Write acceptance mapping table.
- T6.2 Write scheduler result with `PM_PHASE_COMPLETE`.
- T6.3 Ensure final response termination block matches result file.

Acceptance evidence:
- Final evidence transcript/report plus scheduler result.

## Dependency Order
1. T1
2. T2
3. T3
4. T4
5. T5
6. T6

## Readiness Notes
- No human decision is required before implementation.
- Current-SHA verification is the core task; do not replace it with stale previous final review evidence.
- If current verification passes without code edits, the final report/transcript is still a required artifact delta.

