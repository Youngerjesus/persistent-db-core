# QA Mapping: v1-secondary-index-mutation-consistency

Status: qa-prep-red-scaffolded
Phase: qa_prep_exec
Run ID: qa_prep_exec_fresh_20260519_033057_020367_0cdd2e12

## Evidence-Heavy Assessment

This task is not evidence-heavy under the browser/visual/exported-report definition. Acceptance is based on repo-local deterministic Rust integration tests, CLI stdout/stderr/exit-code assertions, `db check`, and command summaries. The approved spec explicitly excludes browser evidence, screenshots, visual evidence, and UX design-review evidence.

## Provenance Contract

- Evidence root: `specs/v1-secondary-index-mutation-consistency/qa_mapping.md`, `tests/secondary_index.rs`, and the current run result at `autopilot/project_manager/tasks/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/runs/qa_prep_exec_fresh_20260519_033057_020367_0cdd2e12/result.md`.
- Required artifact list: QA mapping manifest, red test scaffold in `tests/secondary_index.rs`, command evidence for `cargo test --test secondary_index -- --nocapture`, implementation-phase evidence for `./scripts/verify`, and implementation final report mapping requirement IDs.
- Scenario IDs / evidence IDs: `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`, `S1-update-old-key-removed`, `S2-update-new-key-added`, `S3-update-range-order`, `S4-update-primary-and-table-agree`, `S5-delete-secondary-removed`, `S6-delete-primary-and-table-agree`, `S7-restart-process-boundary`, `S8-wal-sidecar-replay`, `S9-stale-secondary-entry`, `S10-dangling-secondary-pointer`, `S11-missing-visible-secondary-entry`, `S12-unsupported-mutation-breadth`.
- Current-run id source: scheduler metadata `active_run_id` and this file header.
- Clean generation rule: canonical launch or verification evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: verifier-facing command summaries must come from commands executed in this task worktree for the current pass; prior secondary-index range-scan evidence is historical only.
- Writer/validator separation expectation: QA prep writes mapping and red scaffold; implementation makes tests pass; later verifier/reviewer independently validates command evidence and does not treat this mapping as a pass verdict.
- Redaction target list: none. The repo-local commands require no secrets, tokens, credentials, screenshots, or external service payloads.

## Scenario Expansion Lens

| Scenario | Pressure Point | QA Reflection |
|---|---|---|
| S1 update old key removed | Existing `age=20` entry for `id=2` must not survive after `age=30` update. | Exact `SELECT * WHERE age = 20` assertion plus stale-entry negative fixture. |
| S2 update new key added | `id=2` must appear under new secondary key with primary-key tie-break before `id=4`. | Exact `SELECT * WHERE age = 30` assertion. |
| S3 update range order | Inclusive range must sort by secondary key then primary key after mutation. | Exact `BETWEEN 20 AND 30` assertion. |
| S4 update primary/table agreement | Primary lookup and full scan must agree with secondary state. | Exact `WHERE id = 2` and `SELECT *` assertions. |
| S5 delete secondary removed | Deleted `id=3 age=20` must not remain in equality/range scans. | Header-only equality and range assertions after delete. |
| S6 delete primary/table agreement | Deleted row must be invisible to primary lookup and table scan. | Header-only primary lookup and exact table scan assertions. |
| S7 restart/process boundary | Setup, mutation, query, and check must reopen through separate `db` invocations. | Black-box integration test uses one `db` process per step. |
| S8 WAL sidecar replay | Page file and retained `<path>.wal` sidecar must exist while reopen query/check run. | WAL replay scaffold asserts both files exist before query/check. |
| S9 stale secondary entry | Old secondary key remains for visible updated row. | Deterministic fixture builder expects `db check` exit 1 and exact `secondary index` stderr. |
| S10 dangling secondary pointer | Secondary entry points to deleted/nonexistent row position. | Deterministic fixture builder expects exact `secondary index` failure. |
| S11 missing visible secondary entry | Visible indexed row lacks committed secondary entry. | Deterministic fixture builder expects exact `secondary index` failure. |
| S12 unsupported mutation breadth | Non-primary-key or broader mutations must not silently become supported. | Mapped to T3 parser/error tests in implementation; keep out of fixed fixture acceptance. |
| Partial append/retry | Runtime state must not change if durable append fails; replay must be deterministic. | Mapped to T4/T5 implementation tests around single logical record and append-before-apply. |
| Duplicate/already-done | Repeated index metadata or duplicate secondary entries remain invalid. | Existing secondary-index corruption tests plus T2/T5 helper tests. |
| Dependency failure | Missing table/column/type mismatch must fail before mutation. | Mapped to T3 semantic validation tests. |
| Permission/trust boundary | `db check` and `db exec` operate only on local path; no external services/secrets. | Covered by existing CLI/storage tests; no new permission artifact required for this feature. |

