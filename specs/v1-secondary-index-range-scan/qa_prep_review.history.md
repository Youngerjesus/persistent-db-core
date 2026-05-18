# QA Prep Verification Review History

## Archived before qa_prep_verify_2_fresh_20260519_020943_108129_b210442f

# QA Prep Verification Review: v1-secondary-index-range-scan

Verdict: retry

## Summary

The QA package has a task-to-test manifest for T1 through T8, concrete preferred commands, and useful red evidence. It is not yet sufficient for `impl_exec` to consume without additional judgment because the red scaffold does not lock two contract-critical areas: actual secondary-index query path use and the durable-format corruption/retry matrix.

Observed red command:
- `cargo test --test secondary_index -- --nocapture`
- Exit code: `101`
- Current red reason: `error[E0432]: unresolved import persistent_db_core::index::SecondaryIndex`

## Must Fix Now

1. Add explicit secondary query path-use scaffold.
   - Contract risk: `contracts.md` requires secondary equality and `BETWEEN` to use the actual secondary index path; full table scan with matching output must not pass.
   - Current gap: `qa_mapping.md` lists this at T1/T4, but `tests/secondary_index.rs` only checks unsupported pre-index predicate and black-box output after index creation. That does not fail a full-table-scan implementation after `CREATE INDEX`.
   - Required repair: add a red test that asserts an implementation-visible planner/path marker or helper returns `SecondaryIndexEquality` for `SELECT * FROM users WHERE age = 20;` and `SecondaryIndexRange` for `SELECT * FROM users WHERE age BETWEEN 10 AND 20;`, or another equally explicit non-user-facing path evidence point grounded in `design.md`.

2. Close the durable secondary-index corruption matrix in the scaffold or mark exact blockers.
   - Contract risk: T2/T6 require deterministic validation of missing entry, wrong key, wrong tie-break, invalid row position, duplicate entry, missing table/column metadata, and malformed `I` entries including missing, extra, wrong-index, wrong-key, wrong-tie-break, and wrong-row-position.
   - Current gap: the scaffold covers old no-index reopen/backfill, orphan `E`, one wrong-key committed `E`, and one partial multi-index `I`, but `qa_mapping.md` says implementation may add narrower corruption cases if requested.
   - Required repair: add focused red fixture tests for the missing corruption cases, or explicitly document any infeasible case as a blocker with the exact reason. Do not leave required corruption coverage to implementation discretion.

3. Add explicit post-index insert atomicity evidence.
   - Contract risk: post-index inserts must use exactly one `I` record with all embedded entries, not `R + standalone E`, and failed/interrupted post-index insert retry must be deterministic.
   - Current gap: `tests/secondary_index.rs` checks query results after a post-index insert and has one malformed partial `I` fixture, but does not prove a successful post-index insert emits one `I` record and not `R + E`, nor does it scaffold failed/interrupted insert retry behavior.
   - Required repair: add a raw-record or reopen/check test that proves successful post-index insert durability uses a single atomic `I` record for all committed indexes, plus a retry/interruption scenario or an explicit blocker if the failure injection hook is unavailable in QA prep.

4. Align `qa_mapping.md` with the repaired scaffold.
   - Replace notes such as "Implementation may add narrower corruption cases if reviewer requests more granularity" with concrete test names or explicit blockers.
   - Keep `Preferred Commands` concrete: `cargo test --test secondary_index -- --nocapture` for focused QA and `scripts/verify` for baseline/doc contract checks.
   - Keep the red evidence current after scaffold repair.

## Already Sufficient

- T1 through T8 all have mapping entries.
- Required command names are concrete and runnable.
- Exact `CREATE INDEX` semantic errors are pinned.
- Required equality and inclusive range stdout examples are pinned.
- Old no-index reopen/backfill and orphan `E` retry scenarios have useful starting scaffolds.
- This is correctly treated as a non-visual CLI/database task; browser, screenshot, and UX evidence are not accepted.

## Retry Exit Criteria

QA prep can pass verification when:
- `tests/secondary_index.rs` contains explicit red tests for secondary equality/range path use.
- Required durable corruption and atomic `I` scenarios are either scaffolded or explicitly blocked with precise reasons.
- `qa_mapping.md` names the repaired test coverage under the relevant T1, T2, T4, T5, and T6 entries.
- Red evidence is refreshed and still fails for expected pre-implementation reasons.

