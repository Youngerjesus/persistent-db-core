# QA Mapping: v1-secondary-index-range-scan

## Scope
- Phase: `qa_prep_retry`
- Canonical inputs: `spec.md`, `contracts.md`, `plan.md`, `design.md`, `research.md`, `tasks.md`
- QA target: red verification contract before implementation; no app/core implementation files are modified in this phase.
- Required command evidence for implementation completion:
  - `scripts/verify`
  - `cargo test --test secondary_index -- --nocapture`

## Evidence Classification
- Evidence-heavy: yes, narrowly.
- Reason: acceptance requires a final report or scheduler final report that maps `REQ-7-create-index-must-create-disk-3b71a7dc` to command output, CLI examples, index path evidence, persisted compatibility evidence, and `db check` evidence. This is not a visual/browser evidence task.

## Provenance Contract
- Evidence root: implementation-phase `final_review.md` or the scheduler final report for the current run; QA red evidence is recorded in this mapping and the current `qa_prep_retry` result.
- Required artifact list:
  - `tests/secondary_index.rs`
  - `specs/v1-secondary-index-range-scan/qa_mapping.md`
  - implementation final report containing `scripts/verify` stdout/stderr summary and exit code
  - implementation final report containing `cargo test --test secondary_index -- --nocapture` stdout/stderr summary and exit code
  - implementation final report mapping `REQ-7-create-index-must-create-disk-3b71a7dc` to CLI examples, path-use evidence, persisted compatibility evidence, and `db check` evidence
- Scenario/evidence ids:
  - `T1-primitive-path`: secondary index primitive ordering, duplicate rejection, and path-use evidence point
  - `T2-format-check`: `X`, `E`, and `I` logical record decode/check compatibility
  - `T3-create-index-cli`: `CREATE INDEX` success and exact semantic errors
  - `T4-query-path`: indexed equality and inclusive range scan outputs, no full-scan fallback
  - `T5-persistence`: no-index reopen, backfill, post-index insert, process reopen
  - `T6-db-check`: secondary index success/failure invariant validation
  - `T7-docs`: durable CLI/file-format/SQL subset documentation
  - `T8-final-report`: final command evidence and artifact mapping
- Current-run id source: scheduler metadata `active_run_id=qa_prep_retry_2_resume_20260519_021343_653043_ca61c699` and implementation-phase run metadata when that later phase starts.
- Clean generation rule: canonical launch/report evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: do not reuse prior `scripts/verify`, `cargo test --test secondary_index -- --nocapture`, screenshot, browser, or old final-report output as current acceptance proof.
- Writer/validator separation expectation: implementation phase may write tests/code/docs and final report; verifier/reviewer phase independently validates the report, command evidence, and mapping.
- Redaction target list: no secrets are expected; redact absolute machine-specific temp paths from durable final evidence unless the path is necessary to explain a failure. Do not record environment secrets or external scheduler state.

## Scenario Expansion Lens
- Invalid input:
  - `CREATE INDEX` missing table, missing column, unsupported `TEXT` column, duplicate index name.
  - malformed/non-indexed `WHERE` predicates remain unsupported or malformed through existing CLI exit-code contract.
  - corrupt durable `X`, `E`, and `I` records fail deterministically as `secondary index` or storage readability.
- Empty or partial state:
  - empty table `CREATE INDEX` should commit silently and range/equality should return header-only output.
  - old no-index database must reopen, then allow index creation and backfill.
  - stale orphan `E` entries without committed `X` are ignored by `db exec` and `db check`; same-name retry must use a fresh `build_id` and fresh `E` set before final `X`.
- Duplicate/already-done:
  - duplicate `(secondary_key, tie_break)` primitive entry is rejected without overwrite.
  - duplicate index name is a semantic error.
  - duplicate secondary keys tie-break by primary key or durable insertion order.
- Dependency failure or interrupted flow:
  - interrupted backfill with only `E` records is uncommitted and retryable with a new `build_id`.
  - post-index insert with no durable `I` record must be retryable, and successful post-index insert must persist exactly one `I` record with all embedded entries.
  - malformed `I` missing, extra, wrong-index, wrong-key, wrong-tie-break, or wrong-row-position entries for committed indexes fails check.
  - SQL `BETWEEN` with `low > high` must still use the secondary range path and return header-only output.
