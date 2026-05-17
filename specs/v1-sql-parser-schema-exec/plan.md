# Plan: Minimal SQL Schema/Execute Path

Status: ready_for_execution

## Phase Boundary
This artifact closes plan execution only. It does not edit production code, tests, runtime config, SSOT, policies, or final verification evidence. `spec.md` and `contracts.md` are adopted without rewrite.

## Current Repo Reality
- Repo root: `persistent-db-core_worktree/task-2026-05-17-19-38-21-v1-sql-parser-schema-exec`
- Verified HEAD: `8aea6208d2a42d51a78306ccd57dbbc5e7aad6a4`
- Current dirty state observed for this phase: untracked `specs/v1-sql-parser-schema-exec/` package only.
- Existing structure: `src/main.rs`, `src/lib.rs`, `src/storage.rs`, `tests/cli_contract.rs`, `tests/page_storage.rs`, `docs/cli_contract.md`, `docs/file_format.md`.
- Protected areas remain out of scope: `ssot/`, `policies/`.

## Affected Product Contracts
- CLI behavior: promote `exec <path> <sql>` from reserved command to supported command while preserving help and unsupported-input behavior.
- SQL behavior: support exactly `CREATE TABLE`, `INSERT INTO ... VALUES`, and `SELECT * FROM ...`.
- Persisted data compatibility: keep existing page file format unchanged and add SQL logical records as opaque payloads.
- Documented errors: exact stderr, stdout emptiness, and exit codes for unsupported, malformed, semantic, and invalid SQL storage record cases.
- Tests and docs: add focused integration tests and durable docs for the new SQL subset and storage logical record encoding.

## Implementation Boundary
Implement only:
- `db exec <path> <sql>` CLI dispatch.
- `src/sql.rs` with parser, catalog rebuild, executor, SQL logical record encode/decode, and error types.
- `src/lib.rs` export for `sql`.
- `tests/sql_exec.rs` black-box CLI tests plus a PageStore fixture for unknown SQL record.
- `tests/cli_contract.rs` updates for help text and exec command behavior.
- `docs/cli_contract.md`, new `docs/sql_subset.md`, and `docs/file_format.md` SQL logical-record compatibility note.

Do not implement projection, filtering, ordering, joins, updates, deletes, transactions, WAL, indexes, optimizer behavior, shell/stdin SQL, network services, background daemons, benchmarks, or browser evidence.

## Architecture Plan
Keep dependencies one-way:
- `src/main.rs` depends on the library SQL API and maps returned errors to stdout/stderr/exit.
- `src/sql.rs` depends on `src/storage.rs`.
- `src/storage.rs` remains unaware of SQL and must not import upward.

Recommended public SQL API:
```rust
pub fn execute(path: impl AsRef<Path>, sql: &str) -> Result<String, SqlError>;
```

`execute` returns the full stdout buffer on success. It returns typed errors for CLI mapping and must not print.

## Parser Plan
- Split statements on semicolons while respecting single-quoted `TEXT` literals.
- Require every non-empty statement to terminate with `;`.
- Keywords are ASCII case-insensitive.
- Identifiers must match `[A-Za-z_][A-Za-z0-9_]*`.
- Types are exactly `INT` or `TEXT` case-insensitive but should be normalized to documented `INT`/`TEXT` in errors/storage.
- `TEXT` literals are single-quoted UTF-8 strings with no escaping and must reject embedded single quote, `|`, newline, and carriage return.
- `INT` literals parse as signed 64-bit decimal integers.
- Preserve input spelling for table/column names stored in catalog and printed headers/errors.

## Execution Plan
- On startup, open `PageStore` and read records.
- Decode only records with `PDBSQL1\0` prefix and known kind `catalog` or `row`; unknown or missing tag is invalid SQL storage record, exit `1`.
- Rebuild catalog and rows in stored append order. Treat decode failures as invalid SQL storage record unless they are page-level storage errors.
- For each parsed statement:
  - `CREATE TABLE`: check duplicate table and duplicate columns with ASCII case-insensitive equality, then append one catalog record.
  - `INSERT`: lookup table case-insensitively, validate value count and types, then append one row record.
  - `SELECT *`: lookup table case-insensitively, append header and rows to an in-memory stdout buffer.
- If a statement fails, stop execution, return empty stdout, do not run later statements, and do not append a record for the failing statement.
- Prior successful statements may remain durable because command-level atomicity is out of scope.

## Error Plan
Implement exact stderr strings from `contracts.md`:
- unsupported SQL: exit `2`
- malformed SQL: exit `2`
- duplicate table: exit `2`
- missing table for `INSERT` and `SELECT`: exit `2`
- duplicate column: exit `2`
- column count mismatch: exit `2`
- type mismatch: exit `2`
- unknown SQL storage record: exit `1`

Storage-level corruption or IO errors should be deterministic, panic-free exit `1`; do not change existing `StorageError` variants unless required by tests.

## Documentation Plan
- `docs/cli_contract.md`: supported commands include `exec <path> <sql>`; document success, unsupported CLI usage, SQL user errors, storage logical-record error, and exit codes.
- `docs/sql_subset.md`: grammar, identifier/type/literal rules, output rules, semantic failure matrix, persistence behavior, and non-goals.
- `docs/file_format.md`: preserve page format and add a SQL logical-record section clarifying `PDBSQL1\0` payloads are opaque records inside the existing page layer.

## Verification Evidence Mapping
- CLI surface and existing contract preservation: `cargo test --test cli_contract`.
- Happy path, ordering, restart persistence: `cargo test --test sql_exec`.
- Identifier case-insensitive lookup and duplicate checks: `cargo test --test sql_exec`.
- Multi-select, empty table, and empty stdout on failure: `cargo test --test sql_exec`.
- Unsupported, malformed, semantic matrix, and invalid SQL storage record: `cargo test --test sql_exec`.
- Full regression and help smoke: `./scripts/verify`.
- Manual smoke: `tmp_db="$(mktemp -t pdb-sql-smoke.XXXXXX)" && cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;"`

## Visual And UX Evidence
Browser and visual verification are not applicable acceptance evidence for this task. The contract states this is CLI-only Rust database work and browser-based artifacts are out of scope. The evidence layers are deterministic tests, exact command output, durable docs, and run report mapping.

## Risk Controls
- Do not alter `PDBV1\0\0\0`, version `1`, `PDPG`, page size, or page record framing.
- Keep SQL logical encoding small and documented; avoid general SQL feature creep.
- Buffer stdout until command success.
- Validate before append for each mutating statement.
- Escalate if a second recovery attempt would be needed or if repo reality conflicts with canonical contract.

## Execution Checklist
1. Re-check HEAD, dirty state, and latest review/report files before implementation edits.
2. Add red tests in `tests/sql_exec.rs` and adjust `tests/cli_contract.rs`.
3. Implement `src/sql.rs`, expose it from `src/lib.rs`, and wire `db exec` in `src/main.rs`.
4. Update `docs/cli_contract.md`, add `docs/sql_subset.md`, and update `docs/file_format.md`.
5. Run `cargo fmt`.
6. Run `cargo test --test sql_exec`.
7. Run `cargo test --test cli_contract`.
8. Run `./scripts/verify`.
9. Run the required CLI smoke and record stdout/stderr/exit code.

