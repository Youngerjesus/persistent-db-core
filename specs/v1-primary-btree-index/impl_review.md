Verdict: PASS

## Scope
- Phase: Implementation Verification, round 1.
- Task: `task-2026-05-17-22-43-31-v1-primary-btree-index`.
- Reviewed worktree diff from `main`/current dirty state, including `src/index.rs`, `src/lib.rs`, `src/main.rs`, `src/sql.rs`, `tests/primary_index.rs`, `tests/sql_exec.rs`, `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md`.
- Inputs reviewed: `spec.md`, `contracts.md`, `tasks.md`, `qa_mapping.md`, `impl_brake_review.md`, implementation final report, current source/tests/docs, and fresh command output.
- Protected areas: no `ssot/` or `policies/` edits observed.
- Non-visual task: browser, screenshot, DOM, and UX evidence were not used.

## Executed Checks
- `cargo test --test primary_index`: PASS, 7 passed.
- `cargo test --test sql_exec primary_key`: PASS, 11 passed, 17 filtered out.
- `./scripts/verify`: PASS, including baseline fmt/clippy/test/help workflow; observed full `cargo test` result included 5 CLI contract tests, 10 page storage tests, 7 primary index tests, 28 SQL exec tests, doc tests, and `db --help` smoke output.
- Manual runtime check for implementation-brake risk BRK-002: `cargo run --quiet --bin db -- exec <temp>/test.pdb "CREATE TABLE users (id INT PRIMARY KEY, other INT PRIMARY KEY);"` returned exit code `2`, empty stdout, and `error: SQL semantic error: multiple primary key columns for table users`.

## Evidence
- `src/index.rs` defines `PrimaryIndex` over `BTreeMap<i64, usize>` with duplicate-aware `insert`, exact `get`, and ascending `ordered_positions`.
- `src/sql.rs:120-160` rebuilds per-table primary indexes from durable row records during `Database::from_records`; duplicate persisted primary keys map to `SqlError::InvalidStorageRecord`.
- `src/sql.rs:290-316` validates one optional `INT PRIMARY KEY` column and initializes table primary-index state.
- `src/sql.rs:357-379` checks duplicate primary keys before `append_record`, then inserts the row position into `PrimaryIndex`.
- `src/sql.rs:383-429` routes PK table `SELECT *` through `ordered_positions` and exact PK predicates through `PrimaryIndex::get`; non-PK tables keep insert-order scans.
- `src/sql.rs:638-665` parses only exact `WHERE <column> = <int>` predicates into the primary-key select path.
- `src/sql.rs:898-964` encodes/decodes optional catalog primary-key metadata while accepting old catalog records without the extension.
- `tests/primary_index.rs` covers primitive behavior, ordered traversal, empty traversal, reopen/rebuild, duplicate persisted key invalid-storage failure, missing/empty output, and old row-only catalog compatibility.
- `tests/sql_exec.rs` covers the specified CLI stdout/stderr/exit-code behavior for exact lookup, ordered scan, missing lookup, duplicate insert, empty PK table scan, non-PK insert-order preservation, invalid `TEXT PRIMARY KEY`, non-PK predicate rejection, range predicate rejection, and `ORDER BY` rejection.
- `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md` document PK grammar, ordered scan behavior, duplicate PK error, no separate persisted index metadata, rebuild from durable row records, row-only compatibility, corrupt row failure through invalid SQL storage record, and no missing-index-metadata failure mode.

## Primary Success Claims
1. The implementation adds deterministic primary-index primitives and rebuilds in-memory primary indexes from persisted SQL row records without adding separate persisted index metadata.
2. `db exec` supports the approved single `INT PRIMARY KEY` SQL slice: exact primary-key lookup, PK-ordered `SELECT *`, duplicate PK rejection before append, missing-key header-only output, empty PK scans, and preservation of insert-order scans for non-PK tables.
3. The durable docs and compatibility behavior match the contract: existing row-only SQL catalogs remain readable as non-PK tables, and corrupt/duplicate persisted PK row data fails through the existing invalid SQL storage record path.

## Evidence Used
- Fresh commands from this verification round:
  - `cargo test --test primary_index` -> 7 passed.
  - `cargo test --test sql_exec primary_key` -> 11 passed.
  - `./scripts/verify` -> passed full baseline and help smoke.
  - Manual multiple-PK CLI runtime check -> exit `2`, empty stdout, deterministic semantic error.
- Source inspection:
  - `src/index.rs`
  - `src/sql.rs:120-160`
  - `src/sql.rs:290-316`
  - `src/sql.rs:357-429`
  - `src/sql.rs:638-665`
  - `src/sql.rs:898-964`
- Artifact inspection:
  - `specs/v1-primary-btree-index/qa_mapping.md`
  - `specs/v1-primary-btree-index/impl_brake_review.md`
  - implementation report at `runs/impl_exec_fresh_20260517_231225_517160_79cd7c99/final.md`
  - durable docs in `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md`

## Proxy Gap / Reward-Hacking Risk
- The worker changed behavior tests and fixtures, so green tests alone could be a false pass if the tests asserted the new code's shape rather than the contract's user-observable behavior.
- A false pass could occur if exact lookup tests passed through a full scan or if PK ordering was only a test artifact rather than actual `PrimaryIndex` routing.
- A false pass could occur if the final report claimed indexed lookup without clarifying that each `db exec` process still replays durable records and rebuilds the in-memory index on open.
- A false pass could occur if catalog compatibility was only documented but old row-only records failed at runtime.

## Gap-Closing Check
- The false-pass risk from modified tests is closed by current-run black-box CLI checks in `tests/sql_exec.rs` plus source-path inspection: `parse_select` creates `Statement::SelectPrimaryKey` in `src/sql.rs:638-665`, `execute_select_primary_key` uses `existing.primary_index.get(key)` in `src/sql.rs:403-429`, and PK `SELECT *` uses `existing.primary_index.ordered_positions()` in `src/sql.rs:383-399`.
- The cold-start/rebuild ambiguity is closed by `src/sql.rs:120-160`, which rebuilds `PrimaryIndex` during durable record replay, and by docs explicitly stating rebuild-on-open/no separate index metadata in `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md`.
- The row-only compatibility risk is closed by `primary_index_existing_row_only_catalog_remains_insert_order` in `tests/primary_index.rs`, which appends old-format catalog records directly through `PageStore` and verifies insert-order `SELECT *` after reopen.
- The multiple-primary-key verify-risk from `impl_brake_review.md` is closed for acceptance by manual CLI runtime evidence: `CREATE TABLE users (id INT PRIMARY KEY, other INT PRIMARY KEY);` returned exit `2`, empty stdout, and the deterministic multiple-PK semantic error.

## Open Findings
- None.

## Repair Targets
- None.

## Next Action
- Proceed to the next review/closure phase.

## Updated At
- 2026-05-17T23:27:55+0900
