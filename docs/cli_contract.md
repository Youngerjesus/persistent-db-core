# V1 `db` CLI Contract

This slice defines the deterministic command-line contract for the `db` binary,
including the minimal SQL execution path, primary-key lookup path, and database
check path.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
db exec <path> <sql>
db check <path>
```

`db --help` and `db help` exit with code `0`, write no stderr, and write
identical help text to stdout.

`db exec <path> <sql>` executes one SQL argument against the database file at
`<path>`. The file is created if it does not exist. SQL from stdin, interactive
shell input, and multiple SQL argv fragments are not supported.

`db check <path>` validates an existing database file and WAL sidecar without
repairing or mutating them. The file must already exist and must be a regular
file.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
Reserved future commands:
  open <path>
  bench <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: help printed successfully, `db exec` completed successfully, or
  `db check` passed.
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

## SQL Execution

Successful `db exec` writes no stderr. It writes stdout only for supported
`SELECT *` statements. Each result set prints the stored column header followed
by rows, with `|` as the field delimiter and `\n` after every output line.
Tables without a primary key scan in successful `INSERT` append order. Tables
declared with one `INT PRIMARY KEY` scan in ascending primary-key order.
`SELECT * FROM <table> WHERE <primary_key> = <int>;` performs exact primary-key
lookup and prints only the matching row, or only the header when the key is
missing. Multiple `SELECT` statements repeat the header with no blank line,
separator, or count line.

Successful `CREATE TABLE` and `INSERT` mutations are durable across later
`db exec` process starts for the same database path. WAL sidecar details are
documented in `docs/file_format.md`; they do not change successful `db exec`
stdout, stderr, or exit codes.

The supported SQL subset is documented in `docs/sql_subset.md`.

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ..., SELECT * FROM ... WHERE <primary_key> = <int>;
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
`catalog/record invariant`, `primary index`, and `wal replay consistency`.

Missing paths, directories, and paths that cannot be opened or read exit `1`,
write empty stdout, and use this stderr shape with path context:

```text
error: could not open or read database path: <path>
```

## Reserved Future Commands

The following names are reserved for later V1 work but are not executable in this slice:

```text
open <path>
bench <path>
```

Invoking any reserved command currently follows the unsupported input behavior.

## Non-Goals

This slice does not implement projection, general `WHERE`, `ORDER BY`, `JOIN`,
`UPDATE`, `DELETE`, public transaction commands, secondary indexes, networking,
multi-process concurrency, or distributed storage. Primary indexes are rebuilt
from durable SQL row records on open; there is no separate persisted index
metadata, so existing row-only SQL files remain compatible and missing index
metadata is not a failure mode. Corrupt SQL row records, including duplicate
persisted primary-key values, fail with the invalid SQL storage record error.
