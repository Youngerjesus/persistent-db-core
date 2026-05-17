# Implementation Brake Review: v1-transaction-wal-recovery

Verdict: PASS

Updated At: 2026-05-18 00:32:36 KST

## Scope

Phase: Implementation Brake Execution retry pass for `impl_brake_exec_fresh_20260518_002705_421169_1956ba3d`.

Reviewed the approved `spec.md`, `contracts.md`, `qa_mapping.md`, current worktree diff, `src/storage.rs`, `tests/wal_recovery.rs`, `docs/file_format.md`, `docs/cli_contract.md`, prior brake report, and latest implementation retry result at `autopilot/project_manager/tasks/task-2026-05-17-23-45-17-v1-transaction-wal-recovery/runs/impl_retry_0_resume_20260518_002405_126333_a6524445/`.

This brake pass is review-only. It did not repair production code, tests, durable docs, or task documents beyond this implementation-brake report and the current phase result file.

Companion review:
- `implementation-brake-reviewer`: completed; provenance concern accepted as a verify-risk and closed for brake-readiness by current-run evidence below.
- `code-reviewer`: completed; incomplete-tail append and CLI doc scope concerns accepted as verify-risks, Scenario A critique rejected as blocking because Scenario B directly exercises committed WAL replay from fixture-authored WAL.
- `performance-reviewer`: not invoked because the diff has no concrete performance trigger beyond small local file replay over current WAL bytes.

Current read-only verification evidence gathered during this brake pass:
- `cargo test --test wal_recovery`: pass, 3 tests passed.
- `cargo test`: pass, all current unit/integration/doc tests passed.
- `./scripts/verify`: pass, including baseline format, clippy, test, and help checks.
- Direct compiled CLI smoke with redacted temp DB path: create/insert exit `0`, stdout `""`, stderr `""`; reopen select exit `0`, stdout exactly `id|name\n1|ada\n2|bea\n`, stderr `""`; retained WAL sidecar exists with size `202` bytes.
- `git diff --check`: pass.

## Finding Checklist

- [resolved] IB-WAL-001
  - Severity: verify-blocking
  - Kind: behavior defect
  - Risk category: correctness, recovery semantics, contract drift
  - Source attempt: `impl_brake_exec_fresh_20260518_001634_010257_bc5484d7`, code-reviewer companion and main reconciliation
  - Evidence: Prior pass found `src/storage.rs` silently stopped replay when a committed frame's `record_count_before` was ahead of the current durable page-store count, conflicting with frozen design/tasks and `docs/file_format.md`.
  - Repair target: Change replay so `current_record_count < record_count_before` returns deterministic storage/recovery error; add deterministic WAL fixture test; realign `docs/file_format.md`.
  - Closure evidence: `src/storage.rs:223-230` now returns `StorageError::CorruptRecordLength` for ahead-of-store frames. `tests/wal_recovery.rs:187-215` adds `committed_wal_frame_ahead_of_page_store_fails_deterministically`. `docs/file_format.md:99-105` documents ahead-of-count as deterministic storage corruption. `cargo test --test wal_recovery`, `cargo test`, and `./scripts/verify` pass in this brake pass.

- [open] IB-WAL-002
  - Severity: verify-risk
  - Kind: verification gap
  - Risk category: evidence provenance, CLI wrapper semantics
  - Source attempt: `impl_brake_exec_fresh_20260518_001634_010257_bc5484d7`, main brake pass and retry companion
  - Evidence: Required smoke commands mention `cargo run --bin db -- exec ...`, but literal Cargo invocation may emit Cargo wrapper lines on stderr. The compiled `target/debug/db` smoke and `tests/wal_recovery.rs` exact process assertions show the actual `db` process has empty stderr for Scenario A.
  - Verifier question: Should strict `impl_verify` record smoke evidence from the compiled CLI binary, `cargo run --quiet --bin db -- exec ...`, or explicitly separate Cargo wrapper stderr from CLI stderr?
  - Why not verify-blocking: Verification remains executable, and task tests assert exact `db` process stdout/stderr through the Cargo test harness.
  - Repair target: none for implementation; verifier should make smoke command provenance explicit.
  - Closure evidence: pending verifier judgment.

