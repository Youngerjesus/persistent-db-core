# V1 SQL Subset

`db exec <path> <sql>` accepts one SQL argument. Statement delimiter is `;`,
and every statement must end with `;`. Multiple statements in one command run in
order.

## Supported Grammar

```text
CREATE TABLE <table_name> (<column_name> INT|INTEGER|TEXT[, <column_name> INT|INTEGER|TEXT]*);
CREATE TABLE <table_name> (<column_name> INT|INTEGER PRIMARY KEY[, <column_name> INT|INTEGER|TEXT]*);
CREATE INDEX <index_name> ON <table_name>(<integer_column>);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
UPDATE <table_name> SET <non_primary_key_column> = <value> WHERE <primary_key_column> = <int_value>;
DELETE FROM <table_name> WHERE <primary_key_column> = <int_value>;
SELECT * FROM <table_name>;
SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;
SELECT * FROM <table_name> WHERE <indexed_int_column> = <int_value>;
SELECT * FROM <table_name> WHERE <indexed_int_column> BETWEEN <low_int> AND <high_int>;
```

Keywords compare ASCII case-insensitively. Identifiers must match
`[A-Za-z_][A-Za-z0-9_]*`. Table and column equality is ASCII
case-insensitive, while stored catalog spelling is preserved for headers and
errors. Types are `INT`, `INTEGER`, and `TEXT`. `INTEGER` is a spelling alias
for the existing `INT` type, not a separate affinity system. `INT` values are
signed 64-bit decimal integers. `TEXT` values are UTF-8 strings inside single quotes; escape
sequences, embedded single quotes, `|`, newline, and carriage return are not
supported.

This slice supports at most one `INT PRIMARY KEY` or `INTEGER PRIMARY KEY`
column per table. Secondary indexes are explicit and support only integer
columns. `TEXT PRIMARY KEY`,
multiple primary-key columns, non-indexed non-primary-key predicates, range
predicates before `CREATE INDEX`, and non-integer predicate values are rejected.

Projection, general `WHERE`, `ORDER BY`, `JOIN`, non-primary-key-targeted
mutations, primary-key updates, defaults, `NULL`, quoted identifiers, and
transactions are out of scope.

## Output

`SELECT * FROM <table_name>;` prints the catalog column order as a header. For
tables without a primary key, rows print in successful `INSERT` append order.
For tables with an `INT PRIMARY KEY` or `INTEGER PRIMARY KEY`, rows print in ascending primary-key order.
`SELECT * FROM <table_name> WHERE <primary_key_column> = <int_value>;` prints
the header and the matching row, or only the header when the key is missing.
After `CREATE INDEX`, equality and `BETWEEN` predicates on the indexed `INT`
column use the secondary index. `BETWEEN` boundaries are inclusive. Secondary
results are ordered by secondary key ascending, then by primary-key value for
primary-key tables or durable row insertion order for tables without a primary
key. A range with `low > high` prints the header only through the secondary
range path.
Fields are delimited with `|`, and each output line ends with `\n`. Empty tables
print only the header. Multiple `SELECT` statements repeat headers without blank
lines, separators, or count lines.

`UPDATE` and `DELETE` are mutation statements and write no stdout on success.
They require an equality predicate on the table's integer primary-key column.
`UPDATE` may set one existing non-primary-key column to a value matching that
column's declared type. `DELETE` makes the matching row invisible to table
scans, primary-key lookup, and secondary-index equality/range scans. Missing
primary-key targets are successful no-ops.

If any statement in a command fails, command stdout is empty even if an earlier
statement produced a result set. This task does not provide command-level
atomicity: successful statements before the failure remain durable, the failing
statement appends no partial SQL record, and later statements are not executed.

## Error Contract

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

Semantic errors occur inside the supported grammar and exit `2` with empty
stdout:

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

`CREATE INDEX` against a missing table uses:

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

Invalid SQL logical records exit `1`, write empty stdout, and use this stderr:

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

