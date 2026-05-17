# Tasks: Minimal SQL Schema/Execute Path

## Canonical Task List
| Task ID | Title | Scope | Status |
|---|---|---|---|
| `T001` | Reconfirm repo context | Read latest HEAD, dirty state, latest review/report files, and current `src`, `tests`, `docs` before implementation edits. | complete |
| `T002` | Add SQL CLI tests | Create `tests/sql_exec.rs` with red coverage for happy path, multi-select, empty table, errors, restart, mid-command failure, and unknown SQL storage record. | complete |
| `T003` | Update CLI contract tests | Update `tests/cli_contract.rs` so help promotes `exec <path> <sql>` to supported and existing unsupported CLI behavior remains covered. | complete |
| `T004` | Implement SQL module | Add `src/sql.rs` with parser, statement model, catalog rebuild, executor, logical record encode/decode, and typed errors. | complete |
| `T005` | Wire library and CLI | Export `sql` from `src/lib.rs` and route `db exec <path> <sql>` in `src/main.rs` with exact stdout/stderr/exit mapping. | complete |
| `T006` | Update durable docs | Update `docs/cli_contract.md`, create `docs/sql_subset.md`, and update `docs/file_format.md` SQL logical-record compatibility notes. | complete |
| `T007` | Run verification | Run required commands: `cargo test --test sql_exec`, `cargo test --test cli_contract`, `./scripts/verify`, and required CLI smoke. | complete |
| `T008` | Record evidence | Update the phase/final run report with command results, changed files, acceptance mapping, and blockers if any. | complete |

## Detailed Subtasks

### T001: Reconfirm Repo Context
- Run `git rev-parse HEAD` and `git status --short`.
- Read `AGENTS.md`, `spec.md`, `contracts.md`, and latest verifier/reviewer files if present.
- Confirm no protected `ssot/` or `policies/` edits are required.
- Stop if repo reality conflicts with frozen contract.

### T002: Add SQL CLI Tests
- Add helper for invoking `db` and decoding stdout/stderr.
- Add deterministic temp DB helper.
- Assert happy path stdout exactly `id|name\n1|ada\n2|bea\n`.
- Assert restart persistence with a second process.
- Assert mid-command failure leaves only prior durable rows after restart.
- Assert multi-select stdout exactly `id|name\nid|name\n1|ada\n`.
- Assert empty table SELECT emits header only.
- Assert failed command stdout is empty even if an earlier statement in the same command selected rows.
- Assert identifier case-insensitive lookup with `CREATE TABLE Users ...; SELECT * FROM users;`.
- Assert duplicate table case variant reports the new input spelling `Users`.
- Assert duplicate column case variant reports the new input spelling `ID`.
- Assert unsupported SQL, malformed SQL, duplicate table, missing table for `INSERT`, missing table for `SELECT`, duplicate column, column count mismatch, type mismatch, and unknown SQL storage record exact stderr/exit/stdout.

### T003: Update CLI Contract Tests
- Move `exec <path> <sql>` from reserved future command expectations to supported command expectations.
- Add coverage that malformed `exec` arity follows unsupported CLI input behavior if implementation uses the first unsupported token.
- Preserve `db --help` and `db help` identical stdout, exit `0`, empty stderr.
- Preserve unsupported non-exec commands such as `open demo.db` as exit `2`.

### T004: Implement SQL Module
- Define `SqlError` variants for unsupported, malformed, semantic, invalid SQL storage record, and page storage failures.
- Parse statements with semicolon enforcement and single-quote awareness.
- Implement identifier, type, int literal, and text literal validation.
- Rebuild `Database` from `PageStore::read_records`.
- Reject non-SQL or unknown SQL records deterministically.
- Execute statements sequentially, validating before append.
- Buffer stdout and return it only on full command success.

### T005: Wire Library and CLI
- Add `pub mod sql;` in `src/lib.rs`.
- Update `HELP` to list `exec <path> <sql>` under supported commands.
- Add match arm for exactly `["exec", path, sql]`.
- Map SQL user errors to exit `2`, invalid SQL storage/page errors to exit `1`, and success to exit `0`.
- Keep all printing in `main` and all behavior computation in `sql`.

### T006: Update Durable Docs
- `docs/cli_contract.md`: command surface, help stdout, exit codes, unsupported input, `exec` success/error behavior.
- `docs/sql_subset.md`: supported grammar, non-goals, output contract, semantic failure matrix, persistence behavior, unknown record error.
- `docs/file_format.md`: existing page format unchanged; SQL logical records are `PDBSQL1\0` opaque payloads with `catalog` and `row` kinds.
- Keep exact stderr strings identical to tests and `contracts.md`.

### T007: Run Verification
- `cargo fmt`
- `cargo test --test sql_exec`
- `cargo test --test cli_contract`
- `./scripts/verify`
- Required CLI smoke:
```bash
tmp_db="$(mktemp -t pdb-sql-smoke.XXXXXX)" && cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;"
```

### T008: Record Evidence
- Record stdout/stderr/exit for CLI smoke.
- Link each acceptance criterion to test or command evidence.
- If a required command fails, record the failure and do not mark implementation complete.

## Acceptance Trace
| Contract Area | Evidence Task |
|---|---|
| `db exec <path> <sql>` command surface | T003, T005, T007 |
| happy path and row ordering | T002, T004, T007 |
| case-insensitive identifiers | T002, T004 |
| multi-select and empty table output | T002, T004 |
| unsupported and malformed SQL | T002, T004, T006 |
| semantic failure matrix | T002, T004, T006 |
| restart persistence | T002, T004 |
| mid-command failure durability | T002, T004 |
| unknown SQL storage record | T002, T004, T006 |
| page format compatibility | T004, T006, existing `tests/page_storage.rs`, `./scripts/verify` |