- [open] IB-WAL-003
  - Severity: verify-risk
  - Kind: verification gap
  - Risk category: test gap, recovery semantics
  - Source attempt: `impl_brake_exec_fresh_20260518_001634_010257_bc5484d7`, implementation-brake-reviewer companion
  - Evidence: `docs/file_format.md:99-111` documents rolled-back frames as skipped, and `src/storage.rs:232-233` implements skip for rollback state, but `tests/wal_recovery.rs:155-184` covers incomplete-tail ghost exclusion rather than a complete rollback frame. Scenario B allows rollback or incomplete evidence.
  - Verifier question: Should `impl_verify` require a direct rollback-frame fixture because the docs explicitly document rollback skip behavior, or accept the approved Scenario B incomplete-tail proof?
  - Why not verify-blocking: The approved spec and contract require rollback or incomplete proof; the current test satisfies the incomplete branch with exact CLI output and repeated reopen idempotence.
  - Repair target: optional focused rollback-frame fixture if verifier wants direct coverage of the documented branch.
  - Closure evidence: pending verifier judgment.

- [open] IB-WAL-004
  - Severity: verify-risk
  - Kind: edge/failure path verification gap
  - Risk category: correctness, storage I/O failure path
  - Source attempt: `impl_brake_exec_fresh_20260518_001634_010257_bc5484d7`, code-reviewer companion with main reconciliation
  - Evidence: `src/storage.rs:69-74` writes a committed WAL frame before appending the page-store record, and replay applies matching committed frames at `src/storage.rs:223-230`. A low-level page-file append error after a successful WAL write could make a mutation visible on a later reopen even if the original caller saw an error.
  - Verifier question: Is WAL frame persistence the intended commit point for V1 storage I/O failures, or must caller-visible append failure imply no future durable row?
  - Why not verify-blocking: The frozen design says mutation sites should record intent, commit the WAL frame, then apply the page-store append. QA mapping's failed-statement coverage refers to semantic validation and duplicate primary-key failures before append, not injected filesystem failures.
  - Repair target: if verifier treats caller-visible I/O failure as rollback, add failure injection and a compensating or rollback terminal state design; otherwise record WAL write as the V1 commit point.
  - Closure evidence: pending verifier judgment.

- [open] IB-WAL-005
  - Severity: verify-risk
  - Kind: decision gap
  - Risk category: architecture drift, documentation scope
  - Source attempt: `impl_brake_exec_fresh_20260518_001634_010257_bc5484d7`, implementation-brake-reviewer and code-reviewer companion with main reconciliation
  - Evidence: `docs/cli_contract.md:76-79` now documents durability across later `db exec` starts and `docs/cli_contract.md:161-163` removes WAL/recovery from the non-goal line. `contracts.md` expected `docs/cli_contract.md` to remain unchanged unless public CLI behavior changed.
  - Verifier question: Is the narrow CLI contract update acceptable because the old non-goal text would be false after adding WAL/recovery support, or should the implementation revert `docs/cli_contract.md` and report no public output/exit/stderr/command-surface change?
  - Why not verify-blocking: The changed text does not alter stdout, stderr, exit codes, or supported commands; it documents durability and removes now-stale WAL non-goal wording. Strict verification can still judge the product-doc scope.
  - Repair target: if verifier rejects the doc scope, revert the CLI contract delta and state in the final report that public output/exit/stderr/command grammar did not change.
  - Closure evidence: pending verifier judgment.

- [open] IB-WAL-006
  - Severity: verify-risk
  - Kind: behavior defect
  - Risk category: recovery semantics, edge/failure path
  - Source attempt: `impl_brake_exec_fresh_20260518_002705_421169_1956ba3d`, code-reviewer companion
  - Evidence: `replay_wal()` stops at an incomplete trailing header or payload (`src/storage.rs:187-214`). Later `append_wal_frame()` appends at EOF after those incomplete bytes, and `next_wal_frame_id()` also stops counting at the incomplete frame. A later crash after WAL append but before page append could leave a committed frame after an ignored incomplete tail that replay will never reach.
  - Verifier question: Does V1 need to truncate or otherwise supersede an incomplete tail before appending future WAL frames, or is this append-after-incomplete-tail crash case outside the current minimal proof slice?
  - Why not verify-blocking: The approved Scenario B requires incomplete-tail absence and repeated reopen idempotence, which are covered. The companion scenario is a later write after an already-corrupt/incomplete tail plus another failure window; it is important but beyond the canonical Scenario A/B acceptance path and does not make strict verification non-executable.
  - Repair target: future recovery hardening should truncate incomplete tails before appending or add a test that appending after an incomplete tail keeps later WAL frames replayable.
  - Closure evidence: pending verifier judgment.

