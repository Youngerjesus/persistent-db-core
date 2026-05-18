# Implementation Plan: v1-secondary-index-range-scan

## Goal
Add `CREATE INDEX` for secondary `INT` columns, disk-backed secondary index metadata and contents, indexed equality lookup, inclusive bounded `BETWEEN` range scan, deterministic ordering, restart persistence, `db check` invariant validation, and durable documentation.

## Non-Goals
- Closing sibling requirements under `gate-v1-indexes`.
- UPDATE, DELETE, index maintenance for mutation beyond INSERT.
- Multi-column indexes.
- Unique secondary indexes.
- TEXT secondary indexes.
- ORDER BY, projection, joins, general predicates, or optimizer breadth.
- Network services, daemons, external services, browser evidence, screenshots, or UX design review evidence.

## Affected Contract Surface
- CLI behavior: `db exec` success/error stdout, stderr, and exit codes for `CREATE INDEX`, indexed equality, and `BETWEEN`.
- SQL grammar: add narrow `CREATE INDEX` and indexed `WHERE` equality/range predicates.
- Persisted data compatibility: old no-index databases reopen; new secondary index metadata and entries persist.
- File-format docs: SQL logical records must document index metadata and entry records.
- `db check`: validate secondary index metadata and contents against durable rows.
- Tests: add `tests/secondary_index.rs` and update focused SQL integration coverage.

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| `src/index.rs` | Add secondary index primitive over `BTreeMap<(i64, TieBreak), usize>` | std only; deterministic equality/range APIs |
| `src/lib.rs` | Keep existing module export; no new dependency boundary needed unless index helper type is public | avoid broad public API |
| `src/sql.rs` | Parse `CREATE INDEX`, equality, and `BETWEEN`; persist/decode index records; route indexed predicates through secondary index | preserve existing primary-key and row-only behavior |
| `src/check.rs` | Usually no structural change; continue delegating SQL invariant labels through `validate_records_for_check` | failure label must be deterministic |
| `src/main.rs` | Update SQL unsupported hint only if tests/docs require it | preserve exit-code mapping |
| `tests/secondary_index.rs` | New focused black-box and internal evidence for index path, persistence, db check | required command target |
| `tests/sql_exec.rs` | Add or adjust CLI examples/errors where broad SQL contract coverage belongs | avoid duplicating all secondary scenarios |
| `docs/file_format.md` | Document secondary index metadata/storage encoding and check validation | durable compatibility contract |
| `docs/cli_contract.md` | Document `CREATE INDEX`, equality/range behavior, exact errors, ordering | stable CLI contract |
| `docs/sql_subset.md` | Narrow adjacent update for grammar and logical records | required because it owns supported SQL subset |

## Data Model Plan
- Table model gains `secondary_indexes: Vec<SecondaryIndexState>`.
- `SecondaryIndexState` contains preserved index name, table name reference, indexed column ordinal/name, and in-memory `SecondaryIndex`.
- `SecondaryIndex` maps `(indexed_key, tie_break)` to row position.
- `TieBreak` is:
  - primary-key value for tables with an `INT PRIMARY KEY`;
  - durable row insertion position for tables without a primary key.
- Existing rows remain `Vec<Vec<Value>>` in durable insertion order.
- Existing primary-key index remains responsible for primary-key exact lookup and full-table primary-key ordering.

## Persisted Encoding Plan
Keep lower-level page and WAL framing unchanged. Add SQL logical records above `PageStore`:

```text
PDBSQL1\0
X
u64 build_id little-endian
u16 index_name_len
index_name UTF-8 bytes
u16 table_name_len
table_name UTF-8 bytes
u16 indexed_column
u8 tie_break_mode: P for primary-key value, R for row insertion order
```

```text
PDBSQL1\0
E
u64 build_id little-endian
u16 index_name_len
index_name UTF-8 bytes
i64 indexed_key little-endian
i64 tie_break little-endian
u64 row_position little-endian
```

