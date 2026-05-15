# Autopilot V1 Spec: Persistent DB Core

## 1. Summary

V1 is an implementation test for a small SQLite-like database core.

The system must implement a persistent, page-based, disk-backed database with ordered indexes, single-process transactions, WAL-based recovery, deterministic crash simulation, invariant checking, differential/property-based tests, and basic performance constraints.

V1 is not an in-memory toy database. It is a test of whether Autopilot can design, implement, debug, and verify a complex persistent stateful system end to end.

## 2. Capability Being Tested

Passing V1 demonstrates:

- complex persistent stateful system implementation
- disk-backed data structure implementation
- transaction atomicity
- crash and recovery reasoning
- table/index consistency maintenance
- invariant-driven validation
- differential and property-based testing
- performance awareness sufficient to reject toy implementations

## 3. Required SQL Subset

### 3.1 Required Statements

The implementation must support the following SQL forms:

```sql
CREATE TABLE table_name (
  id INTEGER PRIMARY KEY,
  col1 INTEGER,
  col2 TEXT
);

CREATE INDEX index_name ON table_name (col1);

INSERT INTO table_name (id, col1, col2) VALUES (1, 10, 'hello');

SELECT * FROM table_name WHERE id = 1;

SELECT col1, col2 FROM table_name WHERE col1 = 10;

UPDATE table_name SET col1 = 20 WHERE id = 1;

DELETE FROM table_name WHERE id = 1;

BEGIN;
COMMIT;
ROLLBACK;
```

The parser may support only this subset, but unsupported SQL must fail with a clear error rather than crashing.

### 3.2 Required Predicates

The following `WHERE` predicates must be supported for primary key and indexed integer columns:

```sql
WHERE column = value
WHERE column < value
WHERE column <= value
WHERE column > value
WHERE column >= value
WHERE column BETWEEN a AND b
```

### 3.3 Explicitly Excluded SQL Features

The following are out of scope for V1:

- `JOIN`
- `GROUP BY`
- `HAVING`
- aggregation
- subqueries
- foreign keys
- `NULL`
- floating-point types
- concurrent transactions

## 4. Data Types

The implementation must support:

- `INTEGER`: signed 64-bit integer
- `TEXT`: UTF-8 string, maximum 1024 bytes

`NULL` is not supported.

## 5. Error Behavior

The system must return clear errors for:

- duplicate primary key insert
- access to a missing table
- access to a missing column
- unsupported SQL
- syntax errors
- type errors
- malformed database or WAL metadata found during `db check` or `db recover`

Syntax errors and unsupported SQL must not crash the process.

## 6. Storage Requirements

The database must use disk-backed persistent storage.

Required properties:

- Data must be stored in a disk file.
- Storage must be page-based.
- Default page size must be 4096 bytes.
- Data must survive process restart.
- The implementation must not keep all data only in memory and dump it at the end.
- The implementation must not rewrite the entire database file on every operation.
- Reads and writes must operate through page-level storage abstractions.

The following internal details are intentionally left to Autopilot:

- page header format
- record layout
- free page management
- overflow page design
- buffer/cache design
- page allocation policy
- compaction strategy

## 7. Index Requirements

The database must use disk-backed ordered indexes.

Required properties:

- `INTEGER PRIMARY KEY` must be implemented as a disk-backed ordered index.
- The ordered index must be a B+Tree or an equivalent ordered page-based structure.
- `CREATE INDEX` must create a secondary index.
- Equality lookup and range scan must use indexes where applicable.
- Insert, update, and delete must keep table rows and indexes consistent.
- Indexes must survive process restart.

Required invariants:

- primary key uniqueness
- B+Tree ordering
- correct leaf scan order
- secondary index entry matches the referenced table row
- no dangling index pointer
- no visible row missing from the required index

## 8. Transaction Requirements

V1 requires single-process, single-writer transactions only.

Required semantics:

- `BEGIN` starts a transaction.
- `COMMIT` atomically persists all writes in the transaction.
- `ROLLBACK` discards all writes in the transaction.
- If a crash occurs during a transaction before a successful `COMMIT`, uncommitted writes must not be visible after recovery.
- If `COMMIT` returns successfully, the transaction must survive crash and recovery.

Out of scope:

- concurrent transactions
- MVCC
- snapshot isolation
- serializable isolation
- lock manager

## 9. WAL and Recovery Requirements

The implementation must provide a WAL or an equivalent write-ahead recovery mechanism.

Required properties:

- Committed transactions must be applied exactly once after recovery.
- Uncommitted transactions must not be applied after recovery.
- Recovery must be idempotent.
- A checkpoint or log truncation mechanism must be provided.
- Crash during checkpoint must not corrupt the database.

Recovery invariants:

1. A committed transaction survives crash.
2. An uncommitted transaction disappears after recovery.
3. Recovery can run multiple times safely.
4. Table/index consistency remains valid after recovery.
5. Crash during checkpoint does not corrupt the database.

