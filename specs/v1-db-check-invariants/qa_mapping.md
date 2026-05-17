# QA Mapping: `db check` Invariant Validation

## Scope And Phase

- Phase: `qa_prep_exec`
- Source spec: `specs/v1-db-check-invariants/spec.md`
- Source contract: `specs/v1-db-check-invariants/contracts.md`
- Current task run id source: scheduler metadata `active_run_id=qa_prep_exec_fresh_20260518_034805_719555_16589063`
- QA prep rule: this phase adds canonical QA mapping and red test scaffolds only. Application/core implementation files remain untouched.

## Scenario Expansion Lens

- Happy path: a database created by `db exec` passes `db check <path>` with exit `0`, stdout exactly `ok: db check passed\n`, and empty stderr.
- Invalid input: missing path and malformed `db check` arity must remain user-facing CLI errors; unsupported future commands must still report the first token.
- Partial/corrupt state: truncated or internally inconsistent page record length must fail under the `storage record readability` invariant label.
- Logical contradiction: persisted SQL row bytes without a matching catalog must fail under `catalog/record invariant`.
- Duplicate/already-done state: duplicate durable primary-key rows must fail under `primary index`; retained already-applied WAL frames are not themselves failure evidence.
- Dependency failure/trust boundary: temp directory passed as `<path>` is the required unreadable path evidence and must not depend on permissions or platform skips.
- Replay/re-entry: complete committed WAL page-append frame whose `record count before` is greater than durable record count must fail under `wal replay consistency`; absent WAL or storage-only corruption cannot substitute for this scenario.

## Evidence Classification

This is not a browser/UI evidence-heavy task under the frozen contract. Completion evidence is deterministic CLI/test evidence plus durable docs diff. Because the task still depends on current-run command output and generated fixture code, the implementation phase must keep provenance for those artifacts.

## Provenance Contract

