# Tasks: v1-transaction-wal-recovery

## Execution Rules
- Follow `spec.md` and `contracts.md` as frozen inputs.
- Do not edit `ssot/` or `policies/`.
- Keep changes scoped to WAL recovery evidence and directly required integration.
- Prefer std-only implementation; do not add third-party crates unless escalated.
- Preserve existing CLI output, stderr, and exit-code behavior unless the approved contract explicitly requires a change.

## Task List

### T1. Add WAL recovery tests
Status: ready

Files:
- `tests/wal_recovery.rs`

Details:
- Reuse existing test style from `tests/sql_exec.rs`: temp path helper, `Command::new(env!("CARGO_BIN_EXE_db"))`, exact stdout/stderr assertions, cleanup.
- Add Scenario A as separate child-process create/insert and reopen/select commands.
- Add Scenario B as a deterministic incomplete WAL fixture verified through CLI. Create the catalog through CLI, write `<db-path>.wal` directly, then select through CLI.
- Include a test name or comment explaining that WAL bytes are fixture-authored because V1 has no public rollback or incomplete transaction command.
- Include repeated reopen/select assertion to catch duplicate replay from retained WAL frames.

Subtasks:
- T1.1 Add `committed_wal_replay_survives_reopen_via_cli` or equivalent exact-output test.
- T1.2 Add `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` or equivalent fixture test.
- T1.3 Add fixture helpers for the frozen WAL layout: magic `PDBWAL1\0`, version `1`, `u64` frame id, `u64 record_count_before`, state byte, payload-kind byte, `u32` payload length, `u32` wrapping byte-sum checksum, and payload bytes.
- T1.4 Add local SQL row logical-record encoder for table `users` values `1|ada` and `9|ghost`, matching the existing SQL fixture style.

Acceptance evidence:
- `cargo test --test wal_recovery` passes.

### T2. Introduce minimal WAL framing and replay
Status: ready

Files:
- `src/storage.rs`
- `src/lib.rs`
- optional new `src/wal.rs` or `src/recovery.rs`

Details:
- Implement a deterministic WAL sidecar path helper.
- Implement the frozen frame encode/decode layout from `design.md`.
- Implement replay before normal SQL record loading.
- Ensure complete committed frames apply exactly once using page-store record-count checkpointing, and incomplete or rollback frames do not expose rows.

Subtasks:
- T2.1 Define WAL constants: magic `PDBWAL1\0`, version `1`, state `0x01` committed, state `0x02` rollback, payload kind `0x01` page-store record append, fixed header length `36`.
- T2.2 Implement frame writer and reader with explicit incomplete-tail handling.
- T2.3 Implement replay idempotence through `record_count_before`: apply when current count matches, skip when current count is greater, error when current count is lower.
- T2.4 Keep existing page file validation behavior intact for files without WAL.
- T2.5 Retain WAL frames after apply; do not introduce a checkpoint sidecar, applied marker record, or truncation strategy for this task.

Acceptance evidence:
- Scenario B test proves ghost row absence.
- Existing storage and SQL tests remain green.

### T3. Wire SQL mutations through WAL-aware append
Status: ready

Files:
- `src/sql.rs`
- possibly `src/storage.rs` or new WAL module

Details:
- Replace direct mutation append calls in create-table and insert paths with a WAL-aware append helper.
- Update in-memory `Database` state only after durable append succeeds.
- Preserve semantic validation ordering so failed statements do not create committed WAL entries.
- Preserve existing `db exec` stdout/stderr/exit behavior.

Subtasks:
- T3.1 Route catalog record append through WAL-aware helper.
- T3.2 Route row record append through WAL-aware helper.
- T3.3 Add or verify test coverage that failed mid-command statements do not append committed ghost rows.
- T3.4 Verify primary-key duplicate failures still happen before row append.

Acceptance evidence:
- `cargo test` and `cargo test --test wal_recovery` pass.

### T4. Update WAL file-format documentation
Status: ready

Files:
- `docs/file_format.md`
- `docs/cli_contract.md` only if public CLI behavior changes

Details:
- Add a WAL compatibility note covering filename/location, frame layout/framing, replay order, committed/rollback/incomplete handling, and old database file behavior.
- Confirm `docs/cli_contract.md` remains unchanged if there is no public behavior change; mention this in the final report.

Subtasks:
- T4.1 Document exact WAL sidecar path.
- T4.2 Document exact frame fields and validation.
- T4.3 Document replay order and page-store record-count idempotence behavior, including retained WAL frames.
- T4.4 Document that old page files without WAL open normally.

Acceptance evidence:
- Manual doc review in final report.
- `./scripts/verify` passes.

### T5. Run verification and record implementation evidence
Status: ready

Files:
- implementation phase result/report path

Details:
- Run required verification commands:
  - `cargo test`
  - `cargo test --test wal_recovery`
  - `./scripts/verify`
- Run required smoke commands with a temp path and record output summaries.
- Include WAL file-state evidence summary and temp path redaction status.
- Map every acceptance criterion to tests, docs, command output, or blocker.

Subtasks:
- T5.1 Capture command outputs after implementation.
- T5.2 If one repair pass is needed, repair within phase.
- T5.3 If a second recovery attempt becomes necessary after verifier rejection, escalate per contract.

Acceptance evidence:
- Final implementation report connects `tests/wal_recovery.rs`, `docs/file_format.md`, optional `docs/cli_contract.md`, verification output, smoke output, and WAL file-state evidence.

## Dependency Order
1. T1
2. T2
3. T3
4. T4
5. T5

## Readiness Notes
- No human decision is required before implementation.
- Highest-risk implementation detail is replay idempotence; tests should fail before production code changes.
