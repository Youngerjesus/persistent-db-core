# QA Mapping: v1-deterministic-crash-matrix

## Scope

Phase: QA Prep Execution. This manifest maps every task in `tasks.md` to pre-implementation QA coverage. Application/core implementation files are intentionally out of scope for this phase.

Evidence-heavy classification: yes. Acceptance depends on generated crash matrix evidence, a current-run markdown report, and command evidence, not only static code checks.

## Provenance Contract

- Evidence root: `target/crash_matrix/`
- Required artifacts:
  - `target/crash_matrix/crash_matrix_report.md`
  - per-case current-run observations for CM-001..CM-006, if the implementation chooses intermediate machine-readable records
  - command output evidence for `cargo test --test crash_matrix`, `./scripts/verify_crash_matrix`, and `./scripts/verify`
- Scenario/evidence ids: `crash-matrix-case-CM-001`, `crash-matrix-case-CM-002`, `crash-matrix-case-CM-003`, `crash-matrix-case-CM-004`, `crash-matrix-case-CM-005`, `crash-matrix-case-CM-006`
- Current-run id source: scheduler run id `qa_prep_exec_fresh_20260518_024253_590223_73d06571` for this prep pass; implementation and verification passes must stamp or derive their evidence from the active scheduler run that executed the commands.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass is deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: `target/crash_matrix/crash_matrix_report.md` and any per-case observation files are not valid current proof unless produced by the same run that executed the required verification commands.
- Writer/validator separation expectation: the crash matrix test harness may write observed case results, but `scripts/verify_crash_matrix` must independently validate that all required case ids, evidence ids, actual rows, expected rows, reopen commands, WAL/file-format assertions, and exit statuses are present in the generated report.
- Redaction target list: no secrets are expected. Do not include absolute temp database paths, user home paths, environment variables, or full stderr containing machine-specific paths in the durable markdown report unless needed to diagnose a failure; use `<db_path>` in reopen command fields.

## Scenario Expansion Lens

