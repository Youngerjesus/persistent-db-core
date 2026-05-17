# Implementation Verification Review: v1-sql-parser-schema-exec

Verdict: PASS

## Scope

- Phase: `impl_verify` attempt 2.
- Task: `task-2026-05-17-19-38-21-v1-sql-parser-schema-exec`.
- Reviewed current worktree, including modified tracked files and task-critical untracked files.
- Compared implementation against `spec.md`, `contracts.md`, `tasks.md`, `qa_mapping.md`, previous `impl_review.md`, and latest `impl_brake_review.md`.
- No `ssot/` or `policies/` edits were present.

## Executed Checks

- `git status --short --branch`: current verification target is a dirty worktree with task-scoped modified and untracked files.
- `git log --oneline main..HEAD`: no committed task delta; `HEAD` and `main` are both `8aea6208d2a42d51a78306ccd57dbbc5e7aad6a4`.
- `git diff --name-only` plus `git ls-files --others --exclude-standard`: changed files are task-scoped docs/source/tests/spec artifacts.
- `git status --short -- ssot policies`: no protected-area edits.
- `git diff --check`: pass.
- `cargo test --test sql_exec`: pass, 18 tests.
- `cargo test --test cli_contract`: pass, 5 tests.
- `./scripts/verify`: pass; script runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`.
- Required CLI smoke: pass; exit `0`, stdout `id|name\n1|ada\n2|bea\n`, stderr empty.
- Manual parser-boundary probes:
  - `INSERT users VALUES (1);`: exit `2`, unsupported-SQL stderr.
  - `CREATE users (id INT);`: exit `2`, unsupported-SQL stderr.
  - `SELECT FROM users;`: exit `2`, unsupported-SQL stderr.
  - `SELECT * FROM users extra;`: exit `2`, malformed-SQL stderr.
- Manual page-corruption SQL CLI probe: a short file containing `short` returned exit `1`, empty stdout, and storage stderr `error: storage error: TruncatedFile`.
- Documentation parity check with `rg` and numbered excerpts across `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md`, and `contracts.md`.

## Evidence

- `tests/sql_exec.rs` covers happy path, insertion-order row output, empty-table header-only output, multiple SELECT headers without separators, failed-command stdout suppression, restart persistence, mid-command failure durability, case-insensitive identifier lookup with preserved catalog spelling, unsupported SQL, malformed SQL, the required semantic failure matrix, unknown non-SQL storage payloads, and invalid SQL-prefixed record fixtures.
- `tests/cli_contract.rs` covers help parity, supported `exec <path> <sql>` help surface, reserved `open` behavior, unsupported argument behavior, and malformed `exec` arity.
- `src/sql.rs` rebuilds catalog/rows from existing `PageStore::read_records`, appends `PDBSQL1\0` catalog/row payloads through `PageStore::append_record`, buffers stdout until all statements succeed, and validates loaded SQL logical records before accepting them.
- `src/main.rs` maps SQL user errors to exit `2`, invalid SQL logical records and storage errors to exit `1`, and successful execution to exit `0`.
- `docs/cli_contract.md`, `docs/sql_subset.md`, and `docs/file_format.md` document the command surface, exact SQL error strings, case-variant duplicate target spelling, SQL logical record encoding, and unchanged page-storage framing.

## Primary Success Claims

1. The current worktree implements `db exec <path> <sql>` for the approved minimal `CREATE TABLE`, `INSERT INTO ... VALUES`, and `SELECT * FROM ...` subset without regressing the documented help and unsupported CLI contracts.
2. SQL catalog/row persistence is implemented on top of the existing page-storage primitive, preserves insert order across restart, rejects invalid SQL storage records deterministically, and keeps successful pre-failure statements durable while suppressing failed-command stdout.
3. Durable documentation now matches the accepted CLI, SQL grammar, semantic failure matrix, case-variant target spelling, storage error boundary, and SQL logical-record encoding contracts.

## Evidence Used

- Claim 1: `cargo test --test cli_contract`, `cargo test --test sql_exec`, `./scripts/verify`, `src/main.rs` review, and required CLI smoke output `id|name\n1|ada\n2|bea\n`.
- Claim 2: `tests/sql_exec.rs` restart/mid-command/unknown-record/invalid-SQL-prefixed fixture tests, `tests/page_storage.rs` through `./scripts/verify`, manual truncated-file SQL CLI probe, and source review of `src/sql.rs` record load/append paths.
- Claim 3: `rg -n "Users|ID|case-variant|table already exists|duplicate column|PDBSQL1|invalid SQL storage" docs/cli_contract.md docs/sql_subset.md docs/file_format.md specs/v1-sql-parser-schema-exec/contracts.md`, plus numbered excerpts showing `docs/cli_contract.md:95-109` and `docs/sql_subset.md:63-77` both document the duplicate target-spelling case variants.

## Proxy Gap / Reward-Hacking Risk

- Previous verification failed on documentation parity even though runtime tests were green, so green tests alone could still miss a durable-doc contract gap.
- This task modifies test and fixture files, so a false pass could occur if the changed tests merely matched implementation quirks while omitting required behavior from the contract.
- The broader malformed-vs-unsupported boundary from `IBR-005` remains an interpretation risk for SQL-like statements outside the explicit examples.
- The page-corruption surface from `IBR-008` could be missed if only the unknown SQL logical-record fixture were considered.
- The current task-critical implementation files are untracked, so verification success could be lost later if closeout/merge packaging omits them.

## Gap-Closing Check

- Documentation parity gap is closed by concrete file evidence: `docs/cli_contract.md:95-109` now documents case-variant duplicate table target `Users` and duplicate column target `ID`, matching `docs/sql_subset.md:63-77` and `contracts.md:92-109`.
- Test-substitution risk is closed by full integration evidence, not only unit fixtures: `./scripts/verify` passed the complete suite and help smoke, and the required real CLI smoke returned exit `0`, stdout `id|name\n1|ada\n2|bea\n`, and empty stderr.
- Parser-boundary risk was checked with real CLI probes. Missing-keyword forms returned unsupported SQL, while the explicit repaired shape `SELECT * FROM users extra;` returned malformed SQL; this satisfies the explicit contract matrix and examples used for this task.
- Page-corruption risk was checked with a real truncated database file: `db exec <short-file> "SELECT * FROM users;"` returned exit `1`, empty stdout, and deterministic storage stderr, so the SQL layer did not hide page-level corruption as success.
- Packaging risk is recorded for closeout: `git ls-files --others --exclude-standard` lists task-critical untracked files including `src/sql.rs`, `tests/sql_exec.rs`, and `docs/sql_subset.md`; this is not an implementation retry blocker for current-worktree verification, but closeout must include them.

## Open Findings

- None.

## Repair Targets

- None for implementation retry.
- Closeout/merge must include task-critical untracked files in the submitted delta and rerun required verification against that packaged state.

## Next Action

Proceed to the next phase. Current implementation verification passes.

## Updated At

2026-05-17T21:24:16+0900
