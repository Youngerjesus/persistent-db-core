# Technical Design: v1-secondary-index-mutation-consistency

## Overview
The design extends the current append-only SQL logical-record layer with single-record `UPDATE` and `DELETE` mutation events. Reopen reconstructs the current visible table state and committed primary/secondary indexes by replaying catalog, row/index creation, indexed inserts, and mutation records in durable order.

The lower-level page file and WAL sidecar format remain unchanged.

## Components

### Parser And Statement Model
Location: `src/sql.rs`

Add narrow statement variants:

```rust
Statement::Update {
    table: String,
    set_column: String,
    value: Value,
    where_column: String,
    where_key: i64,
    raw: String,
}

Statement::Delete {
    table: String,
    where_column: String,
    where_key: i64,
    raw: String,
}
```

Accepted forms:

```text
UPDATE <table_name> SET <column_name> = <int_or_text_value> WHERE <primary_key_column> = <int_value>;
DELETE FROM <table_name> WHERE <primary_key_column> = <int_value>;
```

Rules:
- Keywords are ASCII case-insensitive.
- Identifiers use the existing identifier rule.
- The `WHERE` column must be the table's `INT PRIMARY KEY`.
- Unsupported mutation shapes return the existing unsupported/malformed SQL surface rather than silently broadening scope.

### Runtime Table State
Location: `src/sql.rs`

Recommended internal shape:

```rust
struct RowState {
    values: Vec<Value>,
    visible: bool,
}
```

`Table.rows` may become `Vec<RowState>` or equivalent. Row positions remain stable durable positions. All user-visible query paths operate only on visible rows.

### Index Maintenance
Location: `src/sql.rs`, optionally `src/index.rs`

Needed secondary-index operations:
- insert `(key, tie_break, row_position)`;
- remove exact `(key, tie_break)` entry;
- equality/range iteration remains deterministic.

For update:
- compute old entries from the old visible row;
- compute new entries from the replacement row at the same row position;
- remove old entries from committed secondary indexes;
- insert new entries.

For delete:
- compute old entries from the target visible row;
- remove old entries;
- mark row invisible;
- remove primary-key mapping.

Primary index maintenance should mirror visibility:
- update primary-key mapping if a future supported assignment changes the primary key;
- delete removes the key from the primary index.

### Durable Mutation Records
Location: `src/sql.rs`, docs in `docs/sql_subset.md` and `docs/file_format.md`

Add SQL logical record kinds:

```text
U  update one visible primary-key row and secondary-index entries atomically
D  delete one visible primary-key row and secondary-index entries atomically
```

Recommended `U` content:
- table name;
- primary-key column ordinal;
- primary-key value identifying the row before mutation;
- set column ordinal;
- complete replacement row values;
- removed embedded secondary entries;
- added embedded secondary entries.

Recommended `D` content:
- table name;
- primary-key column ordinal;
- primary-key value identifying the row before deletion;
- removed embedded secondary entries.

All variable names are UTF-8 bytes with little-endian `u16` lengths. All integer fields use little-endian fixed-width bytes, matching existing `X`, `E`, and `I` conventions.

### Reopen Replay
Replay order:
1. `C`: create table metadata.
2. `R`: append visible row only when no secondary indexes are committed for the table, preserving existing invariant.
3. `E`: collect pending backfill entries by `(index_name, build_id)`.
4. `X`: commit secondary index metadata and attach matching backfill entries.
5. `I`: append visible row plus complete embedded entries for every committed index.
6. `U`: locate visible row by primary key, validate old/current identity, validate encoded removed/added entries, replace row values at the same row position, update primary/secondary indexes.
7. `D`: locate visible row by primary key, validate encoded removed entries, mark row invisible, remove primary/secondary entries.

Malformed mutation records, mutation of a missing/deleted row, wrong old entry data, wrong new entry data, wrong row position, or wrong secondary index identity should fail validation. Use `secondary index` for committed secondary-index mismatch failures and `catalog/record invariant` or `primary index` only when the corruption is clearly outside secondary-index maintenance.

### `db check` Invariants
`db check` must verify:
- every visible indexed row has exactly one committed secondary entry per committed index;
- no committed secondary entry points to a deleted or nonexistent row position;
- no stale old-key entry remains after an update;
- update replacement entries match the visible row's indexed values and tie-breaks;
- delete removal entries match the row being deleted;
- primary-key lookup is rebuildable for visible rows;
- table scans exclude deleted rows.

Required negative fixture labels:
- stale old-key entry after `id=2` changes `age` from `20` to `30`: `secondary index`;
- dangling pointer to a deleted/nonexistent row: `secondary index`;
- missing entry for visible `id=4 age=30`: `secondary index`.

### WAL Evidence
Because mutations use one SQL logical record per statement, WAL replay evidence can reuse the existing page-append sidecar behavior. Tests should retain the `<path>.wal` sidecar and run reopen query plus `db check` in separate processes while both page file and WAL sidecar exist.

### Documentation
Update after implementation bytes are final:
- `docs/cli_contract.md`: supported mutation forms, exact success behavior, post-mutation example outputs, `db check` result.
- `docs/sql_subset.md`: grammar, unsupported breadth, logical `U`/`D` records.
- `docs/file_format.md`: SQL logical record compatibility; lower-level page/WAL unchanged.

