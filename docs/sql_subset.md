# V1 SQL Subset

`db exec <path> <sql>` accepts one SQL argument. Statement delimiter is `;`,
and every statement must end with `;`. Multiple statements in one command run in
order.

## Supported Grammar

```text
CREATE TABLE <table_name> (<column_name> INT|TEXT[, <column_name> INT|TEXT]*);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
SELECT * FROM <table_name>;
```

Keywords compare ASCII case-insensitively. Identifiers must match
`[A-Za-z_][A-Za-z0-9_]*`. Table and column equality is ASCII
case-insensitive, while stored catalog spelling is preserved for headers and
errors. Types are `INT` and `TEXT`. `INT` values are signed 64-bit decimal
integers. `TEXT` values are UTF-8 strings inside single quotes; escape
sequences, embedded single quotes, `|`, newline, and carriage return are not
supported.

Projection, `WHERE`, `ORDER BY`, `JOIN`, `UPDATE`, `DELETE`, constraints,
defaults, `NULL`, quoted identifiers, and transactions are out of scope.

## Output

`SELECT * FROM <table_name>;` prints the catalog column order as a header, then
rows in successful `INSERT` append order. Fields are delimited with `|`, and
each output line ends with `\n`. Empty tables print only the header. Multiple
`SELECT` statements repeat headers without blank lines, separators, or count
lines.

If any statement in a command fails, command stdout is empty even if an earlier
statement produced a result set. This task does not provide command-level
atomicity: successful statements before the failure remain durable, the failing
statement appends no partial SQL record, and later statements are not executed.

## Error Contract

Unsupported SQL exits `2`, writes empty stdout, and uses this stderr:

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ...;
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

Invalid SQL logical records exit `1`, write empty stdout, and use this stderr:

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## SQL Logical Records

The page file format remains the V1 page format in `docs/file_format.md`.
SQL catalog and row data are stored as opaque `PageStore` record payloads.

Each SQL payload starts with the UTF-8 compatible prefix `PDBSQL1\0`, followed
by a one-byte record kind:

```text
C  catalog record
R  row record
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

For row values, `INT` payload bytes are the canonical decimal UTF-8 rendering
of the parsed signed 64-bit integer. For example, SQL literal `-0` is stored and
read back as `0`. `TEXT` payload bytes are the literal text bytes inside the
single-quoted SQL literal.

Existing arbitrary `PageStore` payloads without this SQL prefix are not valid
SQL database records and fail with the invalid SQL storage record error above.
