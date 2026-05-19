# Code Review History

## Archived 2026-05-19T14:48:24+0900

Verdict: FAIL

## Scope

- Phase: `code_review_exec` for `task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency`.
- Baseline identification:
  - `git log --oneline main..HEAD`: no commits.
  - `git diff main...HEAD`: no committed branch diff.
  - Review target is the current uncommitted worktree diff.
- Reviewed changed files: `src/sql.rs`, `src/index.rs`, `tests/secondary_index.rs`, `docs/cli_contract.md`, `docs/file_format.md`, `docs/sql_subset.md`, and task artifacts under `specs/v1-secondary-index-mutation-consistency/`.
- Context reviewed: `spec.md`, `contracts.md`, `qa_mapping.md`, `impl_review.md`, `impl_brake_review.md`, current diff, and companion reviewer outputs.
- Verification commands run during review:
  - `cargo test --test secondary_index -- --nocapture`: exit 0, 29 passed.
  - `./scripts/verify`: exit 0; fmt, clippy, full test suite, doc tests, and help smoke passed.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
|---|---|---|---|---|---|---|
| code-reviewer | Correctness, regressions, completeness, spec mismatch, merge safety | invoked | Agent `019e3eba-8b93-77a0-9d75-24a041dd827f`: no merge findings; noted residual mutation error-surface and WAL lifecycle test gaps. | CR-002, CR-004 | none | n/a |
| testing-reviewer | Coverage gaps, negative paths, edge cases, merge confidence | invoked | Agent `019e3eba-8bee-79f3-bb86-fe4105f503ef`: found missing in-bounds deleted-row dangling fixture and unsupported mutation breadth coverage. | CR-002, CR-003, CR-004 | none | n/a |
| security-reviewer | SQL input boundary and file/WAL persistence boundary changed | invoked | Agent `019e3eba-8c50-78b2-b02d-a9307826d6c7`: reproduced semantically invalid committed `U` WAL frame that `db check` passed before normal open persisted corruption. | CR-001 | none | n/a |
| performance-reviewer | New mutation replay, tombstone scans, validation loops | invoked | Agent `019e3eba-8cab-7b82-9de1-0d49534073a9`: flagged O(history) tombstone memory/reopen/check/index-build cost. | none | CR-R001 | Treated as residual risk, not merge blocker, because the approved spec explicitly requires stable row positions for secondary-index row pointers and has no compaction/performance bound for this slice. |
| maintainability-reviewer | Mutation logic added in `src/sql.rs` | fallback-applied | Self-applied lens: reviewed `U`/`D` encode/decode, tombstone state, and index update helpers for coupling/complexity. No separate merge finding beyond accepted test/WAL issues. | none | none | Dedicated reviewer role unavailable in this runtime. |
| red-team-reviewer | Additive bias, proxy-success evidence, cross-category gaps | fallback-applied | Self-applied lens plus testing/security outputs. Accepted proxy-success gap around `db check`/WAL semantics as CR-001 and retained-WAL evidence as CR-004. | CR-001, CR-004 | none | Dedicated reviewer role unavailable in this runtime. |
| database-reviewer | SQL, indexes, persistence, durable state changed | fallback-applied | Self-applied database lens over `src/sql.rs`, `src/storage.rs`, `src/check.rs`, and secondary-index tests. Accepted CR-001, CR-002, CR-003. | CR-001, CR-002, CR-003 | CR-R001 | Dedicated reviewer role unavailable in this runtime. |
| api-reviewer | Endpoint/transport API changes | skipped | No HTTP/API/DTO/status-code surface in this CLI-only diff. | none | none | Not triggered. |
| ui-ux-reviewer | UI component/layout/accessibility changes | skipped | No UI surface in this Rust CLI diff. | none | none | Not triggered. |

## Findings

- CR-001: `db check` can pass a semantically invalid committed mutation WAL frame, and a later normal open can persist that invalid frame into the base file.
  - Severity: Critical
  - Must Fix Now: yes
  - Evidence: [src/check.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/check.rs:24) validates only durable base records with `sql::validate_records_for_check`, then [src/check.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/check.rs:36) calls WAL validation separately. [src/storage.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/storage.rs:122) only validates WAL frame shape and page-append record-count ordering, not SQL/catalog/index semantics after applying committed WAL payloads. Normal `PageStore::open` replays WAL at [src/storage.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/storage.rs:68), appending the payload to the base file at [src/storage.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/storage.rs:337), before `Database::from_records` can reject invalid `U`/`D` semantics at [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:393).
  - Impact: A checksum-valid retained WAL sidecar with an impossible update row position, wrong table, or mismatched embedded secondary entries is reported healthy by `db check`, then normal `db exec` can durably poison the base file. This violates the task's WAL replay and `db check` integrity evidence expectations for mutation records.
  - Expected repair: Make `db check` validate the semantic result of committed WAL replay, and ensure normal open does not persist semantically invalid SQL mutation records into the base file before they have passed the SQL invariant layer.