- Permission/trust boundary:
  - no network, browser, DOM, screenshot, or external service evidence is accepted.
  - raw persisted fixture tests are trusted only as deterministic local fixtures using the documented byte layout.
- Retry/re-entry:
  - same-name retry after orphan `E` records must commit cleanly and ignore old orphan entries.
  - verification retry must regenerate current-run command evidence and final report mapping.

## Task Mapping

### T1. Add secondary index primitive and path-use evidence point
- Status: red scaffolded
- Verification Layers: internal primitive behavior; implementation-visible path marker/helper; no full-scan guard for secondary query planning
- Test Files: `tests/secondary_index.rs`
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
- Task-Scoped Green:
  - `SecondaryIndex::new`, `insert`, `equality_positions`, and `range_positions` compile and pass deterministic ordering assertions.
  - duplicate `(secondary_key, tie_break)` insert returns an error and does not overwrite the original row position.
  - equality/range query implementation exposes or proves `SecondaryIndexEquality` and `SecondaryIndexRange` path use without a broad user-facing debug surface.
- Notes:
  - Red scaffold currently imports `persistent_db_core::index::SecondaryIndex`, matching `design.md` planned API shape.
  - `planner_path_marker_reports_secondary_equality_and_range_after_create_index` requires a non-user-facing `plan_query_path_for_test`/`QueryPath` path evidence point and asserts `SecondaryIndexEquality` and `SecondaryIndexRange`.

### T2. Extend SQL logical records for secondary indexes
- Status: red scaffolded
- Verification Layers: raw persisted fixtures; old-format compatibility; decode/check invariant failures; orphan-entry ignore behavior
- Test Files: `tests/secondary_index.rs`
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
  - `scripts/verify`
- Task-Scoped Green:
  - fixture helpers encode final-layout `X`, `E`, and `I` records using little-endian binary fields.
  - old no-index catalog/row records reopen and can be backfilled.
  - committed metadata plus missing entry, wrong key, wrong tie-break, invalid row position, duplicate entry, missing table metadata, and missing column metadata fails as deterministic `secondary index`.
  - orphan `E` records without matching committed `X(build_id,index_name)` are ignored by `db exec` and `db check`.
  - malformed `I` records with missing, extra, wrong-index, wrong-key, wrong-tie-break, and wrong-row-position embedded entries fail as deterministic `secondary index`.
- Notes:
  - `db_check_secondary_index_corruption_matrix_reports_secondary_index` covers committed `E`/`X` matrix cases.
  - `indexed_row_corruption_matrix_reports_secondary_index` covers malformed `I` embedded-entry matrix cases.

### T3. Implement `CREATE INDEX`
- Status: red scaffolded
- Verification Layers: black-box CLI success and exact semantic error matrix
- Test Files: `tests/secondary_index.rs`; optional focused additions to `tests/sql_exec.rs` during implementation if shared SQL contract coverage needs them
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
- Task-Scoped Green:
  - `CREATE INDEX idx_users_age ON users(age);` exits `0` with empty stdout/stderr.
  - missing table, missing column, unsupported type, and duplicate index return exit `2`, empty stdout, and the exact contract stderr/hints.
  - backfill writes `E` records before final `X`, and retry after interrupted backfill commits with a new `build_id`.
  - stale orphan `E(build_id=2)` with wrong key/tie-break is retained as orphan, while retry commits `X(build_id=3)` and fresh valid `E(build_id=3)` records immediately before final `X`.
- Notes:
  - Exact stderr examples are pinned in `create_index_semantic_errors_match_contract_exactly`.
  - `stale_orphan_backfill_entries_are_ignored_and_retry_commits_fresh_build_id` discriminates stale-orphan reuse from correct retry behavior.

