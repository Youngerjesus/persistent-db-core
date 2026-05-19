# Implementation Brake Review: v1-secondary-index-mutation-consistency

Verdict: PASS

Updated At: 2026-05-19T14:26:31+0900

## Scope

- Phase: `impl_brake_exec` fresh pass after `impl_retry_1_resume_20260519_141529_326364_c79f916e`.
- Reviewed artifacts: `spec.md`, `contracts.md`, `qa_mapping.md`, current worktree diff, prior `impl_brake_review.md`, latest implementation retry result/final, and companion reviewer outputs.
- Reviewed implementation surface: `src/sql.rs`, `src/index.rs`, `tests/secondary_index.rs`, `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md`.
- Commands run in this brake pass:
  - `cargo test --test secondary_index -- --nocapture`: exit 0; 29 passed, including mutation update/delete restart, retained WAL sidecar, WAL-only `U`/`D` replay, no-op build-id regression, missing-target validation, and three `db check` negative fixture tests.
  - `./scripts/verify`: exit 0; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help` passed.

Fresh Repair Required: no

## Finding Checklist

- IB-001
  - Status: resolved
  - Severity: verify-blocking
  - Kind: behavior defect
  - Risk category: correctness, edge/failure path, test gap
  - Source attempt: `impl_brake_exec_resume_20260519_141229_120417_3e015600`, accepted from prior code-review companion and local code review.
  - Evidence: prior report found `UPDATE` returned a missing-target no-op before validating the `SET` column and value type.
  - Repair target: Move `UPDATE` set-column existence and value-type validation before missing-target no-op; add black-box tests.
  - Closure evidence: `src/sql.rs` now validates the primary-key predicate, set-column existence, primary-key update prohibition, and value type before `existing.primary_index.get(key)` can return `Ok(0)`. `tests/secondary_index.rs::update_validates_set_column_and_type_before_missing_target_noop` covers missing set column and type mismatch with absent target. `cargo test --test secondary_index -- --nocapture` and `./scripts/verify` passed in this brake pass.

- IB-002
  - Status: resolved
  - Severity: verify-blocking
  - Kind: behavior defect
  - Risk category: regression, persisted-data contract, evidence provenance
  - Source attempt: `impl_brake_exec_resume_20260519_141229_120417_3e015600`, accepted from prior code-review companion and local code review.
  - Evidence: prior report found `logical_record_count` advanced for no-op `UPDATE`/`DELETE`, which could inflate subsequent same-command `CREATE INDEX` build ids.
  - Repair target: Increment `logical_record_count` only when `U` or `D` is appended; add same-command no-op mutation plus `CREATE INDEX` build-id regression coverage.
  - Closure evidence: `Statement::Update` and `Statement::Delete` now add the returned append count from `execute_update` and `execute_delete`; missing targets return `Ok(0)`. `tests/secondary_index.rs::noop_mutations_do_not_advance_same_command_create_index_build_id` decodes the committed `X` metadata and asserts build id `2`. `cargo test --test secondary_index -- --nocapture` and `./scripts/verify` passed in this brake pass.

- IB-003
  - Status: resolved
  - Severity: verify-blocking
  - Kind: verification gap
  - Risk category: test gap, evidence provenance, recovery path
  - Source attempt: `impl_brake_exec_resume_20260519_141229_120417_3e015600`, accepted from prior code-review companion.
  - Evidence: prior report found retained-WAL evidence did not force replay of mutation records from WAL-only committed frames.
  - Repair target: Add a replay-required fixture where committed `U` and `D` logical records exist only in the WAL sidecar ahead of the page file, then assert the required range query and `db check` success through separate `db` process invocations.
  - Closure evidence: `tests/secondary_index.rs::mutation_contract_wal_only_update_delete_frames_replay_secondary_indexes` appends committed WAL-only `U` and `D` frames, then reopens through CLI query and `db check`. `cargo test --test secondary_index -- --nocapture` and `./scripts/verify` passed in this brake pass.

- IB-004
  - Status: open
  - Severity: verify-risk
  - Kind: decision gap
  - Risk category: performance, resource growth, maintainability
  - Source attempt: `impl_brake_exec_resume_20260519_141229_120417_3e015600`, carried forward by both companions.
  - Evidence: `DELETE` uses stable tombstone row slots, so reopen, `db check`, and index rebuild iterate historical row slots while skipping tombstones. Current coverage exercises fixed small fixtures.
  - Repair target: None for this brake gate. Verifier question: is history-sized reopen/check/index-build cost after repeated mutations an accepted V1 limitation of the stable-row-slot design?
  - Closure evidence: pending verifier judgment or future compaction/checkpoint design.

- IB-005
  - Status: open
  - Severity: verify-risk
  - Kind: verification gap
  - Risk category: regression, test gap
  - Source attempt: `impl_brake_exec_fresh_20260519_141859_564654_9568d121`, accepted from implementation-brake companion as non-blocking.
  - Evidence: QA mapping includes unsupported mutation-breadth regression expectations. Current tests pin missing set column/type and no-op build-id behavior, but do not explicitly pin wrong-predicate `UPDATE`/`DELETE` or primary-key-column `UPDATE` in black-box tests.
  - Repair target: Optional follow-up tests for `UPDATE users SET age = 30 WHERE age = 10;`, `UPDATE users SET id = 9 WHERE id = 1;`, and `DELETE FROM users WHERE age = 10;`.
  - Closure evidence: not required for verify-readiness because the companion spot-check found current behavior rejects these forms deterministically and strict `impl_verify` remains executable.

- IB-006
  - Status: open
  - Severity: verify-risk
  - Kind: verification gap
  - Risk category: evidence provenance, contract compliance
  - Source attempt: `impl_brake_exec_fresh_20260519_141859_564654_9568d121`, accepted from implementation-brake companion as non-blocking.
  - Evidence: latest `impl_retry` final artifact summarizes repairs and commands but does not include the full per-command stdout/stderr summaries and explicit `REQ-7-insert-update-and-delete-must-997871f9` / `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` mapping requested by the contract.
  - Repair target: Strict `impl_verify` or final reporting should record full command evidence and requirement/evidence ID mapping.
  - Closure evidence: This brake report records fresh command status and maps the relevant tests to the requirement/evidence IDs below, so verify remains executable; no product-code retry is required.

## Must Fix Now

- None. Prior verify-blocking findings IB-001, IB-002, and IB-003 are resolved with code/test evidence.

## Verify Risks

- IB-004: Stable tombstone slots make mutation-heavy reopen/check/index-build cost proportional to historical row slots. This is not verify-blocking because the approved spec uses stable row positions and has no compaction requirement.
- IB-005: Unsupported mutation-breadth boundaries are not fully pinned by automated black-box tests. This is not verify-blocking because the current implementation rejects the spot-checked forms and the contract fixture paths are covered.
- IB-006: The latest implementation retry final artifact is terse. This is not verify-blocking because the required commands were rerun in this brake pass, the brake report records command outcomes, and strict `impl_verify` can independently regenerate final acceptance evidence.

## Blocked On Evidence

- None.

## Blocked On Human Decision

- None for verify-readiness. IB-004 should be treated as a verifier/product tradeoff note, not a blocker for entering `impl_verify`.

## Repair Targets

- None for `impl_retry`.

## Closure Evidence

- `cargo test --test secondary_index -- --nocapture`: exit 0; 29 tests passed. Relevant tests include:
  - `mutation_contract_update_delete_restart_processes_keep_secondary_indexes_consistent`
  - `mutation_contract_retained_wal_replay_keeps_secondary_indexes_consistent`
  - `mutation_contract_wal_only_update_delete_frames_replay_secondary_indexes`
  - `update_validates_set_column_and_type_before_missing_target_noop`
  - `noop_mutations_do_not_advance_same_command_create_index_build_id`
  - `mutation_contract_db_check_rejects_stale_secondary_entry_after_update`
  - `mutation_contract_db_check_rejects_dangling_secondary_pointer_after_delete`
  - `mutation_contract_db_check_rejects_missing_visible_secondary_entry`
- `./scripts/verify`: exit 0; baseline fmt, clippy, full test suite, and help smoke check passed.
- Requirement mapping:
  - `REQ-7-insert-update-and-delete-must-997871f9`: covered by update/delete exact-output process-boundary tests, WAL-only replay test, and no-op/accounting regression coverage.
  - `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`: covered by positive `db check` after mutation/replay and the stale secondary entry, dangling pointer, and missing visible indexed row negative fixtures.
- Storage compatibility outcome: page framing is unchanged; mutation records are additive SQL logical records (`U` and `D`). Docs record existing row-only and existing secondary-index database reopen compatibility.

## Residual Risks

- The code-review companion reported no merge-gate blocker and confirmed prior blockers as closed.
- The implementation-brake companion reported the executable behavior and replay coverage ready for strict verify, but raised evidence/SSOT freshness concerns. The stale brake SSOT concern is resolved by this report refresh. The implementation final-artifact terseness is carried as IB-006 verify-risk rather than a retry target because strict verification remains executable and this brake pass reran the required commands.
- No `tests/db_check.rs` focused command is required by the contract because negative secondary-index mutation fixtures remain in `tests/secondary_index.rs`.

## Next Action

Advance to strict `impl_verify`. Success here only means verify-readiness; it is not final task completion or acceptance proof.