- CR-002: The required deleted-row dangling-pointer case is not directly tested.
  - Severity: High
  - Must Fix Now: yes
  - Evidence: The contract allows the dangling fixture to reference a nonexistent row position or a deleted row, but this task introduced tombstoned in-bounds row slots. The current fixture at [tests/secondary_index.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/tests/secondary_index.rs:776) only uses row position `99`, so it proves out-of-range detection but not a secondary entry pointing to a deleted in-bounds `None` row slot from [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:1049).
  - Impact: A core new invariant path introduced by `Vec<Option<Vec<Value>>>` is not pinned by deterministic negative fixture evidence.
  - Expected repair: Add a deterministic fixture that creates a committed secondary entry pointing at an in-bounds tombstoned/deleted row slot and asserts the exact `error: db check failed: secondary index\n` result.

- CR-003: QA-mapped unsupported mutation breadth is under-tested.
  - Severity: Medium
  - Must Fix Now: yes
  - Evidence: `qa_mapping.md` maps unsupported breadth and dependency validation to tests, but current added negative tests only cover missing `SET` column and type mismatch at [tests/secondary_index.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/tests/secondary_index.rs:694). New branches for primary-key-column update, non-primary-key `WHERE` on `UPDATE`/`DELETE`, and mutation on a table without an `INT PRIMARY KEY` are implemented in [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:816), [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:827), [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:878), and [src/sql.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/src/sql.rs:1093), but are not pinned by black-box tests.
  - Impact: The documented narrow mutation scope can regress or broaden silently while the current focused command remains green.
  - Expected repair: Add black-box CLI assertions for `UPDATE users SET id = ... WHERE id = ...`, `UPDATE ... WHERE age = ...`, `DELETE ... WHERE age = ...`, and mutation attempts on a table without a primary key.

- CR-004: Retained-WAL acceptance is partly proxy evidence.
  - Severity: Medium
  - Must Fix Now: no
  - Evidence: [tests/secondary_index.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/tests/secondary_index.rs:628) proves the page file and retained `.wal` sidecar exist before a reopen query/check, but the page file already contains the final `U`/`D` records. [tests/secondary_index.rs](/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency/tests/secondary_index.rs:658) proves WAL-only replay with manual frames, but not CLI-generated retained-WAL semantics.
  - Impact: This is not enough on its own to fail the gate because WAL-only replay coverage exists, but it should be tightened while repairing CR-001.

## Must Fix Now

- CR-001: Validate semantic committed WAL replay in `db check` and avoid persisting invalid SQL mutation WAL payloads into the base file before SQL invariant validation.
- CR-002: Add in-bounds deleted-row dangling secondary-entry negative fixture coverage.
- CR-003: Add black-box unsupported mutation breadth tests for primary-key updates, non-primary-key predicates, and no-primary-key tables.

## Residual Risks

- CR-R001: Tombstone row slots make reopen, `db check`, `CREATE INDEX` backfill, and secondary-index validation cost proportional to historical row slots, not just live rows. This is accepted as a residual V1 design risk for this slice because the spec requires stable row positions for secondary-index row pointers and does not set compaction or churn-performance acceptance criteria.
- `tests/db_check.rs` focused command is not required by the contract because all new negative secondary-index mutation fixtures remain in `tests/secondary_index.rs`.
- Dedicated maintainability, red-team, and database reviewer roles were unavailable, so those lenses were self-applied and cross-checked against invoked reviewer outputs.

## Next Action

Return to `code_review_retry` / implementation repair for CR-001, CR-002, and CR-003. The review phase itself is complete, so scheduler result is `retry`, not `continue`.

## Updated At

2026-05-19T14:40:00+0900

## Archived 2026-05-19T14:53:27+0900

Verdict: FAIL

## Scope

- Phase: `code_review_verify` round 2 for `task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency`.
- Baseline identification:
  - `git log --oneline main..HEAD`: no commits.
  - `git diff main...HEAD`: no committed branch diff.
  - Review target is the current uncommitted worktree diff plus task artifacts.
- Verified changed files: `src/check.rs`, `src/index.rs`, `src/sql.rs`, `src/storage.rs`, `tests/secondary_index.rs`, `docs/cli_contract.md`, `docs/file_format.md`, `docs/sql_subset.md`, and `specs/v1-secondary-index-mutation-consistency/`.
- Verification commands executed in this independent pass:
  - `cargo test --test secondary_index -- --nocapture`: exit 0; 33 passed, 0 failed.
  - `./scripts/verify`: exit 0; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, doc tests, and `cargo run --bin db -- --help` passed.
  - `git diff --check`: exit 0.
