Verdict: PASS

Updated At: 2026-05-19T14:31:46+0900

## Scope

- Phase: `impl_verify` fresh verification pass for `task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency`.
- Baseline comparison: `git log --oneline main..HEAD` and `git diff --name-status main...HEAD` were empty; current verification target is the dirty worktree diff.
- Dirty worktree under review: `src/sql.rs`, `src/index.rs`, `tests/secondary_index.rs`, `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md`, and new `specs/v1-secondary-index-mutation-consistency/` artifacts.
- Reviewed inputs: `spec.md`, `contracts.md`, `qa_mapping.md`, `tasks.md`, and latest `impl_brake_review.md`.
- `tests/db_check.rs` focused command was not required because all new secondary-index mutation negative fixtures remain in `tests/secondary_index.rs`.

## Executed Checks

- `cargo test --test secondary_index -- --nocapture`: exit 0.
  - Summary: 29 passed, 0 failed.
  - Relevant test names: `mutation_contract_update_delete_restart_processes_keep_secondary_indexes_consistent`, `mutation_contract_retained_wal_replay_keeps_secondary_indexes_consistent`, `mutation_contract_wal_only_update_delete_frames_replay_secondary_indexes`, `mutation_contract_db_check_rejects_stale_secondary_entry_after_update`, `mutation_contract_db_check_rejects_dangling_secondary_pointer_after_delete`, `mutation_contract_db_check_rejects_missing_visible_secondary_entry`, `update_validates_set_column_and_type_before_missing_target_noop`, `noop_mutations_do_not_advance_same_command_create_index_build_id`.
- `./scripts/verify`: exit 0.
  - Summary: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, doc tests, and `cargo run --bin db -- --help` passed.
  - Full test summary included `tests/db_check.rs` 11 passed, `tests/sql_exec.rs` 28 passed, `tests/secondary_index.rs` 29 passed, and all other integration suites passed.
- Manual black-box CLI spot check with `target/debug/db`: exit 0 for setup, update, all required post-update queries, delete, all required post-delete queries, and `db check`.
  - Observed page file and `<path>.wal` sidecar both existed before cleanup.
- Manual unsupported-breadth spot check with `target/debug/db`: exit 2 for primary-key update, wrong-predicate update, and wrong-predicate delete with semantic errors.

## Evidence

- Contract fixture setup used exactly:
  - `CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);`
  - `INSERT INTO users VALUES (1, 10, 'ada');`
  - `INSERT INTO users VALUES (2, 20, 'bea');`
  - `INSERT INTO users VALUES (3, 20, 'cal');`
  - `INSERT INTO users VALUES (4, 30, 'dia');`
  - `CREATE INDEX idx_users_age ON users(age);`
- Manual CLI observed after `UPDATE users SET age = 30 WHERE id = 2;`:
  - `SELECT * FROM users WHERE age = 20;` -> `id|age|name\n3|20|cal\n`
  - `SELECT * FROM users WHERE age = 30;` -> `id|age|name\n2|30|bea\n4|30|dia\n`
  - `SELECT * FROM users WHERE age BETWEEN 20 AND 30;` -> `id|age|name\n3|20|cal\n2|30|bea\n4|30|dia\n`
  - `SELECT * FROM users WHERE id = 2;` -> `id|age|name\n2|30|bea\n`
- Manual CLI observed after `DELETE FROM users WHERE id = 3;`:
  - `SELECT * FROM users WHERE age = 20;` -> `id|age|name\n`
  - `SELECT * FROM users WHERE age BETWEEN 10 AND 30;` -> `id|age|name\n1|10|ada\n2|30|bea\n4|30|dia\n`
  - `SELECT * FROM users WHERE id = 3;` -> `id|age|name\n`
  - `SELECT * FROM users;` -> `id|age|name\n1|10|ada\n2|30|bea\n4|30|dia\n`
  - `db check <path>` -> exit 0, stdout `ok: db check passed\n`, empty stderr.