- [rejected] IB-WAL-007
  - Severity: verify-risk proposed by companion; rejected as verify-blocking
  - Kind: verification gap
  - Risk category: test gap
  - Source attempt: `impl_brake_exec_fresh_20260518_002705_421169_1956ba3d`, code-reviewer companion
  - Evidence: Companion noted Scenario A's CLI reopen test can pass from page-store durability without applying WAL frames.
  - Repair target: none for brake-readiness.
  - Closure evidence: Rejected as a brake blocker because Scenario B fixture authors a committed WAL frame for `1|ada` after the page file contains only the catalog record, then verifies CLI select returns `id|name\n1|ada\n`. That path directly exercises committed WAL replay. Scenario A remains the required CLI-visible completion proof and Scenario B supplies fixture-level replay proof.

## Must Fix Now

None. No open `verify-blocking` finding remains.

## Verify Risks

- IB-WAL-002: Cargo wrapper stderr can contaminate literal `cargo run --bin db -- exec ...` smoke evidence; strict verifier should make command provenance explicit.
- IB-WAL-003: Complete rollback-frame skip is documented and implemented but not directly tested; current required Scenario B is satisfied through incomplete-tail ghost exclusion.
- IB-WAL-004: WAL frame persistence is the apparent commit point for low-level page append failures after WAL write; verifier should decide whether this is acceptable V1 semantics or requires failure-injection repair.
- IB-WAL-005: `docs/cli_contract.md` changed despite derived artifacts expecting no CLI contract update unless public output/exit/stderr/commands changed; verifier should decide whether the narrow durability documentation update is acceptable.
- IB-WAL-006: Future appends after an ignored incomplete WAL tail may leave later committed frames behind unreadable bytes; verifier should decide whether this is outside the minimal Scenario A/B slice or a required recovery hardening.

## Blocked On Evidence

None.

## Blocked On Human Decision

None.

## Repair Targets

None for `impl_retry`. Remaining items are verify-risk questions for strict `impl_verify`.

## Closure Evidence

Closed verify-blocking repair:
- `src/storage.rs:223-230` returns deterministic `StorageError::CorruptRecordLength` when a committed WAL frame is ahead of the durable page-store record count.
- `tests/wal_recovery.rs:187-215` covers ahead-of-store WAL failure.
- `docs/file_format.md:99-105` documents the corrected ahead-of-count behavior.

Current command evidence:
- `cargo test --test wal_recovery`: pass, 3 tests.
- `cargo test`: pass, all current tests.
- `./scripts/verify`: pass.
- Direct compiled CLI smoke: create/insert exit `0`, stdout/stderr empty; select exit `0`, stdout exactly `id|name\n1|ada\n2|bea\n`, stderr empty; WAL sidecar exists and size is `202` bytes.
- `git diff --check`: pass.

Artifact evidence:
- `tests/wal_recovery.rs` exists and covers Scenario A, Scenario B incomplete-tail absence, repeated reopen idempotence, WAL sidecar retention, and ahead-of-store corruption.
- `docs/file_format.md` includes WAL sidecar path, frame layout, replay order, committed/rollback/incomplete handling, retained WAL/idempotence behavior, and existing-file compatibility.
- `docs/cli_contract.md` changed only in durable behavior documentation/non-goal wording; it does not add public commands or change stdout/stderr/exit code contract.

## Residual Risks

- No performance companion was invoked; current replay reads the whole WAL file into memory, acceptable for this minimal evidence slice but not a long-term large-WAL design.
- Multi-process concurrency remains explicitly out of scope.
- Complete rollback-frame coverage is indirect because the approved contract allowed rollback or incomplete proof.
- Current-run implementation retry evidence was terse; this brake report records current-run verification evidence for strict verifier handoff.

## Next Action

PM_RESULT: success. Proceed to strict `impl_verify`; success here only means no open verify-blocking brake finding remains.
