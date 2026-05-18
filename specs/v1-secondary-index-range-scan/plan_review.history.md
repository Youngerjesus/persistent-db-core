# Plan Review History: v1-secondary-index-range-scan

## 2026-05-19 01:51 KST - Previous Latest Review Archived

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
- current `src/` SQL/index/check shape

## Must Fix Before Implementation

### [x] 1. Make secondary-index build/reopen/retry state decision-complete
The current artifacts do not give one executable policy for multi-record `CREATE INDEX` persistence.

Conflicts and gaps:
- `research.md` R4 says `CREATE INDEX` appends metadata, then entries.
- `plan.md` and `design.md` say entries are appended first, then final metadata.
- `plan.md` leaves orphan `E` records as an implementation choice: ignore them or treat them as invalid.
- With the entries-first/final-metadata approach, an interrupted backfill can leave orphan `E` records. If the user later retries the same `CREATE INDEX`, a decoder that attaches pending entries by index name can accidentally attach old orphan entries plus retry entries to the final metadata, producing duplicate or incorrect index contents.

Required repair:
- Pick exactly one durable state machine for `CREATE INDEX`.
- Define record ordering and commit semantics for `X` metadata and `E` entries.
- Define how reopen scopes `E` records to a committed metadata record.
- Define whether orphan/incomplete index records are ignored or reported by `db check`.
- Define retry behavior after an interrupted `CREATE INDEX` with the same index name.
- Add concrete task/test bullets for interrupted backfill reopen, `db check`, and retry or deterministic failure behavior.

Resolution:
- Fixed in `research.md`, `plan.md`, `design.md`, `tasks.md`, `analysis_report.md`, and `readiness.md`.
- Selected policy: `build_id = durable SQL logical record count at CREATE INDEX start`, append `E` entries first, append final `X` commit record, attach entries only by `(case-insensitive index_name, build_id)`, ignore orphan `E` records in `db exec` and `db check`, and allow same-name retry with a new `build_id`.

### [x] 2. Lock the persisted `X`/`E` byte encoding
`plan.md` describes a little-endian binary record layout but also says the implementation may encode signed integers as canonical decimal strings. That leaves file-format work to implementation judgment and weakens the required docs/tests alignment.

Required repair:
- Choose the final `X` and `E` record byte layout in the plan/design.
- Specify integer encoding, field order, lengths, and endianness.
- State how index names and table/column names are normalized or preserved.
- Ensure `docs/file_format.md` and fixture helpers are explicitly tied to that exact layout.

Resolution:
- Fixed in `research.md`, `plan.md`, `design.md`, and `tasks.md`.
- Selected encoding: fixed-width little-endian binary fields for `build_id`, `indexed_key`, `tie_break`, and `row_position`; `u16` little-endian UTF-8 byte lengths for names; preserved name spelling in records; ASCII case-insensitive name comparisons.

### [x] 3. Specify the non-indexed predicate behavior used to prove no full scan
The contract says indexed equality/range must use the secondary index and must not pass via full table scan. The plan says non-indexed predicates must not silently full-scan, but it does not lock the CLI behavior when a valid table/column has no secondary index.

Required repair:
- Choose the expected behavior for `SELECT * FROM users WHERE age = 20;` before `CREATE INDEX`.
- Prefer preserving the current unsupported-SQL surface unless the contract is intentionally expanded.
- Add a focused regression test so a full-scan fallback cannot satisfy the acceptance examples accidentally.

Resolution:
- Fixed in `research.md`, `plan.md`, `design.md`, `tasks.md`, `analysis_report.md`, and `readiness.md`.
- Selected behavior: before `CREATE INDEX`, a non-primary-key predicate such as `SELECT * FROM users WHERE age = 20;` remains unsupported SQL with exit code `2`, empty stdout, and unsupported SQL stderr for the exact statement. Tasks now require a focused regression test.

## Already Good
- Scope stays within the approved secondary-index slice and does not close sibling `gate-v1-indexes` requirements.
- Required CLI examples, exact `CREATE INDEX` errors, ordering, tie-breaks, persisted compatibility, `db check`, docs, and required commands are all represented in the task breakdown.
- Non-visual evidence handling is correct: DOM capture, screenshots, rendered route state, and UX review evidence are excluded.

## Scenario Review Decision
Used `scenario-brake` because this plan has important reopen/retry recovery paths and multiple durable states. Verdict: scenarios missing until the interrupted backfill/orphan-entry/retry path is explicitly separated from normal create, normal reopen, and corrupt-index validation.

## Required Result For Re-Review
- Update `research.md`, `plan.md`, `design.md`, and `tasks.md` so the three findings above are resolved without leaving implementation choices.
- Keep `spec.md` and `contracts.md` frozen unless an explicit escalation changes the task contract.
