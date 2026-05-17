# Implementation Verification Review: `db check` Invariant Validation

## Verdict: PASS

## Scope

- Phase: `impl_verify`
- Verification run: `impl_verify_1_fresh_20260518_043039_480280_d9a22134`
- Spec: `specs/v1-db-check-invariants/spec.md`
- Contract: `specs/v1-db-check-invariants/contracts.md`
- QA mapping: `specs/v1-db-check-invariants/qa_mapping.md`
- Brake input reviewed: `specs/v1-db-check-invariants/impl_brake_review.md`
- Verified implementation surface: `src/check.rs`, `src/main.rs`, `src/lib.rs`, `src/storage.rs`, `src/sql.rs`, `tests/db_check.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`, `docs/file_format.md`
- Diff basis: `git diff main...HEAD` and `git log --oneline main..HEAD` were empty because the implementation is currently uncommitted; `git diff` and `git status --short` identify the active worktree changes. Protected `ssot/` and `policies/` areas have no diff.

## Executed Checks

- `cargo test --test db_check`: pass, 11 tests passed, 0 failed.
- `scripts/verify`: pass. The script ran `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help`.
- `git diff --check`: pass, no whitespace errors.
- Manual runtime read-only observation with `target/debug/db exec` followed by `target/debug/db check`: pass. `db check` printed `ok: db check passed`; DB SHA-256 stayed `81b9288a91073b48568dab128d956361d31b2f6f4df2c0988fa3437107cb5bb4` before/after, and WAL SHA-256 stayed `4b12722f0785f969d4cfff6f5efc0ce0565652dce1eb89d0bcf9a0d9768d21c1` before/after.

## Evidence

- CLI route and output mapping: `src/main.rs` wires `db check <path>` to `check::check_database`, prints `ok: db check passed\n`, maps invariant failures to `error: db check failed: <label>`, and maps open/read failures to `error: could not open or read database path: <path>`.
- Checker behavior: `src/check.rs` rejects missing and non-regular paths before read, scans page records without opening `PageStore`, validates SQL logical records, then validates the WAL sidecar.
- Storage/WAL validation: `src/storage.rs` adds `read_records_for_check` and `validate_wal_for_check`, reads the WAL bytes without truncating, advances a virtual record count for complete committed `0x01` page-append frames, rejects ahead-of-store frames, and validates replayable payload size.
- SQL invariant validation: `src/sql.rs` rebuilds catalog state and primary indexes from durable records, returning distinct `storage record readability`, `catalog/record invariant`, and `primary index` labels.
- Test coverage: `tests/db_check.rs` covers valid DB success, retained chained WAL success, storage record corruption, decode-impossible SQL bytes, catalog/record contradiction, duplicate primary key, WAL ahead-of-store, count-valid unreplayable WAL payload, missing path, directory path, and Unix unreadable regular file. The assertions pin exit code, empty stdout on failure, exact success stdout, and exact failure stderr shapes.
- WAL fixture provenance: `tests/db_check.rs` constructs complete committed WAL frames with magic `PDBWAL1\0`, version `1`, state `0x01`, payload kind `0x01`, little-endian `record_count_before`, payload length, and checksum. The ahead-of-store fixture sets `record_count_before = durable_record_count + 1`.
- CLI contract coverage: `tests/cli_contract.rs` requires `db check <path>` in help output, removes `check` from reserved commands, keeps `open` and `bench` unsupported, and covers malformed `db check` arity.
- Durable docs: `docs/cli_contract.md` documents usage, exact success output, failure prefixes/labels, open/read error shape, exit-code category, and read-only/no-repair behavior. `docs/file_format.md` documents `db check` format validation and clarifies that primary indexes are rebuilt in memory rather than persisted separately.

## Primary Success Claims

1. `db check <path>` is a documented supported CLI surface that succeeds on a valid SQL database with exit code `0`, stdout exactly `ok: db check passed\n`, and empty stderr.
2. `db check` rejects required deterministic corruption classes with stable non-zero output contracts: storage readability, catalog/record invariant, primary index, WAL replay consistency, missing path, and directory path.
3. The checker is read-only for the page file and WAL sidecar, and the current task evidence comes from fresh focused and baseline verification in this worktree rather than stale run artifacts.

## Evidence Used

- `cargo test --test db_check`: 11 passed, including `check_valid_database_passes_with_exact_output`, `check_storage_readability_corruption_fails_with_stable_prefix`, `check_catalog_record_invariant_corruption_fails_with_label`, `check_primary_index_duplicate_key_corruption_fails_with_label`, `check_wal_ahead_of_store_corruption_fails_with_label`, `check_missing_path_fails_as_user_open_read_error`, and `check_directory_path_fails_as_user_open_read_error`.
- `scripts/verify`: passed with full CLI contract tests, crash matrix tests, `db_check` tests, storage tests, primary index tests, SQL tests, WAL recovery tests, doc tests, clippy, format check, and help smoke.
- File inspection: `src/check.rs`, `src/storage.rs`, `src/sql.rs`, `tests/db_check.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`, and `docs/file_format.md`.
- Runtime observation: direct `target/debug/db check` on a fresh database printed `ok: db check passed` and left both database and WAL SHA-256 hashes unchanged before/after.
- QA mapping comparison: `specs/v1-db-check-invariants/qa_mapping.md` maps T1 through T10 to preferred commands and green criteria; the executed focused and baseline commands satisfy the current task-scoped green criteria.

## Proxy Gap / Reward-Hacking Risk

- The worker modified tests and fixture writers, so green tests could be a false pass if the WAL fixture did not match the documented sidecar format or if `db check` mutated state while still returning success.
- A generic `scripts/verify` pass could miss the CLI-only acceptance details if exact stdout/stderr and directory-path evidence were not pinned in the focused test.
- A generated report or prior brake success could overstate acceptance if this verification reused stale command output instead of rerunning the required checks.

## Gap-Closing Check

- WAL fixture shape was checked directly in `tests/db_check.rs`: `committed_wal_frame` writes magic/version/frame id/`record_count_before`/state `0x01`/payload kind `0x01`/payload length/checksum/payload, and `check_wal_ahead_of_store_corruption_fails_with_label` sets `ahead_of_store_record_count_before = durable_record_count + 1`.
- Read-only behavior was checked in `src/storage.rs`: `validate_wal_for_check` uses `std::fs::read` and virtual count advancement only; it does not call `append_record_to_file`, `OpenOptions::write`, `set_len`, replay, or checkpoint logic. The manual SHA-256 observation confirmed unchanged DB and WAL bytes around an actual `db check` run.
- Exact CLI evidence was checked in `tests/db_check.rs`: helpers assert success stdout exactly `ok: db check passed\n`, failure stdout empty, invariant stderr exactly `error: db check failed: <label>\n`, open/read stderr exactly `error: could not open or read database path: <path>\n`, and exit code `1` for negative paths.
- Current-run evidence was regenerated by running `cargo test --test db_check` and `scripts/verify` in this worktree during this verification pass.

## Open Findings

- None.

## Repair Targets

- None.

## Next Action

- Proceed to the next scheduler phase. No implementation retry is required.

## Updated At

- 2026-05-17T19:33:24Z