- Mutation persistence and replay evidence:
  - `mutation_contract_update_delete_restart_processes_keep_secondary_indexes_consistent` asserts separate `db` process invocation per setup, mutation, query, and `db check` step.
  - `mutation_contract_retained_wal_replay_keeps_secondary_indexes_consistent` asserts page file and retained `<path>.wal` exist before reopen query and `db check`.
  - `mutation_contract_wal_only_update_delete_frames_replay_secondary_indexes` appends committed WAL-only `U` and `D` frames and verifies replayed query output plus `db check`.
- Negative invariant evidence:
  - Stale old-key fixture, dangling pointer fixture, and missing visible indexed-row fixture each assert `db check` exit 1, empty stdout, and stderr exactly `error: db check failed: secondary index\n`.
- Requirement mapping:
  - `REQ-7-insert-update-and-delete-must-997871f9`: covered by update/delete exact-output CLI tests, process-boundary restart checks, retained WAL sidecar replay, WAL-only replay, and manual black-box CLI spot check.
  - `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`: covered by positive `db check` after mutation/replay and the three deterministic negative secondary-index fixture tests.
- Storage compatibility outcome:
  - Lower-level page/WAL framing has no storage format change.
  - Durable SQL logical-record format is intentionally extended with additive `U` and `D` records and documented in `docs/file_format.md` and `docs/sql_subset.md`.
  - Existing row-only and existing secondary-index database compatibility remains covered by existing secondary-index compatibility tests, including `old_no_index_database_reopens_then_backfills_and_post_index_insert_persists` and existing secondary-index reopen coverage.

## Primary Success Claims

- Claim 1: Primary-key-targeted `UPDATE` and `DELETE` maintain table rows, primary index, and secondary index equality/range paths consistently across separate process invocations.
- Claim 2: Mutation records survive retained WAL and WAL-only replay, and `db check` validates the replayed secondary-index state.
- Claim 3: `db check` catches stale secondary entries, dangling/deleted row pointers, and missing visible indexed rows with the required `secondary index` failure surface.

## Evidence Used

- `cargo test --test secondary_index -- --nocapture` exit 0 with the exact mutation contract tests and negative fixture tests passing.
- `./scripts/verify` exit 0 with fmt, clippy, full tests, doc tests, and help smoke passing.
- Manual `target/debug/db` CLI spot check reproduced the required UPDATE/DELETE query stdout, silent mutation stdout/stderr, positive `db check`, and page/WAL sidecar existence.
- Source review of `src/sql.rs` confirmed successful mutations append `U`/`D` records before applying runtime state, and that table/primary/secondary scans skip tombstoned rows through visible row slots.
- Source review of `tests/secondary_index.rs` confirmed the contract test invokes the compiled `db` binary via `Command::new(env!("CARGO_BIN_EXE_db"))` for each setup, mutation, query, and check step.

## Proxy Gap / Reward-Hacking Risk

- False-pass path 1: Because `tests/secondary_index.rs` and fixture helpers were modified, green tests alone could hide a harness shortcut that does not exercise the real CLI process path.
- False-pass path 2: Retained WAL existence alone could pass without proving mutation replay if the page file already contained the same `U`/`D` records.
- False-pass path 3: Unsupported mutation breadth could silently become supported beyond the approved primary-key-targeted forms.

## Gap-Closing Check

- For false-pass path 1, manual `target/debug/db` commands outside the Rust test harness reproduced all contract UPDATE/DELETE stdout/stderr/exit-code outcomes and `db check` success on a temporary database.
- For false-pass path 2, `tests/secondary_index.rs::mutation_contract_wal_only_update_delete_frames_replay_secondary_indexes` appends committed `U` and `D` WAL frames with `record_count_before` values beyond the durable page record count, then runs reopen query and `db check`; this closes the replay-required evidence gap rather than relying only on WAL file existence.
- For false-pass path 3, manual CLI checks for `UPDATE users SET id = 9 WHERE id = 1;`, `UPDATE users SET age = 30 WHERE age = 10;`, and `DELETE FROM users WHERE age = 10;` returned exit 2 semantic errors, matching the documented narrow mutation scope.

## Open Findings

- None.

## Repair Targets

- None.

## Next Action

- Advance to the next review/closeout phase. No implementation retry is required.
