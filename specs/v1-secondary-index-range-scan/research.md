# Research: v1-secondary-index-range-scan

## Context
The current repo already has V1 page storage, SQL logical catalog/row records, primary-key metadata, in-memory primary index rebuild, WAL replay/checking, and deterministic CLI tests. Secondary indexes must add a disk-backed query path for `CREATE INDEX <name> ON <table>(<integer_column>)`, equality predicates, and inclusive `BETWEEN` range scans.

## Decisions

### R1. Persist secondary index metadata and index contents as SQL logical records
Decision: add SQL logical records for secondary index metadata and index entries rather than only rebuilding from row records.

Rationale:
- The contract requires disk-backed secondary index metadata/storage encoding.
- `db check` must detect deterministic mismatch between table rows and secondary index contents. If the index were only rebuilt from rows on every open, metadata/content mismatch would not be representable.
- The current page store already supports append-only opaque SQL logical records and WAL protection. New logical records can reuse this durable path without changing page framing.

Planned shape:
- `X` record: committed index metadata, including build id, index name, table name, indexed column ordinal, and tie-break mode.
- `E` record: backfill index entry, including build id, index name, indexed INT key, tie-break key, and durable row position.
- `I` record: atomic indexed insert row, including the row values and all required secondary index entries for that row.

### R2. Use `BTreeMap<(index_key, tie_break), row_position>` for the in-memory secondary index path
Decision: implement a secondary index primitive in `src/index.rs` backed by `std::collections::BTreeMap`.

Rationale:
- Equality lookup can select a key range for `index_key == value`.
- Inclusive bounded range scan can select `(low, min_tie)..=(high, max_tie)`.
- BTreeMap provides deterministic ascending key order using the standard library.
- The existing primary index already uses BTreeMap, so this follows local style.

### R3. Tie-break values are explicit index entry data
Decision: store the tie-break value in each index entry.

Rationale:
- Primary-key tables require ascending primary-key tie-break.
- Tables without a primary key require durable row insertion order tie-break.
- Storing the tie-break in the entry lets `db check` compare durable index contents against durable table rows exactly and makes query ordering independent of incidental in-memory iteration.

### R4. `CREATE INDEX` uses entries-first commit records with a deterministic build id
Decision: `CREATE INDEX` validates the target table/column, assigns a deterministic `build_id`, appends backfill `E` entry records for existing rows, appends one final `X` metadata commit record, and only then registers the index in the database model. Later inserts into indexed tables use the atomic `I` record policy defined in R7.

Rationale:
- Backfill is required by the compatibility criterion.
- Query-path evidence can be made observable with an internal test that calls the planner/execution helper and asserts the selected path is secondary-index equality/range rather than table scan.
- The append-only storage model has no updates or deletes in this slice, so no tombstone/update strategy is needed.
- The `build_id` prevents interrupted backfill entries from being attached to a later retry with the same index name.

Durable state machine:
- `build_id` is `u64` equal to the durable SQL logical record count visible when `CREATE INDEX` starts, before any new backfill entries are appended.
- Backfill entries are written as `E(build_id, index_name, ...)`.
- The index becomes committed only when the final `X(build_id, index_name, table, column, tie_break_mode)` record is durable.
- On reopen, `E` records are scoped by `(case-insensitive index_name, build_id)` and attach only to an `X` record with the same pair.
- Orphan `E` records without a matching committed `X` are ignored by `db exec` and `db check`; they represent an interrupted, uncommitted index build.
- Retry after an interrupted build with the same index name is allowed. Because orphan `E` records increase the durable record count, the retry receives a different `build_id`, writes a fresh `E` set, writes a final `X`, and commits cleanly.
- A committed `X` without exactly matching row entries for its `(index_name, build_id)` is a `secondary index` invariant failure.

### R5. Parser support is narrow
Decision: parse only:
- `CREATE INDEX <index_name> ON <table_name>(<column_name>);`
- `SELECT * FROM <table> WHERE <indexed_int_column> = <int>;`
- `SELECT * FROM <table> WHERE <indexed_int_column> BETWEEN <int> AND <int>;`

