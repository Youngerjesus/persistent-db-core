# Technical Design: v1-primary-btree-index

## Overview
The primary index is an in-memory deterministic B-tree rebuilt from durable SQL row records each time db exec opens the database. The page file remains row-record only: no index pages, metadata files, background rebuild process, or external service are introduced.

## Components

### PrimaryIndex
Planned module: src/index.rs.

Responsibilities:
- insert(key: i64, row_position: usize) -> duplicate-aware result.
- get(key: i64) -> Option<usize>.
- ordered_positions() -> iterator or Vec<usize> in ascending key order.
- len/is_empty helpers as needed by tests.

Implementation:
- std::collections::BTreeMap<i64, usize>.
- Duplicate insert must not overwrite the existing key.

### SQL Table Model
Planned changes in src/sql.rs:
- Column records retain name and type.
- Table records retain rows Vec<Vec<Value>>.
- Table gains optional primary key metadata and an optional PrimaryIndex.
- Primary key metadata identifies one INT column by position.

Invariant:
- If primary_key_column is Some(i), every loaded row has Value::Int at i and PrimaryIndex contains exactly one mapping per row.
- Duplicate primary key during insert is a semantic error.
- Duplicate primary key discovered while loading persisted rows is an invalid SQL storage record.

### Catalog Encoding
The implementation must preserve compatibility with current catalog records.

Recommended approach:
- Decode the current catalog body as no primary key when the record ends immediately after column metadata.
- For new catalog records, append a small versioned catalog extension after existing column metadata that identifies the primary key column.
- Reject malformed extension bytes as invalid SQL storage record.

The exact byte layout is an implementation detail, but docs/file_format.md and docs/sql_subset.md must describe it once implemented.

### Parser
Statement enum should distinguish:
- CreateTable with column definitions that can carry primary key marker.
- Insert unchanged.
- SelectAll { table } unchanged or represented as Select { table, predicate: None }.
- SelectByPrimaryKey { table, column, key } or Select { predicate: Some(ExactIntEquals) }.

Parser acceptance:
- CREATE TABLE users (id INT PRIMARY KEY, name TEXT);
- SELECT * FROM users WHERE id = 2;

Parser rejection remains scoped:
- SELECT * FROM users WHERE name = 'ada'; is unsupported or semantic according to local convention.
- SELECT * FROM users WHERE id > 2; remains unsupported.
- CREATE TABLE users (id TEXT PRIMARY KEY); must not create a table.
- Multiple primary key declarations must not create a table.

### Executor
Create table:
- Validate duplicate columns as today.
- Validate at most one primary key.
- Validate primary key column type is INT.
- Append catalog record only after validations pass.

Insert:
- Validate table existence, column count, and types as today.
- If table has primary key, extract INT key and call PrimaryIndex duplicate check before append_record.
- Append row record only after duplicate check passes.
- Push row and update index with new row position.

Select all:
- Emit header first.
- If primary key exists, iterate PrimaryIndex ordered positions.
- If no primary key, keep current rows iteration order.

Select exact primary key:
- Validate table exists.
- Validate predicate column is the table's primary key column. Non-primary-key predicates are outside this slice and should not silently full-scan.
- Emit header first.
- Use PrimaryIndex get; emit one row when present, header only when missing.

## Persistence And Rebuild Flow
1. PageStore reads opaque records in append order.
2. SQL decoder builds catalog table entries.
3. Row records append rows to the table's row Vec.
4. When a row belongs to a primary-key table, the rebuild path inserts its key and row position into the table's PrimaryIndex.
5. Any duplicate or invalid row for a primary-key table returns SqlError::InvalidStorageRecord.

## Error Strings
Duplicate primary key semantic error:

```text
duplicate primary key for table users: 2
```

Hint:

```text
primary key values must be unique.
```

main.rs already formats SqlError::Semantic as:

```text
error: SQL semantic error: {message}
hint: {hint}
```

## Documentation Delta
- docs/sql_subset.md: grammar, output order split between PK and non-PK tables, WHERE exact lookup, duplicate key error, logical record/catalog primary key marker, non-goals.
- docs/file_format.md: page framing unchanged, SQL logical catalog may carry primary key metadata, no index metadata persisted, rebuild model, compatibility.
- docs/cli_contract.md: primary-key exec examples and updated SQL execution behavior/non-goals.

## Verification Design
- tests/primary_index.rs validates the primitive and rebuild semantics without depending only on CLI outputs.
- tests/sql_exec.rs validates end-to-end CLI observable contract.
- ./scripts/verify remains baseline evidence and catches formatting, clippy, all tests, and help smoke.

