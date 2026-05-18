# Technical Design: v1-secondary-index-range-scan

## Overview
The implementation extends the existing append-only SQL logical-record layer with disk-backed secondary index metadata and index entry records. Runtime query execution reconstructs table rows, primary indexes, and secondary indexes from durable records, then routes supported indexed predicates through the matching secondary index.

The lower-level V1 page format and WAL sidecar remain unchanged.

## Components

### Secondary Index Primitive
Location: `src/index.rs`

Responsibilities:
- Store entries ordered by `(secondary_key, tie_break)`.
- Return row positions for exact secondary key lookup.
- Return row positions for inclusive key ranges.
- Reject duplicate `(secondary_key, tie_break)` entries as invalid durable index contents.

Planned API shape:

```rust
pub struct SecondaryIndex { /* BTreeMap<(i64, i64), usize> */ }

impl SecondaryIndex {
    pub fn insert(&mut self, key: i64, tie_break: i64, row_position: usize) -> Result<(), DuplicateSecondaryIndexEntry>;
    pub fn equality_positions(&self, key: i64) -> Vec<usize>;
    pub fn range_positions(&self, low: i64, high: i64) -> Vec<usize>;
}
```

### SQL Model
Location: `src/sql.rs`

Add:
- `Statement::CreateIndex { index, table, column }`
- `Statement::SelectSecondaryEquality { table, column, key, raw }`
- `Statement::SelectSecondaryRange { table, column, low, high, raw }`
- `SecondaryIndexState` with build id, index name, column ordinal, tie-break mode, and `SecondaryIndex`.
- `LogicalRecord::SecondaryIndexMetadata`
- `LogicalRecord::SecondaryIndexEntry`
- `LogicalRecord::IndexedRow`

The implementation may keep a single parsed select enum and resolve primary/secondary path during execution, but tests must be able to prove secondary equality/range use.

### Parser
Accepted new forms:

```text
CREATE INDEX <index_name> ON <table_name>(<column_name>);
SELECT * FROM <table_name> WHERE <column_name> = <int_value>;
SELECT * FROM <table_name> WHERE <column_name> BETWEEN <low_int> AND <high_int>;
```

Rules:
- Keywords are ASCII case-insensitive.
- Identifiers follow the existing identifier rule.
- Parentheses around `CREATE INDEX` column allow whitespace around the column name.
- `BETWEEN` is inclusive.
- If `low > high`, return header only through the index path. This is the only allowed behavior for this plan.
- If the table/column exists but no committed secondary index exists for a non-primary-key equality or range predicate, return `SqlError::Unsupported(raw)` so the CLI exits `2`, writes empty stdout, and uses the unsupported SQL stderr shape. This locks `SELECT * FROM users WHERE age = 20;` before `CREATE INDEX` as unsupported and prevents full-scan fallback.

### Execution
`CREATE INDEX`:
1. Find table or return exact missing-table semantic error:
   `hint: create the table before INSERT, SELECT, or CREATE INDEX.`
2. Find column or return exact missing-column semantic error.
3. Require target column type `INT` or return exact unsupported-type semantic error.
4. Reject duplicate index name case-insensitively across the database.
5. Compute `build_id = durable SQL logical record count before the command appends any records`.
6. Compute backfill entries for all existing rows.
7. Append durable `E` records with that `build_id`.
8. Append the final durable `X` metadata record with that `build_id`; this is the commit marker.
9. Register runtime index state only after the `X` record append succeeds.

`INSERT`:
1. Preserve existing validation and primary-key duplicate check.
2. If the table has no committed secondary indexes, append the existing `R` row record.
3. If the table has committed secondary indexes, compute the new durable row position and all required secondary index entries, then append exactly one `I` indexed-row record containing the row values and the full embedded entry set.
4. Do not append `R` plus standalone `E` records for post-index inserts.
5. Update in-memory rows and indexes only after the single durable append succeeds.
6. If encoding or append fails, return the storage error and leave runtime state unchanged; the existing no-partial-SQL-record statement contract is preserved because post-index insert uses one logical record.

