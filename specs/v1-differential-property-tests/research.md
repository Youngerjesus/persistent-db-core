# Research: v1-differential-property-tests

## Decision 1: SQLite Oracle
Use `rusqlite` as a `[dev-dependencies]` entry only. The test harness should create an in-memory SQLite connection per deterministic seed and execute equivalent schema, inserts, and selects against it.

Rationale:
- The contract requires a SQLite-backed oracle and forbids an arbitrary in-memory oracle as replacement.
- A Rust dev-dependency avoids requiring an external `sqlite3` binary in the developer environment.
- Keeping the dependency in `[dev-dependencies]` preserves the production dependency boundary.

Constraints:
- Do not add `rusqlite` under `[dependencies]`.
- Do not require network services, background processes, or external binaries.

## Decision 2: Deterministic Operation Generator
Implement a small deterministic generator inside `tests/differential_property.rs` using a fixed local PRNG, for example a simple LCG or xorshift implemented in test code.

Rationale:
- The repo prefers deterministic behavior and standard-library implementations unless a task-level reason exists.
- The task only authorizes SQLite as a test-only dependency.
- Local seed replay must produce exactly the same operation sequence across runs.

Generator requirements:
- Default deterministic seed suite should include multiple hard-coded `u64` seeds.
- Seed-specific replay should be supported by an environment variable such as `PDB_DIFF_SEED`.
- Prefix replay should be supported by an environment variable such as `PDB_DIFF_PREFIX`.
- Each default seed must generate at least 100 operations and at least 25 successful unique rows.
- The sequence must force coverage for create table, successful inserts, duplicate primary key insert, missing key lookup, primary-key lookup, full ordered scan, and post-error continuation by issuing later independent commands.

## Decision 3: SQL Shape
Use the current documented CLI-compatible SQL shape:

```text
CREATE TABLE kv (id INT PRIMARY KEY, value TEXT);
INSERT INTO kv VALUES (<id>, '<value>');
SELECT * FROM kv;
SELECT * FROM kv WHERE id = <id>;
```

Rationale:
- `docs/sql_subset.md` documents `INSERT INTO <table_name> VALUES (...)`, not a column-list insert.
- The canonical spec says `CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)` and `INSERT INTO kv (id, value) VALUES (?, ?)` semantics, but current durable docs and parser use `INT` plus no column-list insert. The implementation should satisfy semantics without introducing new syntax.
- If the implementation discovers the parser cannot express required semantics without CLI contract changes, it must stop and report a contract conflict instead of expanding SQL syntax.

## Decision 4: Result Normalization
Compare rows as parsed `(i64, String)` tuples rather than raw stdout text where possible, but preserve stdout/stderr/exit-code checks for CLI behavior.

Rationale:
- Tuple comparison makes SQLite and `db` output differences explicit while avoiding accidental formatting mismatch in the oracle layer.
- The failure report must still include SQLite expected rows and `db` actual rows.

## Decision 5: Failure Evidence
On any mismatch, write failure evidence to `target/differential_property/failures/<seed>.json` and print a human-readable failure block to stdout before panicking.

Evidence content:
- seed
- failing operation index
- replay command
- minimal reproducible operation prefix
- SQLite expected rows or expected error
- `db` actual rows, stderr, and exit code
- artifact path

The JSON can be hand-escaped with a small helper or produced through a dev-dependency only if already available. Prefer a local string escaping helper to avoid broadening dependencies.

## Decision 6: Verification Script
Add `scripts/verify_differential_property` as a bash script modeled after `scripts/verify`: resolve repo root from the script location, `cd` there, and run exactly:

```bash
cargo test --test differential_property -- --nocapture
```

Rationale:
- The contract requires the exact task-specific command and execution from outside the repo root.

## Open Risks
- If `rusqlite` native build requirements are unavailable in the target environment, implementation may need a blocker report. The contract explicitly authorizes `rusqlite`, so do not replace SQLite with an in-memory oracle.
- If current `db` ordered scan behavior drifts from `id` ascending, report a spec conflict rather than weakening the test.

