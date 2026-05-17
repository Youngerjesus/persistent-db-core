# QA Prep Verification Review History

## 2026-05-17 qa_prep_verify_1

retry

# QA Prep Verification Review

Verdict: retry

## Must Fix Now

- Add the missing canonical QA prep latest review report as part of QA prep output. This file was absent at verification start, so the phase artifact set was incomplete.
- Strengthen T2 old row-only catalog compatibility evidence. `primary_index_existing_row_only_catalog_remains_insert_order` currently creates the fixture through the CLI no-PK path. After implementation, that may write the new catalog encoding and fail to prove compatibility with existing row-only SQL catalog records. Add a direct old-format fixture that appends the current catalog record body with no primary-key extension plus row records, then verifies reopen/select keeps insert-order behavior.
- Close the invalid predicate boundary promised by `qa_mapping.md`. The mapping lists unsupported non-PK `WHERE` and unsupported range/order behavior, but the scaffold only pins `TEXT PRIMARY KEY`. Add focused SQL exec tests, preferably under the `primary_key` filter, proving non-primary-key predicates and range/order-style predicates do not silently full-scan or get treated as supported primary-key lookups.

## Verification Notes

- `tasks.md` T1-T5 are represented in `qa_mapping.md`, with concrete preferred commands and behavior-specific green criteria.
- Red evidence was reproduced:
  - `cargo test --test primary_index` fails with unresolved import `persistent_db_core::index`, matching the intended pre-implementation red state.
  - `cargo test --test sql_exec primary_key` runs 7 filtered tests, with the existing non-PK insert-order test passing and the primary-key grammar/behavior tests failing because the feature is not implemented yet.
- The current scaffold covers happy path, duplicate insert, missing lookup, empty scan, duplicate persisted PK row, restart/rebuild, `TEXT PRIMARY KEY`, and non-PK insert-order behavior.
- The remaining gaps are contract-shaping gaps, not implementation failures: old-format catalog compatibility and unsupported predicate boundaries are not yet pinned strongly enough for `impl_exec` to consume without extra judgment.
