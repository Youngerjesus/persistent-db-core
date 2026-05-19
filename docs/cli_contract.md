# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path, primary-key lookup path, secondary
index lookup/range path, and database check path.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
db exec <path> <sql>
db check <path>
db bench
```

`db --help` and `db help` exit with code `0`, write no stderr, and write
identical help text to stdout.

`db exec <path> <sql>` executes one SQL argument against the database file at
`<path>`. The file is created if it does not exist. SQL from stdin, interactive
shell input, and multiple SQL argv fragments are not supported.

`db check <path>` validates an existing database file and WAL sidecar without
repairing or mutating them. The file must already exist and must be a regular
file.

`db bench` runs the fixed Section 14 benchmark acceptance workload and writes
machine-readable evidence to
`target/bench_acceptance/section14-benchmark-acceptance.json`.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
  db bench
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
  bench       Run the fixed Section 14 benchmark acceptance workload.
Reserved future commands:
  open <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: help printed successfully, `db exec` completed successfully, or
  `db check` passed. `db bench` also exits `0` after generating passing
  Section 14 evidence.
- `1`: storage, SQL logical-record data, or `db check` invariants are invalid
  for this contract. `db check` open/read failures also use exit code `1`.
- `2`: the first argument was unsupported, or no supported command was provided.
  SQL syntax, unsupported SQL, and SQL semantic errors also use exit code `2`.

## Unsupported Input

Unsupported arguments and subcommands exit with code `2`, write no stdout, and write this stderr format:

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

`<token>` is the first unsupported token supplied by the user. For example, `db --unknown` reports `--unknown`, `db open demo.db` reports `open`, and `db exec demo.db` reports `exec`.

`db bench <extra>` is unsupported and reports `bench`.

## SQL Execution

Successful `db exec` writes no stderr. It writes stdout only for supported
`SELECT *` statements. Each result set prints the stored column header followed
by rows, with `|` as the field delimiter and `\n` after every output line.
Tables without a primary key scan in successful `INSERT` append order. Tables
declared with one `INT PRIMARY KEY` or `INTEGER PRIMARY KEY` scan in ascending
primary-key order. `INTEGER` is an accepted spelling alias for the existing
integer column type; it does not add SQL affinity behavior or any other type
alias.
`SELECT * FROM <table> WHERE <primary_key> = <int>;` performs exact primary-key
lookup and prints only the matching row, or only the header when the key is
missing. Multiple `SELECT` statements repeat the header with no blank line,
separator, or count line.

`CREATE INDEX <index> ON <table>(<integer_column>);` creates a durable
secondary index over an existing `INT` column. Successful `CREATE TABLE`,
`INSERT`, `CREATE INDEX`, primary-key-targeted `UPDATE`, and
primary-key-targeted `DELETE` mutations exit `0`, write empty stdout/stderr
unless a later `SELECT` writes rows, and are durable across later `db exec`
process starts for the same database path. WAL sidecar details are documented
in `docs/file_format.md`; they do not change successful `db exec` stdout,
stderr, or exit codes.

`UPDATE <table> SET <non_primary_key_column> = <value> WHERE
<primary_key_column> = <int>;` updates one matching row. `DELETE FROM <table>
WHERE <primary_key_column> = <int>;` deletes one matching row. Missing
primary-key targets are successful no-ops. After either mutation, table scans,
primary-key lookups, and secondary-index equality/range scans must agree across
later process starts.

After `CREATE INDEX`, `SELECT * FROM <table> WHERE <indexed_column> = <int>;`
uses the matching secondary index. `SELECT * FROM <table> WHERE
<indexed_column> BETWEEN <low_int> AND <high_int>;` uses an inclusive bounded
secondary-index range scan. Results are ordered by secondary key ascending and
then by deterministic tie-break ascending. For tables with a primary key, the
tie-break is the primary-key value. For tables without a primary key, the
tie-break is durable row insertion order. If `low_int > high_int`, the range
query still uses the secondary range path and prints the header only.

Before a secondary index exists, non-primary-key equality and `BETWEEN`
predicates remain unsupported SQL and must not silently full-scan.

The supported SQL subset is documented in `docs/sql_subset.md`.

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset is documented in docs/sql_subset.md.
```

