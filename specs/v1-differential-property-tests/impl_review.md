# Implementation Verification Review: v1-differential-property-tests

Verdict: PASS

## Scope

- Phase: `impl_verify`
- Verification run: `impl_verify_1_fresh_20260518_054020_570930_b4b1f8a2`
- Gate: `gate-v1-differential-property-tests`
- Requirement: `req-v1-differential-property-proof`
- Reviewed current worktree implementation for `tests/differential_property.rs`, `scripts/verify_differential_property`, `docs/testing.md`, `Cargo.toml`, and `Cargo.lock`.
- `git log --oneline main..HEAD` was empty and `git diff main...HEAD` had no committed branch delta; verification target is the current dirty worktree delta. Current dirty entries are `Cargo.toml`, `Cargo.lock`, `docs/testing.md`, `scripts/verify_differential_property`, `tests/differential_property.rs`, and the task spec directory.
- `docs/cli_contract.md` was checked and has no diff.

## Executed Checks

- `cargo tree --edges normal` passed and showed only the root crate in the normal dependency tree.
- `./scripts/verify_differential_property` passed: `tests/differential_property.rs`, 1 test passed.
- `(cd /tmp && <repo>/scripts/verify_differential_property)` passed: outside-cwd script execution resolved repo root and ran the same test.
- `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture` passed: 1 test passed.
- `./scripts/verify` passed: fmt, clippy, full test suite, doc tests, and `db --help` smoke completed successfully.
- `find target/differential_property -maxdepth 3 -type f -print` produced no failure artifacts in this green verification pass.

## Evidence

- SQLite oracle is implemented with `rusqlite::{params, Connection, ErrorCode}` in `tests/differential_property.rs`; SQLite executes `CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)`, parameterized inserts, ordered full scans, and primary-key lookups.
- Actual `db` behavior is exercised through `env!("CARGO_BIN_EXE_db")` with one `db exec` process per operation and parsed CLI stdout/stderr observations.
- Generator coverage constants require at least 100 operations and 25 successful unique rows per default seed. Coverage assertions require create table, duplicate primary-key insert, missing lookup, and ordered scan.
- Duplicate primary-key operations are normalized and asserted as `DuplicatePrimaryKeyError` on both SQLite and `db`; missing lookups compare empty row vectors; full scans compare against SQLite `ORDER BY id`.
- Failure reporting source path prints seed, failing operation index, reproducible operation sequence, SQLite expected rows, `db` actual rows, artifact path, and rerun command, and writes `target/differential_property/failures/<seed>.json`.
- `scripts/verify_differential_property` changes to the repo root based on its own path and runs exactly `cargo test --test differential_property -- --nocapture`; executable bit is set.
- `docs/testing.md` documents the script, seed replay, prefix replay, failure artifact location, and generated-local-evidence status.
- `Cargo.toml` adds `rusqlite` only under `[dev-dependencies]`; `[dependencies]` remains empty, and `cargo tree --edges normal` confirms no production dependency expansion.
- `docs/cli_contract.md` is unchanged, and its existing contract states primary-key table scans are in ascending primary-key order.
- Evidence maps directly to `gate-v1-differential-property-tests` and closes `req-v1-differential-property-proof` with executable SQLite differential/property checks plus the required task-specific verification command.

## Primary Success Claims

1. `req-v1-differential-property-proof` is satisfied by a deterministic SQLite-backed differential/property harness for the supported SQL subset.
2. The task-specific verification surface is present and executable from both repo root and an external caller cwd, while baseline `./scripts/verify` remains green.
3. The change remains test/documentation scoped: SQLite is test-only, the CLI contract is unchanged, and docs describe replay plus local failure evidence without changing user-facing CLI behavior.

## Evidence Used

- Command output from `./scripts/verify_differential_property`: `deterministic_sql_subset_matches_sqlite_oracle ... ok`, 1 passed.
- Command output from `(cd /tmp && <repo>/scripts/verify_differential_property)`: 1 passed, confirming outside-cwd execution.
- Command output from `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture`: 1 passed, confirming seed/prefix replay path.
- Command output from `./scripts/verify`: all integration tests passed, including `tests/differential_property.rs`, followed by successful `target/debug/db --help`.
- Command output from `cargo tree --edges normal`: only `persistent-db-core` appears in the normal dependency tree.
- File review of `tests/differential_property.rs` lines 153-181, 185-223, 314-345, 348-407, 410-467, and 469-568.
- File review of `scripts/verify_differential_property` lines 1-7.
- File review of `docs/testing.md` lines 1-24.
- File review of `Cargo.toml` lines 10-13.
- `git diff -- docs/cli_contract.md` was empty.

## Proxy Gap / Reward-Hacking Risk

- Because this task modifies the harness/evidence producer itself, green tests could be a false pass if the harness compared against generator bookkeeping instead of SQLite or skipped the real `db` binary.
- Green-path execution does not naturally exercise mismatch artifact creation, so failure-reporting compliance could be overstated by test success alone.
- A dedicated task command could pass while broader repo verification fails, or the SQLite dependency could accidentally expand production dependencies.

## Gap-Closing Check

- Source review confirmed SQLite, not in-memory bookkeeping, is the expected-result oracle: `execute_sqlite` uses `rusqlite::Connection` and SQL statements for create, insert, ordered scan, and lookup, while `execute_db` invokes `env!("CARGO_BIN_EXE_db")` through `Command`.
- Source review confirmed generator and assertion coverage: default seeds, 100-operation and 25-row thresholds, duplicate insert checks, missing lookup checks, ordered scan checks, and `expected != actual` comparison are present in `tests/differential_property.rs`.
- Source review confirmed mismatch reporting and local artifact contract: `report_failure`, `minimal_failing_prefix`, and `write_failure_artifact` print and write the required seed, index, prefix, sequence, expected/actual observations, artifact path, and rerun command.
- Runtime checks closed command-level false-pass paths: `./scripts/verify_differential_property`, outside-cwd invocation, seed/prefix replay, `cargo tree --edges normal`, `git diff -- docs/cli_contract.md`, and `./scripts/verify` all passed in this verification run.

## Open Findings

- None.

## Repair Targets

- None.

## Next Action

- Proceed to the next phase. Closeout should include the currently untracked implementation and spec artifacts in the final commit, but this is not a verification failure for the current worktree.

## Updated At

2026-05-17T20:42:26Z
