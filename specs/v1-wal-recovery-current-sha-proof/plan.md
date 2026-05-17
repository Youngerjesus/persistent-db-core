# Implementation Plan: v1-wal-recovery-current-sha-proof

## Goal
Regenerate deterministic WAL recovery proof at current task worktree HEAD so `req-v1-wal-recovery-proof` can close for `gate-v1-transactions-wal-recovery`.

## Non-Goals
- No new public SQL transaction commands.
- No CLI stdout, stderr, or exit-code contract changes unless an approved conflict escalation occurs.
- No file-format redesign, checkpointing strategy change, crash matrix expansion, multi-process concurrency, network service, or background daemon.
- No browser, DOM, screenshot, rendered route, or UX design-review evidence for this Rust CLI task.
- No edits to protected `ssot/` or `policies/`.

## Affected Contract Surface
| Surface | Planned Handling |
|---|---|
| CLI behavior | Preserve `db exec <path> <sql>` output and exit codes; smoke evidence proves exact behavior |
| Persisted data compatibility | Re-open existing current WAL/page format behavior; no page format change planned |
| WAL sidecar/recovery docs | Review `docs/file_format.md`; update only if current docs fail the accepted WAL semantics |
| CLI docs | Review `docs/cli_contract.md`; update only if current docs contradict current behavior |
| Tests | Use or repair `tests/wal_recovery.rs` to cover committed survival, uncommitted/incomplete absence, incomplete-tail cleanup, and deterministic corruption |
| Final evidence | Create task-scoped transcript/report with current SHA, dirty state, command output, WAL file-state, and requirement mapping |

## Implementation Boundary
1. Start by re-validating the current worktree:
   - `git rev-parse HEAD`;
   - `git status --short`;
   - existence and current content of `tests/wal_recovery.rs`, `src/storage.rs`, `docs/file_format.md`, and `docs/cli_contract.md`;
   - latest relevant review/report files from `specs/v1-transaction-wal-recovery/`.
2. Run focused WAL tests before editing:
   - `cargo test --test wal_recovery`.
3. If the focused tests pass, proceed to full verification and evidence capture.
4. If they fail, repair only the failing path in the scoped files listed by the spec. Use failing evidence to decide whether the defect is in tests, storage replay, docs, or CLI behavior.
5. Create final report/transcript under this feature directory; include all required command output or transcript references.

## Evidence Capture Plan
The final evidence report must include the following records.

| Evidence Item | Required Content |
|---|---|
| Current SHA | command `git rev-parse HEAD`; stdout must identify the current worktree SHA |
| Dirty state | command `git status --short`; stdout recorded even if empty |
| Focused WAL test | command `cargo test --test wal_recovery`; exit code; stdout/stderr summary or transcript |
| Baseline verify | command `./scripts/verify`; exit code; stdout/stderr summary or transcript |
| Create/insert smoke | command exactly matching the contract shape with temp `DB_PATH`; exit code `0`; stdout `""`; stderr `""` |
| WAL after create/insert | sidecar path, exists yes/no, byte length |
| Reopen/select smoke | command exactly matching the contract shape with same temp `DB_PATH`; exit code `0`; stderr `""`; stdout exactly `id|name\n1|ada\n2|bea\n` |
| WAL after reopen/select | sidecar path, exists yes/no, byte length |
| Uncommitted fixture rationale | direct WAL bytes are valid evidence because V1 has no public rollback/incomplete command; fixture represents observable WAL bytes at recovery boundary |
| Gate mapping | explicit links to `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, `req-v1-wal-recovery-proof` |

## CLI Smoke Commands
Use a temp path and record the literal expanded path or a redacted path plus enough sidecar metadata to audit the run.

```sh
cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"
cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"
```

Expected:
- first command: exit `0`, stdout `""`, stderr `""`;
- second command: exit `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`;
- retained complete WAL frames mean `$DB_PATH.wal` should exist and be non-empty after both commands unless implementation has changed with contract-compatible rationale.

## Verification Commands
- `cargo test --test wal_recovery`
- `./scripts/verify`
- CLI smoke commands listed above

## Stop Conditions
- Required proof cannot be produced without changing `spec.md` or `contracts.md`.
- Required proof would require adding a public rollback/incomplete transaction command.
- A fix would change public CLI output, stderr, or exit code outside approved scope.
- Protected `ssot/` or `policies/` changes appear necessary.
- Verifier rejects the phase and a second recovery attempt is needed.