Malformed SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: malformed SQL statement: CREATE TABLE users id INT);
hint: terminate each statement with ';' and use the documented SQL subset.
```

SQL semantic errors exit `2`, write empty stdout, and use the exact strings below:

```text
error: SQL semantic error: table already exists: users
hint: use a new table name for CREATE TABLE in this database.
```

Case-variant duplicate table input reports the new input spelling, such as
`Users`.

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT or SELECT.
```

`CREATE INDEX` against a missing table uses this more specific hint:

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT, SELECT, or CREATE INDEX.
```

```text
error: SQL semantic error: duplicate column: id
hint: column names in a table must be unique.
```

Case-variant duplicate column input reports the new input spelling, such as
`ID`.

```text
error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1
hint: INSERT values must match the table schema exactly.
```

```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

```text
error: SQL semantic error: duplicate primary key for table users: 2
hint: primary key values must be unique.
```

```text
error: SQL semantic error: primary key column must be INT: id
hint: this SQL slice supports one INT PRIMARY KEY column per table.
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

Required secondary-index examples:

```text
CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);
INSERT INTO users VALUES (3, 20, 'cal');
INSERT INTO users VALUES (1, 10, 'ada');
INSERT INTO users VALUES (2, 20, 'bea');
CREATE INDEX idx_users_age ON users(age);
SELECT * FROM users WHERE age = 20;
```

stdout:

```text
id|age|name
2|20|bea
3|20|cal
```

With the same fixture, `SELECT * FROM users WHERE age BETWEEN 10 AND 20;`
writes:

```text
id|age|name
1|10|ada
2|20|bea
3|20|cal
```

Invalid SQL logical records exit `1`, write empty stdout, and use this stderr:

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## Database Check

Successful `db check <path>` exits `0`, writes no stderr, and writes exactly:

```text
ok: db check passed
```

The trailing newline is part of the contract.

`db check` performs a read-only validation of page-record readability, SQL
catalog/row consistency, primary-key rebuildability, and the documented WAL
sidecar ordering rule. It does not repair files, create missing files, replay
WAL frames into the page file, or checkpoint retained complete WAL frames.

Invariant failures exit `1`, write empty stdout, and use this stderr prefix:

```text
error: db check failed: <invariant label>
```

Documented invariant labels include `storage record readability`,
`catalog/record invariant`, `primary index`, `secondary index`, and
`wal replay consistency`.

Missing paths, directories, and paths that cannot be opened or read exit `1`,
write empty stdout, and use this stderr shape with path context:

```text
error: could not open or read database path: <path>
```

## Reserved Future Commands

The following names are reserved for later V1 work but are not executable in this slice:

```text
open <path>
```

Invoking any reserved command currently follows the unsupported input behavior.

## Benchmark Acceptance

Successful `db bench` exits `0`, writes no stderr, creates
`target/bench_acceptance/section14-benchmark-acceptance.json`, and writes
exactly:

```text
DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json
```

The trailing newline is part of the contract. Failed benchmark evidence exits
non-zero and writes this stdout shape:

```text
DB_BENCH: FAIL check=<check_id> reason=<reason>
```

The companion verifier `scripts/verify_bench_acceptance` must invoke public
`db bench`, validate the same evidence file, finalize
`commands.verify_bench_acceptance.status="pass"` and `result="pass"`, then
write:

```text
BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json
```

Verifier failures use:

```text
BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>
```

The evidence covers Section 14 requirement IDs `METRIC-14-1`, `METRIC-14-2`,
`METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, and `EVID-16-7`.
Threshold formulas are:

```text
equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms
range_index_speedup = range_scan_median_ms / range_indexed_median_ms
```

Hard-fail policy rejects missing required fields, non-positive required numeric
metrics, `equality_index_speedup < 5.0`, `range_index_speedup < 3.0`, eligible
indexed equality/range measurements that observe full scans, retry-required
evidence, and recovery evidence violating
`recovery_ms <= max(2000, wal_file_bytes / 4096)`.

## Non-Goals

This slice does not implement projection, general `WHERE` beyond primary-key
equality and indexed `INT` equality/range predicates, `ORDER BY`, `JOIN`,
non-primary-key-targeted mutations, primary-key updates, public transaction
commands, networking, multi-process concurrency, or distributed storage. Primary
indexes are rebuilt from durable SQL row records on open. Secondary indexes are
persisted with SQL logical records documented in `docs/file_format.md`;
existing row-only SQL files remain compatible. Corrupt SQL row records,
including duplicate persisted primary-key values, fail with the invalid SQL
storage record error.