`SELECT`:
- Primary-key exact lookup remains routed through `PrimaryIndex`.
- Secondary equality resolves an index by table and column and calls `SecondaryIndex::equality_positions`.
- Secondary range resolves an index by table and column and calls `SecondaryIndex::range_positions`.
- Non-indexed predicates must not silently full-scan for this task's equality/range acceptance.

### Reopen And Validation
Reconstruction from records:
1. Decode catalog, row `R`, and indexed-row `I` records in durable order.
2. Decode secondary index entry records into a pending entry map by `(case-insensitive index_name, build_id)`.
3. Decode committed secondary index metadata `X` records and create runtime secondary index states keyed by `(case-insensitive index_name, build_id)`.
4. Attach only entries whose `(index_name, build_id)` matches a committed `X`.
5. Ignore orphan `E` records with no matching committed `X`; they are uncommitted interrupted builds.
6. Apply each `I` as one row plus its embedded entries atomically.
7. Validate committed metadata-backed indexes against rows.

Validation invariants:
- Index metadata references an existing table and `INT` column.
- Index name is globally unique case-insensitively.
- Every metadata-backed index has exactly one entry for every durable row in the owning table.
- Every entry points to an existing row position.
- Entry key equals the indexed row value.
- Entry tie-break equals primary-key value or durable row position according to metadata.
- Duplicate `(key, tie_break)` entries fail.
- Rows for indexed tables missing index entries fail.
- Orphan entries without committed metadata do not fail `db check`.
- Retried same-name builds after interruption commit with a new `build_id`; old orphan entries must not attach to the retried index.
- `I` records must contain exactly one embedded entry for every committed secondary index on the row's table, no entries for unrelated indexes, and row positions matching the row contributed by that `I`.
- Missing, extra, duplicate, wrong-key, wrong-tie-break, wrong-build-id, or wrong-row-position embedded entries in an `I` record fail as `secondary index`.

`db check` must report deterministic label `secondary index` for committed secondary index invariant failures.

### Byte Encoding
All new secondary-index logical records start with `PDBSQL1\0` followed by a one-byte record kind.

Committed metadata record:

```text
kind: X
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes, preserved spelling
u16 table_name_len little-endian
table_name UTF-8 bytes, preserved spelling
u16 indexed_column little-endian
u8 tie_break_mode: P for primary-key value, R for row insertion order
```

Index entry record:

```text
kind: E
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes, preserved spelling
i64 indexed_key little-endian
i64 tie_break little-endian
u64 row_position little-endian
```

Indexed row record:

```text
kind: I
u16 table_name_len little-endian
table_name UTF-8 bytes, preserved spelling
u16 value_count little-endian
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count little-endian
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes, preserved spelling
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

Names are stored with preserved spelling. All comparisons for index-name uniqueness and entry-to-metadata attachment are ASCII case-insensitive. Fixture helpers, `docs/file_format.md`, and `docs/sql_subset.md` must use this exact layout.

## Error Contract
The exact CREATE INDEX semantic errors are contract-fixed:

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT, SELECT, or CREATE INDEX.
```

```text
error: SQL semantic error: column not found for index idx_users_age: age
hint: create the index on an existing table column.
```

```text
error: SQL semantic error: secondary index column must be INT: name
hint: this SQL slice supports secondary indexes only on INT columns.
```

```text
error: SQL semantic error: index already exists: idx_users_age
hint: use a new index name for CREATE INDEX in this database.
```

## Deterministic Ordering
- Primary-key table, duplicate secondary key: tie-break by primary-key value ascending.
- No-primary-key table, duplicate secondary key: tie-break by durable row insertion order.
- Overall order: secondary key ascending, then tie-break ascending.

## Documentation Updates
- `docs/cli_contract.md`: command behavior, grammar summary, stdout/stderr/exit code, exact errors, equality/range examples, ordering.
- `docs/file_format.md`: `X` and `E` SQL logical records, old no-index compatibility, backfill, check invariants.
- `docs/sql_subset.md`: grammar and logical record details because it owns the SQL subset reference.

## Verification Evidence
- `scripts/verify`
- `cargo test --test secondary_index -- --nocapture`
- Final report must map `REQ-7-create-index-must-create-disk-3b71a7dc` to CLI examples, index path use evidence, persisted compatibility evidence, and `db check` evidence.