Persisted duplicate primary-key values in otherwise valid SQL catalog/row
records exit `1`, write empty stdout, and use this stderr:

```text
error: invalid SQL storage record: duplicate primary key for table users: 2
hint: primary key values must be unique in persisted SQL storage.
```

## SQL Logical Records

The page file format remains the V1 page format in `docs/file_format.md`.
SQL catalog and row data are stored as opaque `PageStore` record payloads.

Each SQL payload starts with the UTF-8 compatible prefix `PDBSQL1\0`, followed
by a one-byte record kind:

```text
C  catalog record
R  row record
E  secondary-index backfill entry record
X  committed secondary-index metadata record
I  atomic indexed row record
U  update existing row slot record
D  delete existing row slot record
```

Catalog payload body:

```text
PDBSQL1\0
C
u16 table_name_len
table_name UTF-8 bytes
u16 column_count
repeat column_count:
  u16 column_name_len
  column_name UTF-8 bytes
  u8 type tag: I for INT, T for TEXT
optional primary-key extension for new primary-key tables:
  u8 extension tag: P
  u16 zero-based primary-key column index
```

Row payload body:

```text
PDBSQL1\0
R
u16 table_name_len
table_name UTF-8 bytes
u16 value_count
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len
  value UTF-8 bytes
```

Committed secondary-index metadata payload body:

```text
PDBSQL1\0
X
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 indexed_column little-endian
u8 tie_break_mode: P for primary-key value, R for row insertion order
```

Secondary-index backfill entry payload body:

```text
PDBSQL1\0
E
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes
i64 indexed_key little-endian
i64 tie_break little-endian
u64 row_position little-endian
```

Atomic indexed row payload body:

```text
PDBSQL1\0
I
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 value_count little-endian
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count little-endian
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

Update existing row slot payload body:

```text
PDBSQL1\0
U
u64 row_position little-endian
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 value_count little-endian
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count little-endian
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

Delete existing row slot payload body:

```text
PDBSQL1\0
D
u16 table_name_len little-endian
table_name UTF-8 bytes
u64 row_position little-endian
```

For row values, `INT` payload bytes are the canonical decimal UTF-8 rendering
of the parsed signed 64-bit integer. For example, SQL literal `-0` is stored and
read back as `0`. `TEXT` payload bytes are the literal text bytes inside the
single-quoted SQL literal.

Existing arbitrary `PageStore` payloads without this SQL prefix are not valid
SQL database records and fail with the invalid SQL storage record error above.

Primary indexes are not persisted as separate records, pages, or metadata
files. The only persisted primary-key metadata is the optional catalog extension
that marks which `INT` column owns the primary-key constraint. On every
`db exec` invocation, the executor reads catalog and row records and rebuilds an
in-memory `BTreeMap` primary index from durable row records. Existing row-only
SQL catalog records without the primary-key extension remain compatible and load
as non-primary-key tables, so `SELECT *` keeps insert-order output for those
tables.

If durable row records for a primary-key table contain duplicate primary-key
values, `db exec` fails through the duplicate-primary-key invalid-storage
stderr above. If durable row records contain non-canonical integer values,
output-breaking text, unknown type tags, or other corrupt SQL logical-record
data, `db exec` fails through the generic unknown-record-tag invalid-storage
stderr above. There is no missing-index-metadata failure mode for primary
indexes because no separate primary-index metadata is stored.
Secondary indexes use committed `X` metadata plus matching `E` backfill entries
or embedded entries in `I` records. `CREATE INDEX` appends `E` records before
the final `X` commit marker; orphan `E` records without matching committed
metadata are ignored and can be retried with a fresh build id. After a table has
committed secondary indexes, inserts use a single `I` record containing the row
and all required embedded index entries, not `R` plus standalone `E`.
Updates use one `U` record and keep the row slot stable. Deletes use one `D`
record and tombstone the existing row slot. On replay, primary and secondary
indexes are rebuilt only for visible row slots.