### T4. Implement indexed equality and inclusive range scan
- Status: red scaffolded
- Verification Layers: black-box CLI outputs; ordering/tie-break assertions; no full-scan fallback; path-use evidence from T1 implementation marker/helper
- Test Files: `tests/secondary_index.rs`; optional `tests/sql_exec.rs`
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
- Task-Scoped Green:
  - required equality example outputs `id|age|name\n2|20|bea\n3|20|cal\n`.
  - required inclusive range example outputs `id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n`.
  - duplicate secondary key tie-break is primary key for primary-key tables and durable row insertion order for no-primary-key tables.
  - `SELECT * FROM users WHERE age = 20;` before `CREATE INDEX` exits `2` with empty stdout and unsupported SQL stderr for the exact statement.
  - `SELECT * FROM users WHERE age BETWEEN 10 AND 20;` before `CREATE INDEX` exits `2` with empty stdout and unsupported SQL stderr for the exact statement.
  - `SELECT * FROM users WHERE age BETWEEN 30 AND 20;` after `CREATE INDEX` returns header-only output through `SecondaryIndexRange`.
  - accepted secondary equality and `BETWEEN` predicates use the secondary index path, not a full table scan.
- Notes:
  - Current unsupported SQL hint assertion uses the pre-implementation documented hint. If implementation updates the durable hint to mention `CREATE INDEX`/`BETWEEN`, update this test and docs together.
  - `planner_path_marker_reports_secondary_equality_and_range_after_create_index` is the explicit guard against full-table-scan implementations after `CREATE INDEX`.
  - `range_predicate_before_create_index_is_unsupported_not_full_scan` locks missing-index range behavior.
  - `indexed_range_with_low_greater_than_high_returns_header_only_through_range_path` locks SQL-level range boundary behavior and path use.

### T5. Maintain secondary indexes on post-index inserts and reopen
- Status: red scaffolded
- Verification Layers: persisted compatibility; process reopen behavior through repeated CLI invocations; raw `I` fixture validation
- Test Files: `tests/secondary_index.rs`
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
  - `scripts/verify`
- Task-Scoped Green:
  - old no-index databases reopen, then `CREATE INDEX` backfills existing rows.
  - rows inserted after index creation appear in equality and range outputs.
  - reopening through separate `db exec` invocations preserves equality/range output.
  - post-index inserts use one atomic `I` record containing all required embedded entries, not `R + standalone E`.
  - successful post-index insert with two committed indexes produces exactly one `I` record, zero post-index `R` records, zero standalone `E` records, and two embedded entries.
  - interrupted/failed post-index insert with no durable `I` record can be retried and produces one atomic `I`.
  - malformed or partial multi-index `I` records fail `db check` as `secondary index`.
- Notes:
  - `old_no_index_database_reopens_then_backfills_and_post_index_insert_persists` is the contract-required compatibility path.
  - `post_index_insert_persists_one_atomic_indexed_row_record_for_all_indexes` locks the success durability shape.
  - `interrupted_post_index_insert_with_no_indexed_row_record_can_be_retried` locks the no-durable-`I` retry path; a recovered whole-`I` duplicate retry remains covered by normal primary-key duplicate semantics during implementation verification.

### T6. Add `db check` secondary index invariants
- Status: red scaffolded
- Verification Layers: black-box `db check`; raw corrupt fixture checks
- Test Files: `tests/secondary_index.rs`
- Preferred Commands:
  - `cargo test --test secondary_index -- --nocapture`
  - `scripts/verify`
- Task-Scoped Green:
  - valid secondary-indexed database returns `ok: db check passed\n`, empty stderr, exit `0`.
  - committed secondary metadata/content mismatch returns exit `1`, empty stdout, and `error: db check failed: secondary index\n`.
  - orphan backfill entries are ignored and still pass check.
  - committed `E`/`X` corruptions for missing entry, wrong key, wrong tie-break, invalid row position, duplicate entry, missing table metadata, and missing column metadata fail as `secondary index`.
  - malformed `I` records with missing, extra, wrong-index, wrong-key, wrong-tie-break, and wrong-row-position entries fail as `secondary index`.
- Notes:
  - Existing `tests/db_check.rs` remains the baseline check contract; this feature's focused check evidence is isolated in `tests/secondary_index.rs`.

### T7. Update durable docs
- Status: mapped, implementation phase pending
- Verification Layers: manual review plus `scripts/verify` doc-linked examples where applicable
- Test Files: not directly test-only; behavior must stay aligned with `tests/secondary_index.rs`
- Preferred Commands:
  - `scripts/verify`
