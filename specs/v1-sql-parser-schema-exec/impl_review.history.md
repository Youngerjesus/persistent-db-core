## 2026-05-17T21:24:16+0900 - Archived Previous impl_review.md

# Implementation Verification Review: v1-sql-parser-schema-exec

Verdict: FAIL

## Scope

- Phase: `impl_verify`
- Task: `task-2026-05-17-19-38-21-v1-sql-parser-schema-exec`
- Reviewed current task worktree, including uncommitted modified files and untracked feature files.
- Compared implementation against `specs/v1-sql-parser-schema-exec/spec.md`, `contracts.md`, `tasks.md`, `qa_mapping.md`, and latest `impl_brake_review.md`.
- No `ssot/` or `policies/` edits were present.

## Executed Checks

- `git status --short --branch`: task branch with modified docs/source/tests and untracked feature files.
- `git log --oneline main..HEAD`: no committed task delta; verification target is the current worktree.
- `git diff --name-only && git ls-files --others --exclude-standard`: confirmed touched files are task-scoped and no protected paths are edited.
- `cargo test --test sql_exec`: pass, 18 tests.
- `cargo test --test cli_contract`: pass, 5 tests.
- `./scripts/verify`: pass; includes fmt check, clippy with warnings denied, full test suite, and help smoke.
- Required CLI smoke command: pass with exit `0`, stdout `id|name\n1|ada\n2|bea\n`, and empty stderr.
- Manual parser-boundary probes:
  - `INSERT users VALUES (1);`: exit `2`, unsupported-SQL stderr.
  - `CREATE users (id INT);`: exit `2`, unsupported-SQL stderr.
  - `SELECT FROM users;`: exit `2`, unsupported-SQL stderr.
  - `SELECT * FROM users extra;`: exit `2`, malformed-SQL stderr.
- Documentation parity check:
  - `rg -n "Users|ID|case-variant|case variant|table already exists|duplicate column" docs/cli_contract.md docs/sql_subset.md specs/v1-sql-parser-schema-exec/contracts.md`
  - `nl -ba docs/cli_contract.md | sed -n '80,112p'`
  - `nl -ba docs/sql_subset.md | sed -n '56,80p'`

## Evidence

- Runtime behavior covers the SQL happy path, restart persistence, mid-command failure persistence, failure stdout suppression, case-insensitive identifiers, unsupported/malformed SQL, semantic failure matrix, and invalid SQL storage record handling through `tests/sql_exec.rs`.
- CLI surface behavior covers `--help`, `help`, unsupported commands, reserved future commands, and malformed `exec` arity through `tests/cli_contract.rs`.
- `src/sql.rs` stores SQL logical records over `PageStore` opaque payloads with `PDBSQL1\0` prefix and rejects non-SQL or invalid SQL-prefixed records.
- `docs/sql_subset.md` documents SQL grammar, output behavior, semantic errors, case-variant duplicate target spelling, and SQL logical record encoding.
- `docs/file_format.md` documents that the lower-level page format is unchanged and SQL records are opaque payloads.

## Primary Success Claims

1. The implementation adds `db exec <path> <sql>` for the approved minimal `CREATE TABLE`, `INSERT INTO ... VALUES`, and `SELECT * FROM ...` SQL subset without breaking the existing CLI contract.
2. SQL catalog and row state persist through the existing page-storage primitive, preserve insertion order, reject invalid SQL storage records deterministically, and keep pre-failure successful statements durable without command-level atomicity.
3. Durable docs mirror the accepted CLI, SQL error, semantic failure, and SQL logical record contracts.

## Evidence Used

- Claim 1 was evaluated with `cargo test --test cli_contract`, `cargo test --test sql_exec`, `./scripts/verify`, and the required CLI smoke command.
- Claim 2 was evaluated with `tests/sql_exec.rs` restart, mid-command failure, unknown record, SQL-prefixed invalid catalog/row, and noncanonical persisted-int tests, plus source review of `src/sql.rs`.
- Claim 3 was evaluated by comparing `docs/cli_contract.md`, `docs/sql_subset.md`, and `docs/file_format.md` against `contracts.md`, especially the semantic failure matrix and SQL logical-record requirements.

## Proxy Gap / Reward-Hacking Risk

- Green tests could still miss a documentation-contract failure because tests assert runtime stderr strings but do not require `docs/cli_contract.md` and `docs/sql_subset.md` to carry the same semantic failure matrix details.
- Since this task modified test and fixture files, a false pass could occur if tests cover the implemented behavior but omit a contract-required documentation parity case.
- The broad malformed-vs-unsupported parser boundary from `IBR-005` could be over-interpreted, but the explicit contract examples and required matrix do not require `INSERT users VALUES (1);`, `CREATE users (id INT);`, or `SELECT FROM users;` to be malformed rather than unsupported.

## Gap-Closing Check

- For the documentation parity false-pass path, `rg` and numbered doc excerpts show that `docs/sql_subset.md:63` and `docs/sql_subset.md:76` document case-variant duplicate targets `Users` and `ID`, while `docs/cli_contract.md:88` through `docs/cli_contract.md:112` lists the base semantic error strings only and omits those case-variant target-spelling notes. `contracts.md` requires duplicate table and duplicate column case-variant targets to be part of the semantic failure contract and requires that semantic failure matrix documentation be present in both durable docs.
- For the parser-boundary risk, manual runtime probes confirmed the previously repaired explicit trailing-token case is malformed, while missing-keyword forms fail as unsupported SQL. This is acceptable for this verification because those forms are not part of the explicit required malformed examples or semantic matrix.

## Open Findings

- `IMPL-VERIFY-001` - Documentation parity gap; severity: retry-blocking. `docs/cli_contract.md` does not document the duplicate table and duplicate column case-variant target-spelling rules required by `contracts.md`. The SQL subset doc includes `Users` and `ID` notes, but the CLI contract semantic matrix only lists the base `users` and `id` examples at `docs/cli_contract.md:88` through `docs/cli_contract.md:112`. The contract requires semantic failure matrix documentation in both `docs/cli_contract.md` and `docs/sql_subset.md`, so acceptance claim 3 is not fully closed.

## Repair Targets

- Update `docs/cli_contract.md` under SQL semantic errors to include the duplicate table case-variant target rule for `Users` and duplicate column case-variant target rule for `ID`, matching the contract and `docs/sql_subset.md`.
- Rerun `cargo test --test sql_exec`, `cargo test --test cli_contract`, `./scripts/verify`, and the required CLI smoke after the doc repair.

## Next Action

Return to implementation repair for the durable documentation gap only. No production runtime defect was found in this verification pass.

## Updated At

2026-05-17T21:10:07+0900
