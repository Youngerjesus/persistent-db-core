# Implementation Brake Review: v1-deterministic-crash-matrix

## Verdict: PASS

- PM_RESULT: success
- PM_PHASE_COMPLETE: yes
- Latest brake attempt: impl_brake_exec_fresh_20260518_030658_945314_c26e1e71
- Updated At: 2026-05-17T18:11:21Z

## Scope

Review-only implementation brake between `impl_retry` and strict `impl_verify`.

Inputs inspected:
- Approved spec: `specs/v1-deterministic-crash-matrix/spec.md`
- Contract: `specs/v1-deterministic-crash-matrix/contracts.md`
- QA mapping: `specs/v1-deterministic-crash-matrix/qa_mapping.md`
- Prior implementation-brake report: this file's previous `FAIL` verdict for IB-001 through IB-003
- Latest implementation retry result: `autopilot/project_manager/tasks/task-2026-05-18-02-23-10-v1-deterministic-crash-matrix/runs/impl_retry_0_resume_20260518_030058_255588_6a8998c9/result.md`
- Current implementation artifacts: `src/storage.rs`, `tests/crash_matrix.rs`, `tests/fixtures/crash_matrix/README.md`, `scripts/verify_crash_matrix`, `docs/file_format.md`, `target/crash_matrix/crash_matrix_report.md`

Checks run during this brake pass:
- `./scripts/verify_crash_matrix`: passed
- `cargo test --test crash_matrix`: passed, 7 tests
- `./scripts/verify`: passed

Companion review reconciliation:
- `implementation-brake-reviewer`: invoked as read-only companion because the delta covers stateful/recovery behavior, but timed out before returning findings; fallback lens applied by this brake pass.
- `code-reviewer`: invoked as read-only merge-gate companion, but timed out before returning findings; fallback correctness/completeness lens applied by this brake pass.
- `performance-reviewer`: not invoked; no concrete performance trigger in this test/evidence-focused delta.

## Finding Checklist

- [resolved] IB-001
  - severity: verify-blocking
  - kind: missing behavior
  - risk category: correctness, edge/failure path, test gap
  - source attempt: impl_brake_exec_fresh_20260518_025457_435548_6bb0f4b5; code-reviewer companion accepted
  - evidence: Prior implementation started CM-005 with row 2 already durable, contradicting the contract's seed-only durable setup for recovery interruption.
  - repair target: Rework CM-005 so its evidence matches the approved recovery-interrupted-after-first-apply boundary.
  - closure evidence: `tests/crash_matrix.rs:464` starts CM-005 from `seed_committed_one`, writes committed WAL rows 2 and 3, runs `PDB_CRASH_AFTER_WAL_REPLAY_APPLIES=1`, asserts interrupt exit `Some(101)`, then verifies two successful deterministic reopens. `target/crash_matrix/crash_matrix_report.md` records seed-only setup, interrupted reopen status, and first/second recovery rows. `cargo test --test crash_matrix` passed.

- [resolved] IB-002
  - severity: verify-blocking
  - kind: verification gap
  - risk category: evidence provenance, test gap, silent failure
  - source attempt: impl_brake_exec_fresh_20260518_025457_435548_6bb0f4b5; implementation-brake-reviewer and code-reviewer companions accepted
  - evidence: Prior `scripts/verify_crash_matrix` only validated labels and global substrings, not exact per-case report content.
  - repair target: Strengthen `scripts/verify_crash_matrix` so each CM-001..CM-006 block is validated against exact expected and actual rows, evidence id, reopen command, WAL/file-format assertion, and exit status.
  - closure evidence: `scripts/verify_crash_matrix:27` through `scripts/verify_crash_matrix:104` validates exact rows, case evidence ids, reopen command, WAL assertions, exit status, CM-004 repeated-open evidence, and CM-005 interruption evidence per case. `scripts/verify_crash_matrix:112` through `scripts/verify_crash_matrix:119` adds a negative self-check. `./scripts/verify_crash_matrix` passed.

- [resolved] IB-003
  - severity: verify-blocking
  - kind: verification gap
  - risk category: evidence provenance
  - source attempt: impl_brake_exec_fresh_20260518_025457_435548_6bb0f4b5; implementation-brake-reviewer companion accepted
  - evidence: Prior generated report lacked active-run provenance, making stale evidence hard to reject in strict verification.
  - repair target: Add current-run provenance to the generated report and validate it in `scripts/verify_crash_matrix`.
  - closure evidence: `scripts/verify_crash_matrix:8` exports a generated or caller-provided `CRASH_MATRIX_RUN_ID`; `tests/crash_matrix.rs:285` through `tests/crash_matrix.rs:347` writes that run id into the report header and each case block; `scripts/verify_crash_matrix:45` through `scripts/verify_crash_matrix:49` rejects missing current run id or command provenance. `./scripts/verify_crash_matrix` regenerated and validated `target/crash_matrix/crash_matrix_report.md`.

## Must Fix Now

None. No open `verify-blocking` findings remain.

## Verify Risks

- VR-001: The deterministic replay interruption hook is production code gated by the environment variable `PDB_CRASH_AFTER_WAL_REPLAY_APPLIES` in `src/storage.rs:255`. Risk category: correctness, evidence provenance, CLI surface. Verifier question: confirm this remains acceptable as the spec's minimal crash injection hook and does not constitute a user-facing CLI contract change because normal runs without the variable are unchanged. Why not verify-blocking: the approved scope explicitly allows the minimum necessary crash injection hook, all required commands pass, and normal CLI tests pass.

## Blocked On Evidence

None.

## Blocked On Human Decision

None.

## Repair Targets

None for `impl_retry`.

## Closure Evidence

- `./scripts/verify_crash_matrix`: passed during this brake pass and regenerated current-run report evidence.
- `cargo test --test crash_matrix`: passed during this brake pass, 7 tests.
- `./scripts/verify`: passed during this brake pass, including full test suite and `db --help` smoke.
- Latest retry result reports `cargo fmt --check`, `cargo test --test wal_recovery`, and `cargo test --test cli_contract` also passed.

## Residual Risks

- Companion reviewers were invoked but did not return before timeout; this report records fallback review by the main brake pass.
- Strict `impl_verify` should still perform final acceptance/provenance verification. This PASS only means the implementation is ready to enter that phase.

## Next Action

Proceed to strict `impl_verify`.
