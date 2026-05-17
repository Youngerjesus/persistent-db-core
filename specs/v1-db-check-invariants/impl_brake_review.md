# Implementation Brake Review: `db check` Invariant Validation

## Verdict: PASS

Updated At: 2026-05-17T19:29:09Z

Latest outcome: `success`

The latest retry repaired the prior WAL replay-consistency false negative. The current implementation is ready to enter strict `impl_verify`: no open verify-blocking findings or human-decision blockers remain. This is verify-readiness only, not final task acceptance.

Fresh Repair Cleared: yes
Fresh Repair Required: no

## Scope

- Phase: `impl_brake_exec`
- Review mode: current-state implementation brake audit after `impl_retry_1_resume_20260518_042340_546427_3d671e78`
- Spec: `specs/v1-db-check-invariants/spec.md`
- Contract: `specs/v1-db-check-invariants/contracts.md`
- QA mapping: `specs/v1-db-check-invariants/qa_mapping.md`
- Latest implementation result: `autopilot/project_manager/tasks/task-2026-05-18-03-29-23-v1-db-check-invariants/runs/impl_retry_1_resume_20260518_042340_546427_3d671e78/result.md` (`success`, `PM_PHASE_COMPLETE: yes`)
- Current implementation surface reviewed: `src/check.rs`, `src/main.rs`, `src/storage.rs`, `src/sql.rs`, `src/lib.rs`, `tests/db_check.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`, `docs/file_format.md`
- Protected areas: no `ssot/` or `policies/` edits observed.
- Companion review: `implementation-brake-reviewer` and `code-reviewer` completed read-only passes. Performance reviewer was not invoked because the diff has no concrete performance trigger.
- Commands run by this brake pass: `cargo test --test db_check` and `scripts/verify`.

## Finding Checklist

- [resolved] `IB-001` verify-blocking behavior defect, correctness / edge-failure path, source attempt `impl_brake_exec_fresh_20260518_035837_203881_2a3e1492`.
  - Evidence: earlier brake report showed `src/check.rs` mapped page-file open/read `StorageError::Io` from `read_records_for_check` to `storage record readability`.
  - Repair target: preserve page-file `StorageError::Io` as `CheckError::OpenRead { path }`; keep actual page/header/record corruption under `storage record readability`.
  - Closure evidence: current `src/check.rs` maps `StorageError::Io` to `CheckError::OpenRead`; `tests/db_check.rs` includes exact missing path, directory path, and Unix regular-file unreadable cases; `cargo test --test db_check` passed with 11 tests.

- [resolved] `IB-002` verify-blocking behavior defect, correctness / evidence provenance, source attempt `impl_brake_exec_fresh_20260518_035837_203881_2a3e1492`.
  - Evidence: earlier brake report showed `src/sql.rs` mapped all SQL `decode_record` failures to `catalog/record invariant`.
  - Repair target: split structural persisted-byte decode failures from logical catalog/row invariant failures, or return the documented `storage record readability` label for decode-impossible persisted bytes; add a focused fixture.
  - Closure evidence: current `src/sql.rs` maps `decode_record` errors to `storage record readability`; `tests/db_check.rs` corrupts the SQL logical record kind byte and asserts the storage readability label; `cargo test --test db_check` passed.

- [resolved] `IB-003` verify-blocking verification gap, test gap, source attempt `impl_brake_exec_fresh_20260518_035837_203881_2a3e1492`.
  - Evidence: earlier brake report showed focused negative-path tests did not pin exact exit code and documented open/read stderr shape strongly enough to catch `IB-001`.
  - Repair target: strengthen `tests/db_check.rs` helpers/cases to assert exit code `1` for every negative `db check` case and exact documented open/read stderr shape for missing and directory paths.
  - Closure evidence: current `tests/db_check.rs` helpers assert exit code `1`, empty stdout, exact invariant stderr, and exact open/read stderr; focused and baseline verification passed.

- [resolved] `IB-004` verify-risk verification gap, test gap / regression, source attempt `impl_brake_exec_fresh_20260518_035837_203881_2a3e1492`.
  - Evidence: earlier brake report noted malformed `db check` arity was not directly covered by CLI-contract tests.
  - Repair target: verifier judgment requested on whether to require a narrow arity assertion.
  - Closure evidence: current `tests/cli_contract.rs` includes `check_requires_path_argument`; `scripts/verify` passed with `tests/cli_contract.rs` reporting 7 passed.

- [resolved] `IB-005` verify-blocking behavior defect, correctness / WAL replay consistency, source attempt `impl_brake_exec_fresh_20260518_040738_412901_a7a93965`.
  - Evidence: earlier brake report showed `validate_wal_for_check` compared every complete committed page-append frame against the original durable count and rejected valid chained retained WAL frames.
  - Repair target: track a virtual current record count while scanning complete committed page-append frames.
  - Closure evidence: current `src/storage.rs:122-190` tracks `virtual_record_count`; `tests/db_check.rs:235-257` adds `check_valid_chained_retained_wal_frames_pass`; `cargo test --test db_check` passed with 11 tests.