## Task Coverage

### T1. Add red contract tests for mutation fixture outputs

- Status: red-scaffolded
- Verification Layers: black-box CLI integration, exact stdout/stderr/exit-code assertions, separate process invocation per setup/mutation/query/check step.
- Test Files: `tests/secondary_index.rs`
- Preferred Commands: `cargo test --test secondary_index mutation_contract_update_delete_restart_processes_keep_secondary_indexes_consistent -- --nocapture`; full focused command `cargo test --test secondary_index -- --nocapture`.
- Task-Scoped Green: the fixed users fixture is created exactly; supported update/delete both exit 0 silently; every post-update and post-delete query stdout matches `contracts.md`; `db check` exits 0 with `ok: db check passed\n`.
- Notes: current red expectation is that `UPDATE users SET age = 30 WHERE id = 2;` is still unsupported before implementation.

### T2. Add deterministic `db check` negative fixtures

- Status: scaffolded
- Verification Layers: deterministic SQL-record fixture builders, exact `db check` exit/stdout/stderr assertions.
- Test Files: `tests/secondary_index.rs`
- Preferred Commands: `cargo test --test secondary_index mutation_contract_db_check -- --nocapture`; full focused command `cargo test --test secondary_index -- --nocapture`.
- Task-Scoped Green: stale old-key entry, dangling/nonexistent row pointer, and missing visible indexed row each fail with exit 1, empty stdout, and exactly `error: db check failed: secondary index\n`.
- Notes: fixtures are in `tests/secondary_index.rs`, so `cargo test --test db_check -- --nocapture` is not required unless implementation moves this coverage.

### T3. Extend SQL parser for narrow mutation forms

- Status: mapped-not-implemented
- Verification Layers: black-box CLI parser/semantic tests, unsupported breadth regression tests.
- Test Files: `tests/secondary_index.rs`; optional `tests/sql_exec.rs` if parser-specific errors are split out.
- Preferred Commands: `cargo test --test secondary_index -- --nocapture`; optional targeted `cargo test --test sql_exec update delete -- --nocapture` if implementation adds parser-only cases there.
- Task-Scoped Green: accepted primary-key-targeted `UPDATE` and `DELETE` forms execute; missing table/column/type/wrong predicate forms fail deterministically without introducing broad table-scan mutation semantics.
- Notes: unsupported mutation-breadth scenarios must preserve existing user-facing error style where the contract does not define exact new errors.

### T4. Implement durable update/delete logical records

- Status: mapped-not-implemented
- Verification Layers: record-count/record-kind assertions, reopen replay tests, compatibility tests for existing `C/R/E/X/I` databases.
- Test Files: `tests/secondary_index.rs`; docs evidence in `docs/sql_subset.md` and `docs/file_format.md` during implementation.
- Preferred Commands: `cargo test --test secondary_index -- --nocapture`; baseline `./scripts/verify` after docs update.
- Task-Scoped Green: successful mutations append one durable logical `U` or `D` record each; existing row-only and existing secondary-index files reopen unchanged; malformed/wrong mutation deltas fail under `db check`.
- Notes: lower-level page/WAL framing must remain unchanged unless implementation explicitly escalates a spec conflict.

### T5. Maintain runtime primary and secondary indexes on mutation