- Evidence root: current implementation/verification run directory for task `task-2026-05-18-03-29-23-v1-db-check-invariants`; QA prep output lives in `specs/v1-db-check-invariants/qa_mapping.md`.
- Required artifact list: `tests/db_check.rs`, updated `tests/cli_contract.rs`, future `docs/cli_contract.md` diff, optional future `docs/file_format.md` diff, `cargo test --test db_check` output, `scripts/verify` output, final implementation run report.
- Scenario ids: `DBCHK-VALID`, `DBCHK-STORAGE-CORRUPT`, `DBCHK-CATALOG-RECORD`, `DBCHK-PRIMARY-INDEX`, `DBCHK-WAL-AHEAD`, `DBCHK-MISSING-PATH`, `DBCHK-DIRECTORY-PATH`, `DBCHK-HELP-SURFACE`, `DBCHK-DOCS`, `DBCHK-VERIFY`.
- Current-run id source: scheduler-provided `active_run_id` or the implementation phase's own run id when it supersedes QA prep.
- Clean generation rule: canonical launch/verification evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: prior terminal output, previous run reports, stale screenshots, or old generated fixtures do not satisfy current proof; tests must generate database/WAL fixtures inside the current test process.
- Writer/validator separation expectation: implementation worker may write code/docs/tests and capture command output; verifier/reviewer must independently inspect the latest QA mapping, test code, and current-run command evidence.
- Redaction target list: no secrets are expected. If future logs include absolute local paths, keep them only where needed for command evidence and do not copy unrelated machine/runtime state into durable docs.

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | red scaffold added | CLI integration; deterministic fixture generation; storage/logical/index/WAL negative coverage; missing/directory path boundary coverage | `tests/db_check.rs` | `cargo test --test db_check` | All seven `db check` tests pass; valid case has exact stdout/stderr; each corrupt case has required prefix and label; missing and directory paths fail without panic. | Covers `DBCHK-VALID`, `DBCHK-STORAGE-CORRUPT`, `DBCHK-CATALOG-RECORD`, `DBCHK-PRIMARY-INDEX`, `DBCHK-WAL-AHEAD`, `DBCHK-MISSING-PATH`, `DBCHK-DIRECTORY-PATH`. Initial expected state is red because `db check` is unsupported. |
| T2 | red scaffold added | CLI help contract; reserved-command regression | `tests/cli_contract.rs` | `cargo test --test cli_contract` or `scripts/verify` | Help output lists `db check <path>` as supported, omits it from reserved future commands, and `open`/`bench` remain unsupported. | `DBCHK-HELP-SURFACE`; expected red until `src/main.rs` help text and routing are updated. |
| T3 | pending implementation | CLI route behavior; stdout/stderr/exit-code mapping; non-regression for existing command parsing | `tests/db_check.rs`, `tests/cli_contract.rs` | `cargo test --test db_check`; `cargo test --test cli_contract` | `db check <path>` dispatches to checker, valid DB exits `0`, invariant failures exit non-zero with `error: db check failed:`, open/read failures use `error:` and path context. | QA prep must not implement this task. |
| T4 | pending implementation | Read-only page scan; durable record count; read-only WAL sidecar parsing; no mutation of existing `db exec` behavior | `tests/db_check.rs`, existing storage/WAL tests through `scripts/verify` | `cargo test --test db_check`; `cargo test --test wal_recovery`; `scripts/verify` | Storage corruption and WAL ahead-of-store fixtures are rejected by `db check`; existing WAL replay tests still pass. | WAL test fixture writes complete committed `0x01` frame with `record count before > durable record count`. |
| T5 | pending implementation | SQL logical validation; catalog/row invariant labeling; primary-key rebuild/duplicate detection | `tests/db_check.rs`, existing SQL/index tests through `scripts/verify` | `cargo test --test db_check`; `cargo test --test primary_index`; `scripts/verify` | Row-without-catalog fails as `catalog/record invariant`; duplicate durable primary key fails as `primary index`; existing SQL output and grammar remain unchanged. | Primary index means rebuildability/key-set consistency, not a persisted index sidecar. |
| T6 | pending docs | Durable CLI contract documentation | future `docs/cli_contract.md` diff plus test expectations | `scripts/verify` | Docs list `db check <path>`, exact success output, failure prefix/category, no repair/mutation note, and remove `check <path>` from reserved commands. | `DBCHK-DOCS`; implementation phase owns docs update. |
| T7 | pending docs | Durable file-format compatibility note; WAL sidecar source alignment | future `docs/file_format.md` diff if needed | `scripts/verify` | Docs state `db check` validates page records, SQL logical consistency, primary-key rebuildability, and documented WAL sidecar consistency. | Must clarify no separate persisted primary-index file exists. |
| T8 | pending verification | Focused current-run test evidence | `tests/db_check.rs` | `cargo test --test db_check` | Focused command passes in current task worktree after implementation. | QA prep captures red evidence now; implementation must capture green evidence later. |
| T9 | pending verification | Baseline verification from current task worktree | full repo | `scripts/verify` | `cargo fmt --check`, clippy `-D warnings`, full tests, and help smoke pass. | Required completion gate. |
| T10 | pending report | Acceptance-to-evidence mapping; scheduler outcome report; explicit CLI-only evidence note | implementation run report | n/a; report references `cargo test --test db_check` and `scripts/verify` outputs | Final report maps every Candidate Acceptance Criteria to evidence and states visual/UX evidence is intentionally not applicable. | No UI, DOM, screenshot, rendered route, or UX review artifacts are completion evidence. |

## Testing-Review Lens

- Task ID coverage: T1 through T10 are mapped.
- Preferred commands: focused command is `cargo test --test db_check`; baseline command is `scripts/verify`; CLI-contract surface can be isolated with `cargo test --test cli_contract`.
- Task-scoped green definitions: each task has concrete pass criteria tied to stdout/stderr, labels, docs, or command evidence.
- Negative/boundary coverage: storage corruption, catalog/record contradiction, duplicate primary key, WAL ahead-of-store, missing path, directory path, reserved commands, and no-repair/no-mutation docs are represented.
- Flake control: temp paths include process id and nanosecond suffix; directory-path evidence does not rely on permissions; WAL fixture bytes are generated locally by the test.

## Red Evidence Captured During QA Prep

- `cargo test --test db_check` result: red as expected. The test binary compiles; all 7 tests fail because current CLI routes `check` to `error: unsupported argument or command: check`.
- `cargo test --test cli_contract` result: red as expected. 4 existing behavior checks pass; 2 help-surface checks fail because help output does not yet list `db check <path>` as supported and still lists `check <path>` as reserved.
- `scripts/verify` result: red as expected. The script reaches the full test suite and stops at the same `tests/cli_contract.rs` help-surface failures.
- Interpretation: QA prep is verifier-ready for this phase; red status is due to intentionally missing implementation/docs, not broken test compilation or fixture setup.