- Python-specific `pytest`, `ruff`, and `mypy` checks are not applicable: this repo contains no Python files and the required Rust static analysis is covered by `cargo clippy` through `./scripts/verify`.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
|---|---|---|---|---|---|---|
| code-reviewer | Correctness, regressions, completeness, merge safety | verified-prior-output | Prior `code_review.md` plus current source/test verification. Current product-code checks pass, but the latest code-review SSOT was not refreshed after fixes. | CRV-001 | CR-001, CR-002, CR-003 as current product-code findings | No new reviewer invoked in verify phase by instruction. |
| testing-reviewer | Coverage gaps and edge cases for prior CR-002/CR-003 | verified-prior-output | `tests/secondary_index.rs` now includes deleted-row slot coverage and unsupported mutation breadth tests; focused test command passed 33 tests. | CRV-001 | CR-002, CR-003 as current coverage findings | No new reviewer invoked in verify phase by instruction. |
| security-reviewer | WAL/check persistence boundary for prior CR-001 | verified-prior-output | `src/check.rs`, `src/storage.rs`, `src/sql.rs`, and `db_check_rejects_invalid_committed_mutation_wal_without_poisoning_base_file`; focused and baseline commands passed. | CRV-001 | CR-001 as current product-code finding | No new reviewer invoked in verify phase by instruction. |
| performance-reviewer | Tombstone/history-size residual risk | verified-prior-output | Prior CR-R001 remains a residual risk only; no spec performance bound or compaction requirement applies to this slice. | none | CR-R001 as merge blocker | No new reviewer invoked in verify phase by instruction. |
| maintainability-reviewer | Mutation logic in `src/sql.rs` | fallback-verified | Local read of update/delete application and replay validation paths found no new merge finding. | CRV-001 | none | Dedicated reviewer role not invoked in verify phase. |
| red-team-reviewer | Report/evidence drift and proxy-success risk | fallback-verified | The current product checks are green, but the latest code-review report still advertised old open findings and stale test counts. | CRV-001 | none | Dedicated reviewer role not invoked in verify phase. |
| database-reviewer | SQL/index/storage consistency | fallback-verified | Current WAL replay semantic validation, mutation tombstone behavior, secondary-index invariant fixtures, and docs were reviewed against the contract. | CRV-001 | CR-001, CR-002, CR-003 as current product-code findings | Dedicated reviewer role not invoked in verify phase. |
| api-reviewer | Endpoint/transport API changes | skipped | CLI-only Rust diff; no HTTP/API/DTO/status-code surface. | none | none | Not triggered. |
| ui-ux-reviewer | UI component/layout/accessibility changes | skipped | No UI surface in this Rust CLI diff. | none | none | Not triggered. |

## Findings

- CRV-001: Latest code-review SSOT was stale after the repair path.
  - Severity: High
  - Must Fix Now: yes
  - Evidence: Before this verification update, `specs/v1-secondary-index-mutation-consistency/code_review.md` still had `Verdict: FAIL`, `Updated At: 2026-05-19T14:40:00+0900`, and open Must Fix Now entries CR-001/CR-002/CR-003. Current source and test evidence no longer matches that report: `src/check.rs` now validates committed WAL payloads through SQL invariants before `db check` passes, `src/sql.rs` validates replayable records before normal open can persist invalid SQL WAL payloads, `tests/secondary_index.rs` includes the deleted-row dangling fixture and unsupported mutation breadth tests, and the focused test count is now 33 passed rather than the stale report's 29 passed.
  - Impact: The code-review phase cannot be accepted as completed because the canonical latest review report did not preserve an accurate current verdict/routing state after repair. Scheduler consumers would see old blocking findings or stale evidence and could not distinguish product-code closure from report drift.
  - Expected repair: Return to `code_review_retry` or the owning code-review reporting step to refresh the code-review SSOT with a current PASS or any true current findings, including current command evidence and routing evidence. No product-code regression was found in this verify pass.

## Must Fix Now

- CRV-001: Refresh the owning code-review report/evidence state so the latest `code_review.md` represents the current branch state rather than the pre-repair FAIL report.

## Residual Risks

- Stable tombstone row slots keep reopen, `db check`, and index rebuild work proportional to historical row slots. This remains accepted as a residual V1 limitation for this slice because the spec requires stable row positions and does not require compaction.
- Retained-WAL CLI evidence still includes page-file records, but the separate WAL-only `U`/`D` replay test closes the replay-required proof gap.
- No `tests/db_check.rs` focused command is contract-required because the new secondary-index mutation negative fixtures remain in `tests/secondary_index.rs`; the full baseline did run `tests/db_check.rs` successfully.

## Next Action

Return to `code_review_retry` for report/evidence refresh. Product-code tests and static analysis passed in this verification pass, so the retry target is the stale code-review SSOT unless the owning review phase finds a new current issue.

## Updated At

2026-05-19T14:48:24+0900
