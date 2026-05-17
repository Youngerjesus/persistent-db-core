# Technical Design: v1-differential-property-tests

## Components

### Operation
Represent generated steps as an enum in `tests/differential_property.rs`:

```rust
enum Operation {
    CreateTable,
    Insert { id: i64, value: String, duplicate: bool },
    SelectAll,
    SelectById { id: i64, expected_missing: bool },
}
```

The generated sequence should include `CreateTable` exactly once at index 0. If the implementation prefers to create the table outside the generator, the failure prefix must still include it so replay is complete.

### Deterministic Generator
Use a local PRNG struct with an explicit `seed: u64`. Keep generation independent of wall-clock time.

Generation outline:
- Start with `CreateTable`.
- Insert at least 25 unique keys.
- Add at least one duplicate insert using a known inserted key.
- Add at least one missing lookup using a key never inserted.
- Add existing lookups and `SelectAll` checks.
- Fill to at least 100 operations with weighted choices over insert/select variants while maintaining a known inserted-key set for generation only.

The known inserted-key set is only generator bookkeeping. It must not be used as the expected-result oracle for assertions.

### SQLite Oracle
Create an in-memory `rusqlite::Connection` per replay:

```sql
CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)
```

For inserts, execute parameterized SQLite statements. For full scans, query:

```sql
SELECT id, value FROM kv ORDER BY id
```

For primary-key lookups, query:

```sql
SELECT id, value FROM kv WHERE id = ?
```

SQLite expected rows are authoritative for successful selects. Duplicate insert operations must produce an error from SQLite and a non-zero semantic error from `db`; the test should compare error class, not exact SQLite wording.

### `db` Runner
Run the compiled binary through `Command::new(env!("CARGO_BIN_EXE_db"))` with:

```text
db exec <temp_db_path> <sql>
```

Use one command per operation so durable restart/reopen behavior is naturally exercised across process starts.

Parse `db` stdout:
- First line is header and must be `id|value`.
- Remaining lines parse as `id|value` rows.
- Missing lookup should parse to an empty row vector.
- Full scan rows must match SQLite expected rows in ascending `id`.

For successful mutations, `db` stdout and stderr must be empty and exit code must be `0`.

### Failure Minimization
On first mismatch at operation `i`, find the shortest prefix that reproduces the failure:
- Replay prefixes from 1 through `i + 1` against fresh temp DBs and fresh SQLite connections.
- A prefix reproduces when the last operation in the prefix yields the same mismatch category.
- If no shorter prefix is found, use `i + 1`.

This is acceptable because the default sequence size is intentionally small and deterministic.

### Failure Artifact
Write `target/differential_property/failures/<seed>.json`.

Suggested JSON shape:

```json
{
  "seed": 1,
  "failing_operation_index": 42,
  "minimal_prefix_len": 17,
  "replay_command": "PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture",
  "operations": [],
  "sqlite_expected_rows": [],
  "db_actual_rows": [],
  "db_exit_code": 0,
  "db_stderr": ""
}
```

Use deterministic operation rendering so stdout and artifacts are stable.

### Replay Interface
Recommended environment variables:
- `PDB_DIFF_SEED=<u64>`: run one seed instead of the default suite.
- `PDB_DIFF_PREFIX=<usize>`: truncate the generated sequence to the first N operations.

`docs/testing.md` must document these names if used. If implementation chooses different names, update docs and failure stdout consistently.

### Verification Script
`scripts/verify_differential_property`:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo test --test differential_property -- --nocapture
```

Set executable bit during implementation.

## Contract Compatibility Notes
- `docs/cli_contract.md` already states primary-key table scans are ascending by primary key.
- Current `docs/sql_subset.md` uses `INT` and no column-list insert syntax. The test should use supported syntax while preserving the approved semantics.
- The harness is test-only and should not require source changes under `src/` unless implementation discovers a real product bug. If a product bug is found, repair must stay within the approved behavior and be covered by tests.