- Task-Scoped Green:
  - `docs/cli_contract.md` documents `CREATE INDEX`, indexed equality, inclusive `BETWEEN`, success/error stdout/stderr/exit codes, exact errors, ordering, and tie-breaks.
  - `docs/file_format.md` documents `X`, `E`, and `I` encoding, no-index compatibility, backfill state machine, post-index insert atomicity, and `db check` validation.
  - `docs/sql_subset.md` documents the narrow SQL grammar and logical records.
  - docs match exact test strings and final encoding.
- Notes:
  - Docs should be updated after behavior and encoding are final to avoid contract drift.

### T8. Verification and final implementation report
- Status: mapped, implementation phase pending
- Verification Layers: command evidence; final report acceptance mapping; sibling requirement non-closure statement
- Test Files: final report path, `tests/secondary_index.rs`, full repo verification suite
- Preferred Commands:
  - `scripts/verify`
  - `cargo test --test secondary_index -- --nocapture`
- Task-Scoped Green:
  - both required commands pass with stdout/stderr and exit code summarized in the final report.
  - final report maps `REQ-7-create-index-must-create-disk-3b71a7dc` to CLI examples, path-use evidence, persisted compatibility evidence, and `db check` evidence.
  - final report states sibling `gate-v1-indexes` requirements were not closed by inference.
- Notes:
  - A second recovery attempt after verifier rejection must escalate per contract.

## Testing-Review Lens Self-Check
- All Task IDs covered: yes, T1 through T8 have mapping entries.
- Preferred commands concrete and runnable: yes, `cargo test --test secondary_index -- --nocapture` and `scripts/verify`.
- Task-scoped green criteria specific: yes, each task has concrete output, invariant, or docs criteria.
- Negative/boundary coverage:
  - invalid `CREATE INDEX` semantic errors: covered.
  - unsupported non-index predicate before `CREATE INDEX`: covered.
  - unsupported range predicate before `CREATE INDEX`: covered.
  - low-greater-than-high SQL `BETWEEN` boundary through secondary range path: covered.
  - explicit post-index path-use marker for equality/range: covered.
  - duplicate secondary entries: covered at primitive level.
  - old no-index compatibility: covered.
  - stale orphan interrupted backfill retry with fresh build id and E-before-X ordering: covered.
  - successful post-index atomic `I` persistence and no-durable-`I` retry path: covered.
  - committed `E`/`X` corruption matrix: covered.
  - malformed `I` embedded-entry matrix: covered.
  - range low-greater-than-high: covered at primitive and SQL CLI levels.
- Flakiness review:
  - temp database paths include process id and monotonic timestamp suffix.
  - tests avoid network/browser/external services.
  - raw fixtures use deterministic byte helpers.
- Known red expectation:
  - before implementation, `cargo test --test secondary_index -- --nocapture` should fail because `SecondaryIndex`, the non-user-facing query path marker, and secondary SQL/file-format support do not exist yet.

## Red Evidence
- Retry 1 repair added explicit path-use scaffold, full committed secondary-index corruption matrix, malformed `I` matrix, post-index atomic `I` success evidence, and no-durable-`I` retry evidence.
- Retry 2 repair added stale-orphan interrupted-backfill discriminator, fresh retry `build_id`/record-order assertions, pre-index `BETWEEN` unsupported coverage, and SQL low-greater-than-high `BETWEEN` range-path coverage.
- `cargo fmt --check`
  - Exit code: `1` before formatting the retry repair; follow-up `cargo fmt` executed successfully and only formatted QA scaffold.
  - Refreshed exit code: `0` after formatting.
- `cargo test --test secondary_index -- --nocapture`
  - Exit code: `101`.
  - Expected red reason: `error[E0432]` unresolved imports for `persistent_db_core::index::SecondaryIndex`, `persistent_db_core::sql::plan_query_path_for_test`, and `persistent_db_core::sql::QueryPath`.
  - stdout/stderr summary: compilation reached `persistent-db-core`; test target failed before runtime assertions because the T1 primitive and explicit T1/T4 path marker are not implemented yet.
- `scripts/verify`
  - Exit code: `101`.
  - Expected red reason: same unresolved `SecondaryIndex`, `plan_query_path_for_test`, and `QueryPath` imports from `tests/secondary_index.rs`.
  - stdout/stderr summary: baseline verification reaches cargo check/test compilation and fails on the new red QA scaffold.