- Status: mapped-not-implemented
- Verification Layers: exact query-output assertions across secondary equality, secondary range, primary lookup, and table scan.
- Test Files: `tests/secondary_index.rs`; optional `src/index.rs` unit-style helper coverage if removal/update helpers are added.
- Preferred Commands: `cargo test --test secondary_index -- --nocapture`.
- Task-Scoped Green: update replaces the row at a stable position and moves secondary entries; delete marks the row invisible and removes primary/secondary entries; all query paths agree after mutation.
- Notes: runtime state must apply only after durable append succeeds.

### T6. Add restart and WAL replay evidence

- Status: red-scaffolded
- Verification Layers: separate compiled `db` invocations, file existence checks for page file and `<path>.wal`, reopen query, positive `db check`.
- Test Files: `tests/secondary_index.rs`
- Preferred Commands: `cargo test --test secondary_index mutation_contract_retained_wal_replay_keeps_secondary_indexes_consistent -- --nocapture`; full focused command `cargo test --test secondary_index -- --nocapture`.
- Task-Scoped Green: after setup/update/delete through separate process invocations, retained WAL sidecar replay yields exact range query output and `db check` success while page and WAL files exist.
- Notes: current scaffold will remain red until mutation records are implemented and WAL retention behavior satisfies the contract.

### T7. Update durable docs

- Status: mapped-not-implemented
- Verification Layers: durable docs diff review, CLI contract tests if help/output changes, baseline verification.
- Test Files: existing docs-backed tests such as `tests/cli_contract.rs` if user-facing help changes; otherwise docs are reviewed through `./scripts/verify`.
- Preferred Commands: `./scripts/verify`.
- Task-Scoped Green: `docs/cli_contract.md`, `docs/sql_subset.md`, and `docs/file_format.md` describe supported mutation forms, exact success behavior, `U`/`D` record layouts, `db check` invariants, and compatibility notes without contradicting tests.
- Notes: do not update docs in QA prep; docs change belongs to implementation once bytes and behavior are final.

### T8. Verification and implementation report

- Status: mapped-not-implemented
- Verification Layers: command execution evidence, stdout/stderr summaries, requirement/evidence ID traceability, compatibility statement.
- Test Files: run-local result/final report artifact.
- Preferred Commands: `./scripts/verify`; `cargo test --test secondary_index -- --nocapture`; `cargo test --test db_check -- --nocapture` only if negative fixtures move to `tests/db_check.rs`.
- Task-Scoped Green: final report records command, exit status, stdout/stderr summary, relevant test names, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`, and storage compatibility outcome.
- Notes: QA prep result is not implementation completion evidence.

## Testing-Review Lens

- All Task IDs T1-T8 are mapped.
- Preferred commands are concrete and runnable from the task worktree root.
- Task-scoped green criteria are specific and tied to `contracts.md`.
- Negative/boundary coverage includes stale entry, dangling pointer, missing visible entry, unsupported breadth, dependency validation, partial append/retry, duplicate/already-done, and process-boundary restart/WAL paths.
- Current red scaffold intentionally does not modify `src/` or durable docs.
- `tests/secondary_index.rs` is the only test file modified in QA prep, so no `tests/db_check.rs` focused command is currently required.

## Red Evidence

- Current-run command: `cargo test --test secondary_index -- --nocapture`.
- Exit status: 101.
- Summary: 26 tests compiled and ran; 24 passed and 2 failed.
- Failing tests:
  - `mutation_contract_update_delete_restart_processes_keep_secondary_indexes_consistent`
  - `mutation_contract_retained_wal_replay_keeps_secondary_indexes_consistent`
- Red reason: both failures occur at the first supported mutation statement, `UPDATE users SET age = 30 WHERE id = 2;`, where the scaffold expects exit 0 with empty stdout/stderr but current implementation exits 2 with `error: unsupported SQL statement: UPDATE users SET age = 30 WHERE id = 2;` and the documented SQL subset hint.
- Passing scaffold evidence: the three deterministic `db check` negative fixtures compile and currently pass with exact `secondary index` failure assertions:
  - `mutation_contract_db_check_rejects_stale_secondary_entry_after_update`
  - `mutation_contract_db_check_rejects_dangling_secondary_pointer_after_delete`
  - `mutation_contract_db_check_rejects_missing_visible_secondary_entry`