- Invalid input: unsupported `ORDER BY` must not be used in tests because current CLI contract rejects it; use deterministic `SELECT * FROM items;` with a primary-key table.
- Empty or partial state: CM-001 covers absent/empty WAL; CM-002 covers incomplete WAL header/payload; CM-006 covers committed prefix followed by incomplete/invalid-length tail.
- Duplicate or already-done action: CM-004 and CM-005 require repeated reopen checks so committed WAL replay is idempotent and does not duplicate rows.
- Dependency failure: `scripts/verify_crash_matrix` must fail when `cargo test --test crash_matrix` fails, when the report is missing, or when any required identifier/field is absent.
- Permission/trust boundary: fixtures and reports must stay local to `tests/fixtures/crash_matrix/` and `target/crash_matrix/`; no protected `ssot/` or `policies/` edits.
- Retry/re-entry: fresh verification must regenerate current-run evidence and must not reuse a previous `target/crash_matrix/crash_matrix_report.md`.

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| T001 | QA mapped | preflight, file existence | `qa_mapping.md` | `git status --short`; `git rev-parse HEAD`; `test -f src/storage.rs tests/wal_recovery.rs docs/file_format.md scripts/verify` | Latest HEAD and expected files recorded before implementation. | Current prep observed HEAD `358854464059ba41ed2f232cfc6ab17a9ce51dac`; worktree has untracked spec package artifacts. |
| T002 | QA mapped | canonical input guard | `qa_mapping.md` | `git diff -- specs/v1-deterministic-crash-matrix/spec.md specs/v1-deterministic-crash-matrix/contracts.md` | No incompatible spec/contract mutation during implementation. | QA prep does not edit `spec.md` or `contracts.md`. |
| T003 | QA mapped | regression smoke | `tests/wal_recovery.rs` | `cargo test --test wal_recovery` | Existing WAL recovery regression test still passes before and after implementation. | Preserves current uncommitted, incomplete-tail, and retained sidecar coverage. |
| T004 | Red scaffolded | integration harness shape | `tests/crash_matrix.rs` | `cargo test --test crash_matrix` | Shared helpers execute CLI reopen, temp DB, WAL path, stdout/stderr capture, cleanup, report case data. | Red scaffold currently fails until the executable case harness replaces scaffold failures with observed assertions. |
| T005 | Red scaffolded | CM-001 scenario | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md` | `cargo test --test crash_matrix cm_001_pre_wal_append_seed_only_visible` | Row 2 is invisible; WAL absent/empty is compatible; file header/version unchanged. | Evidence id `crash-matrix-case-CM-001`. |
| T006 | Red scaffolded | CM-002 boundary | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md` | `cargo test --test crash_matrix cm_002_partial_wal_frame_is_ignored` | Reopen exits 0 with only seed row; incomplete tail ignored/truncated without panic. | Evidence id `crash-matrix-case-CM-002`. |
| T007 | Red scaffolded | CM-003 no-commit marker | `tests/crash_matrix.rs`; `tests/wal_recovery.rs` | `cargo test --test crash_matrix cm_003_wal_frame_without_commit_marker_is_not_visible`; `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` | Rolled-back/uncommitted row is not visible and existing regression remains intact. | Current WAL maps absent commit marker to state byte `0x02`. |
| T008 | Red scaffolded | CM-004 idempotent committed replay | `tests/crash_matrix.rs` | `cargo test --test crash_matrix cm_004_committed_wal_before_data_apply_is_idempotent` | First and second reopen show seed and committed row exactly once. | Evidence id `crash-matrix-case-CM-004`. |
| T009 | Red scaffolded | CM-005 interrupted recovery | `tests/crash_matrix.rs` | `cargo test --test crash_matrix cm_005_recovery_interrupted_after_first_apply_replays_remaining_once` | Reopen after interrupted recovery shows rows 1, 2, and 3 exactly once; repeated reopen remains idempotent. | Prefer fixture-only state with row 2 already in page file and WAL rows 2/3 retained. |
| T010 | Red scaffolded | CM-006 committed prefix plus corrupt tail | `tests/crash_matrix.rs` | `cargo test --test crash_matrix cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix` | Successful reopen with rows 1 and 2; no user-facing CLI output/error change. | Use incomplete/invalid-length trailing fragment, not complete invalid magic/checksum frame. |
| T011 | QA scaffolded | fixture identity manifest | `tests/fixtures/crash_matrix/README.md` | `test -f tests/fixtures/crash_matrix/README.md` | Manifest names `seed_committed_one`, CM-001..CM-006, crash labels, evidence IDs, and expected rows. | Binary fixtures may still be generated by test helpers. |
| T012 | QA mapped | WAL state mapping assertion | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md` | `cargo test --test crash_matrix cm_003_wal_frame_without_commit_marker_is_not_visible` | CM-003 report/test states `WAL_STATE_ROLLED_BACK` `0x02` represents no commit marker in current V1 WAL. | Prevents invalid unknown-state fixture from masking corruption behavior. |
| T013 | Red evidence required | red command evidence | `tests/crash_matrix.rs` | `cargo test --test crash_matrix` | Command fails in QA prep for scaffold reason, then passes only after implementation satisfies cases. | Red evidence captured in this run. |
| T014 | QA mapped | CM-005 implementation decision | `tests/crash_matrix.rs`; optional private hook tests | `cargo test --test crash_matrix cm_005_recovery_interrupted_after_first_apply_replays_remaining_once` | Fixture-only proof preferred; smallest private/test-only hook only if fixture proof is impossible. | QA prep does not add core hook. |
| T015 | QA mapped | narrow replay compatibility | `tests/crash_matrix.rs`; `docs/file_format.md` | `cargo test --test crash_matrix cm_002_partial_wal_frame_is_ignored cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix` | Incomplete/invalid-length tail succeeds; complete corrupt frames remain existing storage corruption behavior. | `docs/file_format.md` already has a WAL sidecar note covering this model. |
| T016 | QA mapped | CLI contract preservation | `tests/crash_matrix.rs`; `tests/cli_contract.rs` | `cargo test --test crash_matrix`; `cargo test --test cli_contract` | No public command, flag, stdout, stderr, or exit-code change. | `docs/cli_contract.md` should remain unchanged unless implementation changes user-facing behavior. |
| T017 | Red scaffolded | runner script | `scripts/verify_crash_matrix` | `./scripts/verify_crash_matrix` | Script resolves repo root, runs matrix test, requires generated report, and validates required identifiers. | Red now because crash matrix scaffold intentionally fails before implementation. |
| T018 | QA mapped | observed report data | `tests/crash_matrix.rs`; `scripts/verify_crash_matrix` | `./scripts/verify_crash_matrix` | Report fields come from executed case observations, not a static success template. | Implementation may emit machine-readable case records or run harness in report mode. |
| T019 | QA mapped | markdown report completeness | `target/crash_matrix/crash_matrix_report.md` | `./scripts/verify_crash_matrix`; `test -f target/crash_matrix/crash_matrix_report.md` | Report includes all CM-001..CM-006 fields: case id, evidence id, reopen command, expected rows, actual rows, WAL assertion, exit status. | Generated artifact is current-run evidence only. |
| T020 | QA mapped | report validation failure modes | `scripts/verify_crash_matrix` | `./scripts/verify_crash_matrix` | Script fails if report or any required case/evidence/actual rows/status field is missing. | Validation must be independent from the writer. |
| T021 | QA mapped | file-format documentation | `docs/file_format.md`; `qa_mapping.md` | `rg -n "incomplete trailing frame|Complete WAL frames are retained" docs/file_format.md` | WAL sidecar compatibility note covers no sidecar, incomplete tail cleanup, committed prefix replay, idempotence, and complete corrupt frame errors. | Current doc contains the needed note; implementation final report can cite lines in `docs/file_format.md` if unchanged. |
| T022 | QA mapped | CLI docs non-change guard | `docs/cli_contract.md`; `tests/cli_contract.rs`; `tests/crash_matrix.rs` | `git diff -- docs/cli_contract.md`; `cargo test --test cli_contract` | No CLI docs/test update unless user-facing output/error changes and is kept in scope. | CM-006 required path is successful reopen with no CLI surface change. |
| T023 | QA mapped | required matrix command | `tests/crash_matrix.rs` | `cargo test --test crash_matrix` | Passes after all six cases execute and assert expected rows/status/compatibility. | Red in QA prep. |
| T024 | QA mapped | required runner command | `scripts/verify_crash_matrix`; `target/crash_matrix/crash_matrix_report.md` | `./scripts/verify_crash_matrix` | Passes after script validates current-run report generated from observed cases. | Red in QA prep. |
| T025 | QA mapped | baseline verification | `scripts/verify` | `./scripts/verify` | Baseline fmt, clippy, full tests, and help smoke pass after implementation. | Must not be skipped in implementation/final verification. |
| T026 | QA mapped | WAL regression preservation | `tests/wal_recovery.rs` | `cargo test --test wal_recovery`; `./scripts/verify` | Existing WAL recovery tests remain present and passing. | QA prep red scaffold should not delete or weaken these tests. |
| T027 | QA mapped | scheduler final evidence | scheduler final report artifact; `target/crash_matrix/crash_matrix_report.md` | final phase report plus required commands | Final report links artifact delta and command evidence for all required commands. | Not produced in QA prep; mapped for implementation/final phases. |

## Testing-Review Lens

- All task IDs T001 through T027 are covered above.
- Preferred commands are concrete and runnable from the repo root.
- Each CM case has a named test target, expected rows, and evidence id.
- Negative/boundary coverage is present for absent WAL, partial WAL, uncommitted/rolled-back frame, duplicate replay, interrupted replay, corrupt tail, missing report, and stale evidence.
- The current red scaffold intentionally fails with case-specific messages rather than passing with assertion-light placeholders.

## Red Evidence

Initial QA prep expected red commands:

- `cargo test --test crash_matrix` must fail until the implementation replaces the red scaffold with executable case assertions.
- `./scripts/verify_crash_matrix` must fail because it delegates to the red crash matrix test and requires a current-run report.

The exact command outcomes for this run are recorded in the scheduler result artifact.