## Archived before qa_prep_verify_3_fresh_20260519_021643_988905_c2dad83e

# QA Prep Verification Review: v1-secondary-index-range-scan

Verdict: retry

## Summary

The retry repair closed the prior broad gaps for path-use evidence, committed secondary-index corruption coverage, malformed `I` coverage, and post-index atomic `I` persistence. The package is still not sufficient for `impl_exec` because the interrupted backfill retry and SQL range boundary scaffolds do not yet lock the exact plan behavior.

Observed red commands:
- `cargo test --test secondary_index -- --nocapture`
- Exit code: `101`
- `scripts/verify`
- Exit code: `101`
- Current red reason: unresolved imports for `persistent_db_core::index::SecondaryIndex`, `persistent_db_core::sql::plan_query_path_for_test`, and `persistent_db_core::sql::QueryPath`, which is an expected pre-implementation red state.

## Must Fix Now

1. Add a discriminating interrupted-backfill retry scaffold.
   - Contract risk: `design.md`, `plan.md`, and T3.6 require orphan `E` records from an interrupted build to be ignored, retry to commit with a new `build_id`, and queries to use only the retried committed entries.
   - Current gap: `orphan_backfill_entries_are_ignored_and_same_name_retry_can_commit` uses an orphan `E` whose key, tie-break, and row position match the eventual valid entry. An implementation that accidentally reuses the stale orphan entry or commits metadata with the orphan build id can still pass.
   - Required repair: change or add a red fixture where the orphan `E` has a wrong key/tie-break or otherwise distinguishable stale content, then retry `CREATE INDEX` with the same index name and assert `db check` plus equality/range output prove the stale orphan was ignored. Also inspect raw records enough to prove the committed retry `X` uses a fresh `build_id` and the committed backfill records are the new `E` set followed by final `X`.

2. Add SQL-level range boundary and missing-index range scaffolds.
   - Contract risk: `design.md` states `BETWEEN` with `low > high` must return header-only output through the index path, and non-primary-key equality/range predicates without a committed secondary index must be unsupported rather than full-scanned.
   - Current gap: `SecondaryIndex::range_positions(30, 20)` covers only the primitive. There is no black-box SQL test for `SELECT * FROM users WHERE age BETWEEN 30 AND 20;` after `CREATE INDEX`, and no pre-index `BETWEEN` unsupported test.
   - Required repair: add a post-index CLI test expecting header-only stdout for a low-greater-than-high `BETWEEN`, with the planner/path marker still proving `SecondaryIndexRange`. Add a pre-index `BETWEEN` test expecting exit `2`, empty stdout, and the unsupported SQL stderr for the exact statement.

3. Align `qa_mapping.md` with the repaired scaffold.
   - Name the new interrupted-backfill discriminator, fresh-build-id/order assertion, low-greater-than-high SQL boundary, and pre-index `BETWEEN` unsupported tests under T3/T4.
   - Keep `Preferred Commands` concrete: `cargo test --test secondary_index -- --nocapture` for focused QA and `scripts/verify` for baseline/doc contract checks.
   - Keep the red evidence current after scaffold repair.

## Already Sufficient

- T1 through T8 all have mapping entries.
- Required command names are concrete and runnable.
- Exact `CREATE INDEX` semantic errors are pinned.
- Required equality and inclusive range stdout examples are pinned.
- Explicit secondary equality/range path-use scaffold is present.
- Committed `E`/`X` corruption matrix is present.
- Malformed `I` embedded-entry matrix is present.
- Successful post-index insert atomic `I` shape and no-durable-`I` retry scaffolds are present.
- Old no-index reopen/backfill has useful starting scaffold.
- This is correctly treated as a non-visual CLI/database task; browser, screenshot, and UX evidence are not accepted.

## Retry Exit Criteria

QA prep can pass verification when:
- `tests/secondary_index.rs` contains a discriminating interrupted-backfill retry test that would fail if stale orphan entries attach to the retried committed index.
- `tests/secondary_index.rs` proves retry `CREATE INDEX` uses a fresh committed `build_id` and records the new backfill `E` set before final `X`.
- `tests/secondary_index.rs` contains SQL-level low-greater-than-high `BETWEEN` and pre-index `BETWEEN` unsupported tests.
- `qa_mapping.md` names the repaired test coverage under T3 and T4.
- Red evidence is refreshed and still fails for expected pre-implementation reasons.
