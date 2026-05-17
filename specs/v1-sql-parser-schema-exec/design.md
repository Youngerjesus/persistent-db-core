# Design: Minimal SQL Schema/Execute Path

## Module Boundary
```text
src/main.rs
  -> persistent_db_core::sql
       -> persistent_db_core::storage
```

`storage` remains a generic opaque-record page store. `sql` owns SQL parsing, catalog reconstruction, semantic validation, logical record encoding, and stdout production. `main` owns CLI argument validation and process exit behavior.

## Data Model
```rust
struct Database {
    tables: Vec<Table>,
}

struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}

struct Column {
    name: String,
    ty: SqlType,
}

enum SqlType {
    Int,
    Text,
}

enum Value {
    Int(i64),
    Text(String),
}
```

Tables and rows use vectors to preserve catalog order and insertion order. Case-insensitive lookup can be implemented by scanning with `eq_ignore_ascii_case`; no hash map is required for this V1 scale.

## Statement Model
```rust
enum Statement {
    CreateTable { table: String, columns: Vec<Column> },
    Insert { table: String, values: Vec<Value> },
    SelectAll { table: String },
}
```

The parser should keep the original statement text in each parse error so CLI stderr can echo the exact unsupported or malformed statement.

## Logical Storage Encoding
All SQL logical records are existing PageStore opaque payloads. The SQL layer recognizes:

```text
PDBSQL1\0catalog\t<table>\t<column>:<type>\t...
PDBSQL1\0row\t<table>\t<type>:<value>\t...
```

Implementation may use private escaping for tabs, backslashes, colons, and percent signs inside stored table names, column names, and values. User-facing `TEXT` literals still reject `|`, newline, carriage return, and single quote, so SELECT output does not need delimiter escaping.

Invalid records:
- missing `PDBSQL1\0` prefix
- unknown record kind
- malformed catalog/row field layout
- unknown stored type
- row for a table that is not present in the reconstructed catalog

The required user-facing storage error for a missing/unknown tag is:

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## CLI Flow
1. Collect args.
2. `--help` and `help`: unchanged exit `0`, empty stderr, identical stdout.
3. `exec <path> <sql>` with exactly two operands:
   - call `sql::execute(path, sql)`.
   - success: print returned stdout, exit `0`.
   - SQL user error: print exact two-line stderr, exit `2`.
   - SQL storage/page error: print deterministic stderr, exit `1`.
4. Other input: preserve unsupported CLI contract, exit `2`.

## Output Flow
The SQL executor builds a `String` output buffer. `SELECT *` appends:
- header: catalog column names joined by `|`
- each row: stored values joined by `|`
- every line ending with `\n`

The CLI prints the buffer only after every statement succeeds. On any error, the returned error carries no partial stdout.

## Semantic Validation
- Duplicate table: compare new table against all catalog table names with `eq_ignore_ascii_case`; stderr target is new input spelling.
- Duplicate column: compare within new column list with `eq_ignore_ascii_case`; stderr target is new input spelling.
- Missing table: lookup input spelling case-insensitively; stderr target is lookup input spelling.
- Column count mismatch: use catalog table spelling in message.
- Type mismatch: use catalog column spelling and normalized type names in message.

## Parser Failure Boundary
- Unsupported SQL examples fixed by contract must produce `unsupported SQL statement`.
- Broken supported shapes such as `CREATE TABLE users id INT);`, invalid identifiers, invalid types, unterminated literals, bad literal characters, and missing semicolons must produce `malformed SQL statement`.
- Parser and executor must not panic for malformed UTF-8 input because CLI args are already Rust UTF-8 `String`s.

## Test Design
Use black-box `Command::new(env!("CARGO_BIN_EXE_db"))` tests for CLI behavior. Use deterministic temp DB file paths under `std::env::temp_dir()` with process id plus test name; remove files best-effort at test end.

The unknown SQL storage record fixture should use `persistent_db_core::storage::PageStore` to append `b"legacy"` before invoking `db exec`.

## Documentation Design
Docs must repeat exact stderr strings from the contract. `docs/sql_subset.md` is the central SQL behavior reference; `docs/cli_contract.md` points to it while still documenting CLI exit behavior; `docs/file_format.md` explains that SQL records are logical payloads above the unchanged page format.