```text
PDBSQL1\0
I
u16 table_name_len
table_name UTF-8 bytes
u16 value_count
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

Encoding is final for implementation planning:
- All integer fields in `X`, `E`, and `I` records are fixed-width little-endian binary fields.
- `index_name`, `table_name`, and existing catalog column names are stored as UTF-8 bytes with `u16` byte lengths and preserved spelling.
- Name comparison for duplicate-index, metadata attachment, and lookup is ASCII case-insensitive.
- `indexed_column` is a zero-based ordinal into the table catalog.
- `I` records store SQL row values using the same type/value encoding as `R` records, then append the complete embedded entry set for that row.
- Fixture helpers and `docs/file_format.md` must use this exact field order and encoding.

Build/reopen/retry state machine:
- At `CREATE INDEX` start, compute `build_id` as the current durable SQL logical record count before appending any new records.
- Append one `E(build_id, index_name, key, tie_break, row_position)` record for each existing row.
- Append one final `X(build_id, index_name, table_name, indexed_column, tie_break_mode)` metadata record. This `X` record is the commit marker.
- On reopen/check, attach entries only by exact `(case-insensitive index_name, build_id)` match to a committed `X` record.
- Orphan `E` records with no matching `X` are ignored by both `db exec` and `db check`; they are uncommitted interrupted builds.
- Retry after an interrupted same-name `CREATE INDEX` is allowed. The retry sees a larger durable record count, receives a new `build_id`, writes a fresh entry set, and commits with a new `X`; old orphan entries cannot attach to it.
- A committed `X` with missing, duplicate, wrong-key, wrong-tie-break, or invalid-row-position `E` records fails deterministic `secondary index` validation.

Post-index `INSERT` state machine:
- If the target table has no committed secondary indexes, `INSERT` keeps the existing single `R` row-record append behavior.
- If the target table has one or more committed secondary indexes, `INSERT` must append exactly one `I` indexed-row record containing the row values plus one embedded entry for each committed secondary index on that table.
- The `I` record is the commit unit. There is no durable `R + E + E...` sequence for post-index inserts.
- Before append, compute the new row position as the current durable row count for that table, compute the primary-key or insertion-order tie-break, and compute all embedded entries.
- If `I` encoding is too large for one page-store record, fail before append through the existing storage error surface and do not update runtime state.
- If the append call fails, do not update runtime state; because no partial SQL logical record exists, the existing "failing statement appends no partial SQL record" contract is preserved.
- On reopen/check, an `I` record contributes exactly one durable row and all its embedded index entries atomically.
- A malformed `I`, an `I` with missing/extra/wrong embedded entries for the table's committed secondary indexes, or an embedded entry pointing to the wrong row position fails deterministic `secondary index` validation.
- Retrying after an interrupted or failed post-index insert is deterministic: either no `I` record was durable and retry performs the insert, or a whole `I` record was recovered and normal table constraints/output reflect that committed row.

## Query Path Mapping
- `db exec` opens `PageStore`, reads records, and reconstructs tables, primary indexes, secondary index metadata, and secondary index contents.
- `CREATE INDEX` validates table, column existence, column type `INT`, and duplicate index name case-insensitively.
- Successful `CREATE INDEX` appends durable index entries for all existing rows and durable index metadata; stdout/stderr remain empty.
- `INSERT` into a table without secondary indexes appends the existing `R` row record.
- `INSERT` into a table with committed secondary indexes appends one atomic `I` indexed-row record with all required embedded entries; it must not append `R` plus standalone `E` records.
- `SELECT * FROM table WHERE indexed_col = value` resolves a secondary index on `indexed_col` and uses its equality API.
- `SELECT * FROM table WHERE indexed_col BETWEEN low AND high` resolves a secondary index on `indexed_col` and uses its inclusive range API.
- Result ordering is secondary key ascending, then tie-break ascending.
- A missing secondary index for a non-primary-key predicate is unsupported SQL, not a full table scan. Example before `CREATE INDEX`: `SELECT * FROM users WHERE age = 20;` exits `2`, writes empty stdout, and writes the unsupported SQL stderr for that exact statement.

## Test Strategy
1. Add failing `tests/secondary_index.rs` scenarios for the required success examples, exact error strings, persistence/backfill, interrupted build/retry, tie-break ordering, and `db check`.
2. Add an internal or observable path-use test that fails if indexed equality/range falls back to table scan. Preferred evidence: expose a small planner/path enum under test-only or crate-visible API and assert `SecondaryIndexEquality` and `SecondaryIndexRange`.
3. Implement the secondary index primitive and focused unit/integration tests.
4. Implement SQL parsing/execution/storage/check integration.
5. Update durable docs after behavior and encoding are final.
6. Run required verification commands and map evidence in final report.

## Required Verification
- `scripts/verify`
- `cargo test --test secondary_index -- --nocapture`

## Acceptance Mapping
| Acceptance Item | Planned Evidence |
|---|---|
| `CREATE INDEX <name> ON <table>(<integer_column>)` success | `tests/secondary_index.rs` CLI success assertions; `docs/cli_contract.md` |
| Missing table/column, unsupported type, duplicate index exact errors | `tests/secondary_index.rs` exact stderr matrix |
| Equality example output | `tests/secondary_index.rs` black-box fixture with required rows |
| Inclusive `BETWEEN` output | `tests/secondary_index.rs` black-box fixture with required rows |
| Actual secondary index path use | internal planner/path test or observable trace-free path assertion in `tests/secondary_index.rs` |
| No full-scan fallback before index creation | black-box unsupported-SQL test for `SELECT * FROM users WHERE age = 20;` before `CREATE INDEX` |
| Ordering and tie-break rules | tests for primary-key tie-break and no-primary-key insertion-order tie-break |
| Persisted compatibility | old no-index fixture reopen, backfill, post-index atomic `I` insert, process reopen equality/range, interrupted backfill reopen/retry, post-index insert retry tests |
| `db check` secondary invariant | positive check plus deterministic corrupted/missing index-entry fixture |
| Existing primary/row-only behavior | baseline `scripts/verify` |
| Docs | manual final report mapping for `docs/cli_contract.md`, `docs/file_format.md`, and `docs/sql_subset.md` |

## Risks
- Disk-backed index consistency is a larger surface than in-memory primary indexes; implement the invariant tests before production code.
- Crash during multi-record `CREATE INDEX` uses the selected `build_id`/final-`X` commit policy; tests must cover orphan entries, reopen, `db check`, and retry.
- Post-index `INSERT` must use the single-record `I` policy; `R + standalone E` implementation is out of plan because it cannot preserve no-partial-record behavior.
- Existing exact hints mention only INSERT/SELECT for table-not-found. `CREATE INDEX` needs the contract-specific hint without accidental regression.
- The parser currently treats range predicates as unsupported; `BETWEEN` support must stay narrow.

## Stop Conditions
- Any required behavior needs a change to `spec.md` or `contracts.md`.
- Disk-backed secondary index contents cannot be made checkable without changing the lower-level page/WAL format.
- A second recovery attempt is needed after verifier rejection.