- [resolved] `IB-006` verify-blocking verification gap, test gap / evidence provenance, source attempt `impl_brake_exec_fresh_20260518_040738_412901_a7a93965`.
  - Evidence: earlier brake report showed there was no positive retained-WAL sidecar fixture with chained complete committed frames.
  - Repair target: add a focused `tests/db_check.rs` success fixture for a valid retained WAL sidecar with at least two complete committed `0x01` page-append frames whose `record_count_before` values chain correctly.
  - Closure evidence: current `tests/db_check.rs:235-257` writes two chained committed page-append frames and asserts exact success stdout/stderr; `cargo test --test db_check` passed with 11 tests.

- [resolved] `IB-007` verify-blocking behavior defect, correctness / WAL replay consistency / persisted-data integrity, source attempt `impl_brake_exec_fresh_20260518_041509_379319_511ba89c`.
  - Evidence: earlier companion code-review reproduced a count-valid committed page-append WAL frame with a one-byte-too-large payload: `db check` returned success while normal replay failed with `RecordTooLarge`.
  - Repair target: make `validate_wal_for_check` validate each complete committed `0x01` page-append payload against the same appendability size constraint used by replay, without mutating the page file or sidecar.
  - Closure evidence: current `src/storage.rs:172-178` calls `validate_wal_page_append_payload` before advancing virtual replay count, and `src/storage.rs:193-198` rejects payloads larger than `max_record_payload_len`; `cargo test --test db_check` passed with 11 tests.

- [resolved] `IB-008` verify-blocking verification gap, test gap / regression, source attempt `impl_brake_exec_fresh_20260518_041509_379319_511ba89c`.
  - Evidence: earlier brake report showed no negative fixture for a count-valid but unreplayable committed WAL payload.
  - Repair target: add a focused `tests/db_check.rs` fixture for a complete committed page-append WAL frame whose `record_count_before` equals the current durable record count but whose payload cannot be appended by page storage.
  - Closure evidence: current `tests/db_check.rs:351-369` writes a count-valid complete committed WAL frame with `MAX_RECORD_PAYLOAD_LEN + 1` payload bytes and asserts exact `wal replay consistency` failure; `cargo test --test db_check` passed with 11 tests.

- [resolved] `IB-009` verify-blocking verification gap, evidence provenance / reviewer SSOT, source attempt `impl_brake_exec_fresh_20260518_042510_679046_c3d8c953`.
  - Evidence: implementation-brake companion reported that the latest `impl_brake_review.md` still listed `IB-007` and `IB-008` as open even though the live tree had the repair and tests.
  - Repair target: refresh `specs/v1-db-check-invariants/impl_brake_review.md` so open blockers and closure evidence match the live tree.
  - Closure evidence: this report now records `IB-007` and `IB-008` as resolved with current code/test references and fresh command evidence.

## Must Fix Now

- None.

## Verify Risks

- `VR-001` evidence provenance: the latest implementation result file for `impl_retry_1_resume_20260518_042340_546427_3d671e78` contains only `success` and `PM_PHASE_COMPLETE: yes`. The implementation run transcript includes command evidence notes, and this brake pass reran both required commands successfully. Verifier question: can `impl_verify` independently bind acceptance evidence to the current worktree from fresh command output and artifacts? This is not verify-blocking because strict verification remains executable and this brake report records fresh evidence.
- `VR-002` WAL replay semantics: after repair, `impl_verify` should still inspect that `db check` remains read-only and does not append to the page file, truncate incomplete WAL tails, or checkpoint retained complete frames while validating WAL replay consistency. This is not verify-blocking because the current implementation uses read-only WAL byte scanning in `validate_wal_for_check`, and required tests pass.

## Blocked On Evidence

- None.

## Blocked On Human Decision

- None.

## Repair Targets

- None for `impl_retry`.

## Closure Evidence

- `cargo test --test db_check`: passed in this brake worktree, 11 tests passed.
- `scripts/verify`: passed in this brake worktree, including `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help`.
- Companion `code-reviewer`: no findings; confirmed prior WAL blockers are closed by `src/storage.rs` and `tests/db_check.rs`.
- Companion `implementation-brake-reviewer`: no live code-path or test defect; identified stale reviewer SSOT as the remaining blocker, now resolved by this report refresh.

## Residual Risks

- Strict `impl_verify` still owns final acceptance/provenance verification. In particular, it should inspect that required fixture generation is test-local, the WAL sidecar fixture uses the documented complete committed `0x01` page-append frame shape, and durable docs match the exact CLI contract.
- UI, DOM, screenshot, rendered route state, and UX design review remain non-applicable for this CLI-only task.

## Next Action

Proceed to strict `impl_verify`.
