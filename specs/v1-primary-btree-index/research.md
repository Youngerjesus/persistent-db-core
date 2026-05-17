# Research: v1-primary-btree-index

## Frozen Inputs
- spec.md and contracts.md are adopted without rewrite.
- This phase is planning-only; production code, tests, runtime config, and final evidence artifacts are not edited.

## Current Repo Facts
- SQL execution currently rebuilds an in-memory Database from PageStore records on every db exec invocation.
- Tables currently store ordered columns and rows in insertion order.
- SQL logical records have only catalog and row payloads over the existing V1 PageStore record framing.
- SELECT currently supports only SELECT * FROM table; and WHERE is currently rejected as unsupported.
- CLI maps SqlError::Semantic to exit code 2, InvalidStorageRecord to exit code 1, and successful exec to exit code 0 with stderr empty.
- Existing docs define append-order SELECT behavior and list indexes and WHERE as non-goals; implementation must update those durable docs for this slice.

## Technical Decisions

### D1. Primary index implementation
Use a new src/index.rs module with an in-memory B-tree primitive backed by std::collections::BTreeMap<i64, usize>.

Rationale:
- The contract asks for a primary B-tree index primitive and deterministic ordered traversal proof.
- BTreeMap is in the Rust standard library, fits the repo's no-new-dependency preference, and gives deterministic key ordering.
- The index can map primary key values to row positions in the table's row Vec, preserving existing row storage while providing lookup and ordered scan paths.

Rejected alternatives:
- Persisted index metadata: explicitly out of scope.
- Custom on-disk B-tree pages: beyond this slice and would change file-format risk.
- Vec sort on every SELECT: could produce ordered results but would not provide the requested index primitive or path evidence.

### D2. Schema metadata
Extend SQL catalog in memory and encoded catalog payloads to represent at most one primary key column.

Rationale:
- The contract adds only single INT PRIMARY KEY column declarations.
- Persisted row records remain unchanged. Existing row-only files without primary-key catalog metadata must still load.
- Catalog evolution must be documented. The implementer should choose a backward-compatible catalog-body extension that old catalog records can be decoded as "no primary key" while new catalog records can carry primary key index information.

Implementation constraint:
- The final implementation must not make existing row-only SQL database files unreadable.
- If the current decoder requires exact end-of-record after columns, the implementation must update decode_record in a way that accepts old catalog records and new catalog records deterministically.

### D3. Database rebuild model
Rebuild primary indexes from durable catalog and row records during Database::from_records.

Rationale:
- Process restart/reopen evidence must prove exact lookup and ordered scan are stable with no persisted index metadata.
- Duplicate primary keys in durable records should be treated as invalid SQL storage records during rebuild because the file violates the primary key invariant.

### D4. SQL grammar
Extend parser only for:
- CREATE TABLE table (id INT PRIMARY KEY, ...);
- SELECT * FROM table WHERE id = 2;

Rationale:
- Exact primary key predicate is in scope.
- ORDER BY, range predicates, non-primary-key WHERE, multi-column primary key, non-INT primary key, projection, and optimizer behavior remain out of scope.

Parsing notes:
- Keywords remain ASCII case-insensitive.
- PRIMARY KEY should be accepted only directly after an INT column type.
- Duplicate or multiple primary key declarations should be semantic or malformed according to existing parser conventions; tests should pin the selected behavior if implemented.
- WHERE must be accepted only in the exact shape WHERE <identifier> = <signed int literal> after SELECT * FROM <table>.

### D5. Execution behavior
- INSERT into a primary-key table checks the primary index before appending a row record.
- SELECT * FROM primary-key table without WHERE emits rows by primary key ascending using the primary index.
- SELECT * FROM primary-key table WHERE pk = value emits either one row or only the header.
- SELECT * FROM non-primary-key table preserves existing insertion-order behavior.

Rationale:
- This directly matches observable SQL contract and preserves existing behavior for tables without primary keys.

### D6. Error behavior
Duplicate primary key must return:

```text
error: SQL semantic error: duplicate primary key for table users: 2
hint: primary key values must be unique.
```

Other existing error strings should remain stable unless the approved contract explicitly changes them.

### D7. Documentation
Update docs/file_format.md, docs/sql_subset.md, and docs/cli_contract.md in the implementation phase.

Required points:
- No separate persisted index metadata is stored.
- Existing row-only SQL files remain compatible.
- Indexes are rebuilt from catalog and row records on open/exec.
- Corrupt SQL row records still fail through the existing invalid SQL storage record path.
- Missing index metadata is not a failure mode because no index metadata exists.

## Open Questions
None requiring product or contract decisions. Remaining choices are implementation details bounded by the approved package.