## 10. Required CLI/API Contract

The implementation must provide a CLI with the following commands:

```bash
db init <path>
db exec <path> "CREATE TABLE users (id INTEGER PRIMARY KEY, age INTEGER, name TEXT);"
db exec <path> "INSERT INTO users (id, age, name) VALUES (1, 20, 'kim');"
db query <path> "SELECT * FROM users WHERE id = 1;"
db recover <path>
db check <path>
db bench <path>
```

`db check` is mandatory.

`db check` must validate at least:

- page reachability
- free page consistency
- B+Tree ordering
- primary key uniqueness
- secondary index consistency
- WAL metadata consistency
- absence of table/index dangling references

## 11. Crash Simulation

Deterministic crash injection is mandatory.

Required behavior:

- Workloads must be executable in a child process.
- The implementation must support forced process termination after the Nth low-level file write.
- The crash point must be deterministic and reproducible.

Example:

```bash
DB_CRASH_AFTER_WRITE=7 db exec test.db "BEGIN; INSERT INTO users VALUES (1, 20, 'kim'); COMMIT;"
```

Required crash test flow:

1. Run workload in a child process.
2. Force crash after the Nth write.
3. Reopen the database.
4. Run `db recover`.
5. Run `db check`.
6. Compare final state with the expected committed state.

## 12. Differential Testing

SQLite must be used as a differential oracle for the supported SQL subset.

The same operation sequence must be run against both the implementation and SQLite. `SELECT` results must match for all supported semantics.

Required operation coverage:

- `INSERT`
- `UPDATE`
- `DELETE`
- `SELECT`
- `CREATE INDEX`
- workloads without explicit transactions
- workloads with committed transactions

The implementation is compared only on the supported subset, not on all SQLite behavior.

## 13. Property-Based Testing

The test suite must include property-based tests for at least:

1. After insert, selecting the same primary key returns the inserted row.
2. After delete, selecting the row returns no result.
3. After update, selecting the row returns the new value.
4. Duplicate primary key insert fails.
5. Index scan result equals full table scan result.
6. After commit and restart, data remains visible.
7. After rollback, changes disappear.
8. After crash/recovery, only committed state remains.
9. Running recovery multiple times produces the same result.

## 14. Performance Constraints

The goal is not to beat SQLite. The goal is to reject toy implementations.

Performance constraints are part of the V1 correctness gate. A functionally correct implementation can still fail V1 if it uses an obviously non-scalable strategy such as full scans for indexed lookups, whole-file rewrites for small updates, or full logical database rebuild during recovery.

The benchmark must be repeatable and must report enough metrics to explain whether indexes, page storage, and WAL recovery are actually being used.

### 14.1 Benchmark Dataset

The benchmark dataset must contain:

- 100,000 rows
- integer primary key
- secondary indexed integer column
- text column length between 8 and 64 bytes

### 14.2 Required Benchmarks

The benchmark suite must include:

1. 100k sequential inserts
2. 10k random primary-key lookups
3. 10k secondary-index equality lookups
4. 1k indexed range scans
5. full scan vs indexed scan comparison
6. recovery after 10k committed transactions

### 14.3 Required Metrics

The performance report must include:

- total elapsed time per benchmark
- rows or operations per second
- database file size after benchmark
- WAL file size before and after checkpoint
- recovery time for the 10k committed transaction workload
- indexed equality lookup latency compared with full scan latency
- indexed range scan latency compared with full scan latency
- evidence that the indexed query path actually uses the index

The implementation may expose additional internal metrics, but these are mandatory.

### 14.4 Performance Lower Bounds

The implementation must satisfy:

- indexed equality lookup is at least 5x faster than full scan on selective workloads
- indexed range scan is at least 3x faster than full scan on selective workloads
- recovery time is proportional to WAL size and does not perform a full logical database rebuild
- no abnormal O(n^2) behavior at 100k rows
- a single-row update or delete must not rewrite the full database file
- checkpoint must not require rebuilding all logical rows from SQL-level state

### 14.5 Performance Hard Fail Conditions

Any of the following is a V1 performance failure:

- indexed lookup internally performs a full table scan
- secondary index exists on disk but is not used for eligible equality/range predicates
- every write rewrites the full database file
- recovery replays by reconstructing the entire logical database from scratch when page/WAL metadata would allow bounded recovery
- benchmark only measures tiny datasets that hide O(n^2) behavior
- no reproducible benchmark command or performance report is provided

## 15. Required Documentation

The repository must contain:

- `docs/architecture.md`
- `docs/storage_format.md`
- `docs/index_design.md`
- `docs/transaction_recovery.md`
- `docs/testing_strategy.md`
- `docs/performance_report.md`
- `docs/bug_diary.md`

`docs/bug_diary.md` must include, for each discovered bug:

- bug description
- root cause
- fix
- regression test added

## 16. Acceptance Test Matrix

