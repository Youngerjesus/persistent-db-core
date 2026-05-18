# Plan Review: v1-secondary-index-range-scan

## Verdict
resolved_for_re_review

## Review Scope
- `spec.md`
- `contracts.md`
- `research.md`
- `plan.md`
- `design.md`
- `tasks.md`
- `readiness.md`
- current `src/` SQL/index/check/storage shape
- current durable docs for SQL statement failure and `db check`

## Must Fix Before Implementation

### [x] 1. Make post-index `INSERT` multi-record persistence decision-complete
The previous retry findings for `CREATE INDEX` build/reopen/retry state, `X`/`E` encoding, and no-full-scan predicate behavior are resolved. However, the refreshed artifacts introduce a second multi-record mutation path without a durable state policy: after an index exists, `INSERT` appends one row record `R` and then one `E` index-entry record per secondary index on the table.

Why this blocks implementation:
- `plan.md` says `INSERT` appends the row and then appends index entry records for every secondary index.
- `design.md` says in-memory rows and indexes update only after durable appends succeed.
- Existing docs state a failing SQL statement appends no partial SQL record.
- If the process or storage write fails after `R` is durable but before all required `E` records are durable, reopen sees a committed row with missing committed-index entries. The current plan does not say whether this is recoverable, ignored, a deterministic corruption, or a contract-breaking partial failed `INSERT`.
- If an alternative implementation writes `E` before `R`, the plan also does not define how to prevent orphan future-row entries from later attaching to a different row position.
- Multiple secondary indexes make the partial state larger than one missing entry: `R + E(index_a)` can be durable while `E(index_b)` is missing.

Required repair:
- Pick exactly one durable state machine for post-index `INSERT` when one or more secondary indexes exist.
- Define record ordering and commit semantics for `R` row records and all required `E` index-entry records.
- Define reopen and `db check` behavior for partial post-index `INSERT` states, including row-with-missing-`E`, `E`-without-row if that ordering is selected, and multi-index partial writes.
- Define CLI behavior when an append fails mid-`INSERT`, and either preserve the existing "failing statement appends no partial SQL record" contract or explicitly mark the required escalation/doc contract change.
- Add concrete task/test bullets for post-index `INSERT` interruption/reopen, deterministic `db check`, and retry behavior after a failed or interrupted post-index `INSERT`.
- Align `research.md`, `plan.md`, `design.md`, `tasks.md`, `analysis_report.md`, and durable doc tasks with the selected policy.

Resolution:
- Fixed in `research.md`, `plan.md`, `design.md`, `tasks.md`, `analysis_report.md`, and `readiness.md`.
- Selected policy: tables without committed secondary indexes continue to append one `R` row record; tables with committed secondary indexes append exactly one atomic `I` indexed-row record containing row values plus all embedded entries for that row.
- `R + standalone E` is explicitly out of plan for post-index inserts, so row-with-missing-entry, E-without-row, and multi-index partial-entry states are not valid encodings.
- If `I` encoding or append fails, runtime state is not updated and the existing no-partial-SQL-record statement contract is preserved. On crash after a WAL frame is durable, the whole `I` may recover as one committed row/index unit, not a partial state.
- Reopen and `db check` validate `I` records atomically; malformed/missing/extra/wrong embedded entries fail with deterministic `secondary index`.
- Tasks now require concrete tests for atomic `I` reopen, failed/interrupted insert retry, multi-index insert, and corrupted `I` `db check`.

## Already Good
- The `CREATE INDEX` backfill state machine is now decision-complete: deterministic `build_id`, `E` records first, final `X` commit marker, orphan `E` ignored, and same-name retry allowed with a new `build_id`.
- The `X`, `E`, and `I` byte layouts are now fixed-width little-endian for storage metadata and no longer leave decimal-string alternatives.
- Non-primary-key predicates before `CREATE INDEX` are explicitly unsupported SQL, which gives implementation a concrete no-full-scan regression target.
- Required CLI examples, exact `CREATE INDEX` errors, ordering, tie-breaks, persisted compatibility, `db check`, docs, and required commands are represented in the task breakdown.
- Non-visual evidence handling is correct: DOM capture, screenshots, rendered route state, and UX review evidence are excluded.

## Scenario Review Decision
Used `scenario-brake` because this plan has important reopen/retry recovery paths and multiple durable states. The original verdict was `[SCENARIOS MISSING]` for post-index `INSERT`; this retry resolves it by selecting the atomic `I` record policy and adding concrete normal insert maintenance, interrupted insert/retry, multi-index insert, reopen, and `db check` tasks.

## Required Result For Re-Review
- Update `research.md`, `plan.md`, `design.md`, `tasks.md`, `analysis_report.md`, and `readiness.md` so the post-index `INSERT` partial-write policy is explicit and testable.
- Keep `spec.md` and `contracts.md` frozen unless an explicit escalation changes the task contract.