Rationale:
- This is the smallest contract-complete grammar.
- Existing unsupported/malformed behavior must remain stable outside this slice.
- Primary-key equality remains supported; non-indexed predicates should not silently fall back to a full table scan for this task's required indexed predicates.
- Before a secondary index exists, `SELECT * FROM users WHERE age = 20;` on a non-primary-key `age` column remains unsupported SQL with exit code `2`, empty stdout, and the unsupported SQL stderr shape for the exact statement. This preserves the current no-full-scan behavior and gives the tests a clear regression target.

### R6. Lock `X` and `E` encoding to little-endian binary fields
Decision: use the exact binary layout in `plan.md` and `design.md` for `X`, `E`, and `I`; do not allow decimal-string alternates.

Rationale:
- The file-format docs and fixture helpers need one final byte layout.
- `build_id` and row positions are storage metadata, not SQL values, so fixed little-endian integers match the page/WAL format style.
- Index, table, and column names preserve user catalog spelling in payload bytes, while name comparisons remain ASCII case-insensitive during validation and lookup.

### R7. Post-index `INSERT` uses one atomic indexed-row record
Decision: after one or more committed secondary indexes exist on a table, `INSERT` appends a single `I` logical record instead of appending `R` plus separate `E` records. The `I` record contains the inserted row values and one embedded index entry for every committed secondary index on that table.

Rationale:
- This preserves the existing SQL failure contract that a failing statement appends no partial SQL logical record: a post-index `INSERT` has exactly one logical append.
- It avoids row-with-missing-entry and multi-index partial-entry states because row and entries are committed together.
- It avoids `E`-without-row states because standalone `E` records are reserved for `CREATE INDEX` backfill only.
- It keeps the lower-level page and WAL framing unchanged by using the existing single-record append path.

Durable state machine:
- Tables without committed secondary indexes continue to use existing `R` row records.
- Tables with at least one committed secondary index use `I` records for new inserts.
- Before appending, the executor computes the next durable row position and all embedded entries for the committed indexes on that table.
- If encoding would exceed one page-store record, the statement fails before append with the existing storage error surface and no logical row/index content is durable.
- If `PageStore::append_record(I)` returns an error, runtime state is not updated; because there is one logical record, there is no representable partial SQL logical record for that statement.
- On crash after the WAL frame for `I` is durable, WAL replay may make the entire indexed row durable; this is a committed statement after recovery, not a partial row/index state.
- Retrying after a failed append is safe because either no `I` exists, or a whole `I` exists and normal table constraints determine the retry result.
- `db check` treats malformed `I` records, missing embedded entries, wrong keys/tie-breaks, unknown index references, duplicate embedded entries, or an embedded entry set that does not exactly match the table's committed secondary indexes as `secondary index` invariant failures.

## Compatibility Findings
- Existing no-index SQL databases contain only catalog/row records and optional primary-key catalog extensions. They must continue to reopen.
- New decoders must accept old catalog/row records unchanged.
- New index records must be validated after rows and metadata are reconstructed.
- Orphan `E` records from interrupted uncommitted builds are compatible ignored records, not committed index contents.
- Existing old no-index databases never contain `I` records and continue to decode through `R` records.
- New databases may contain both `R` records before index creation and `I` records after index creation for the same table; both contribute durable rows in append order.
- `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md` should all describe the new SQL logical records and query behavior, even though only `docs/file_format.md` and `docs/cli_contract.md` are named in the task's intended touches. Updating `docs/sql_subset.md` is a narrow adjacent durable-doc update because it owns SQL grammar.

## Risks
- Appending many index entry records during `CREATE INDEX` can create partial logical states if a process crashes mid-command. The selected policy is entries first, final `X` commit, `build_id` scoping, orphan `E` ignored, and same-name retry allowed with a new `build_id`.
- Post-index `INSERT` must not be implemented as `R` plus standalone `E` records. It must use the single `I` record policy to preserve statement-level no-partial-record behavior.
- Exact error text is contract-sensitive. `CREATE INDEX` missing-table must use the new hint mentioning CREATE INDEX without changing existing INSERT/SELECT tests unless intentionally updated.
- `BETWEEN` parsing must reject malformed/range predicates outside the exact approved shape while preserving existing primary-key unsupported cases where appropriate.