V1 acceptance must be evaluated through black-box tests against the required CLI. The implementation internals must not be trusted as evidence unless they are exposed through reproducible tests, `db check`, or documented benchmark output.

### 16.1 Smoke and SQL Subset Tests

Required scenario:

```bash
db init test.db
db exec test.db "CREATE TABLE users (id INTEGER PRIMARY KEY, age INTEGER, name TEXT);"
db exec test.db "CREATE INDEX idx_users_age ON users (age);"
db exec test.db "INSERT INTO users (id, age, name) VALUES (1, 20, 'kim');"
db query test.db "SELECT * FROM users WHERE id = 1;"
db query test.db "SELECT id, name FROM users WHERE age = 20;"
db check test.db
```

Pass condition:

- primary-key lookup returns the inserted row
- secondary-index lookup returns the inserted row
- `db check` passes

### 16.2 Persistence and Restart Tests

Required scenario:

1. Create a database.
2. Insert rows.
3. Commit the transaction.
4. Fully terminate the process.
5. Reopen the database.
6. Query the same rows.
7. Run `db check`.

Pass condition:

- committed rows remain visible after restart
- indexes remain usable after restart
- `db check` passes after restart

### 16.3 Transaction Atomicity Tests

Required scenarios:

- insert/update/delete inside `BEGIN` followed by `COMMIT`
- insert/update/delete inside `BEGIN` followed by `ROLLBACK`
- duplicate primary key insert inside a transaction
- syntax error inside a transaction

Pass condition:

- committed changes are visible
- rolled-back changes are invisible
- failed statements do not corrupt transaction state
- `db check` passes after each scenario

### 16.4 Crash/Recovery Matrix

For each critical workload, run the workload repeatedly with `DB_CRASH_AFTER_WRITE=N` for increasing `N`.

Required workloads:

```bash
db exec test.db "BEGIN; INSERT INTO users (id, age, name) VALUES (1, 20, 'kim'); COMMIT;"
db exec test.db "BEGIN; UPDATE users SET age = 30 WHERE id = 1; COMMIT;"
db exec test.db "BEGIN; DELETE FROM users WHERE id = 1; COMMIT;"
db exec test.db "BEGIN; INSERT INTO users (id, age, name) VALUES (2, 40, 'lee'); ROLLBACK;"
```

Required flow for each crash point:

1. Run workload in a child process.
2. Crash after the Nth low-level file write.
3. Reopen database.
4. Run `db recover`.
5. Run `db recover` again.
6. Run `db check`.
7. Compare final state with the expected committed state.

Pass condition:

- no uncommitted write is visible
- every successfully committed transaction is visible
- recovery is idempotent
- table and indexes remain consistent
- crash during checkpoint does not corrupt the database

### 16.5 Differential Test Matrix

Required scenario:

1. Generate random supported SQL operation sequences.
2. Execute the same sequence against the implementation and SQLite.
3. Compare every `SELECT` result.

Required coverage:

- insert/update/delete/select
- primary-key predicates
- secondary-index predicates
- range predicates
- committed transactions
- rollback transactions, compared through expected model state when SQLite behavior differs from the supported subset

Pass condition:

- every supported `SELECT` result matches the oracle
- errors occur at equivalent logical points for supported constraints

### 16.6 Property Test Matrix

The property-based test suite must execute at least the properties listed in Section 13.

Pass condition:

- generated tests are reproducible by seed
- every failing seed is recorded
- every fixed failure gets a regression test in `docs/bug_diary.md`

### 16.7 Performance Acceptance Gate

Required scenario:

```bash
db bench test.db
```

Pass condition:

- benchmark uses the dataset from Section 14.1
- required metrics from Section 14.3 are reported
- lower bounds from Section 14.4 pass
- none of the performance hard fail conditions in Section 14.5 occur

## 17. Definition of Done

V1 is complete only when all of the following are true:

1. SQL subset functional tests pass.
2. Page-based persistent storage is implemented.
3. Primary and secondary disk-backed B+Tree indexes are implemented.
4. `BEGIN`/`COMMIT`/`ROLLBACK` semantics pass.
5. WAL recovery passes.
6. Deterministic crash/recovery matrix passes.
7. SQLite differential tests pass.
8. Property-based tests pass.
9. `db check` invariant validation passes.
10. Performance lower bounds pass.
11. Architecture, recovery, testing, and performance documentation is complete.
12. Every discovered bug has a regression test.

## 18. Hard Fail Conditions

Any of the following is an automatic V1 failure:

- using an existing database engine as the implementation
- keeping all data in memory only
- rewriting the whole database file for every operation
- no crash recovery
- table/index inconsistency
- data loss after a successful `COMMIT`
- rolled-back data remains visible
- tests are hardcoded to fixed examples instead of validating general behavior

## 19. One-Line Claim

Passing V1 proves that Autopilot can implement and verify a complex persistent stateful system with page storage, B+Tree indexes, transactions, WAL recovery, crash simulation, differential/property testing, and performance constraints.
