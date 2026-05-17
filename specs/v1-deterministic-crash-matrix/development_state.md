# Development State: v1-deterministic-crash-matrix

## Current Implementation Pass

- Pass type: implementation retry repair driven by `impl_brake_review.md`.
- Implemented deterministic crash matrix coverage in `tests/crash_matrix.rs` for CM-001 through CM-006.
- Added current-run evidence generation under `target/crash_matrix/`, including `crash_matrix_report.md` assembled from observed case output.
- Tightened `scripts/verify_crash_matrix` to run the matrix test and validate every case block for evidence id, reopen command, expected rows, actual rows, WAL/file-format assertion, and successful exit status.
- Added a narrow `PDB_CRASH_AFTER_WAL_REPLAY_APPLIES` replay-interruption hook in `src/storage.rs` for deterministic crash-matrix injection.
- Repaired CM-005 to start from seed-only durable state, write committed WAL frames for rows 2 and 3, interrupt after the first replay apply, and verify two subsequent reopens.
- Strengthened `scripts/verify_crash_matrix` to validate exact per-case expected rows, actual rows, evidence ids, reopen command, WAL/file-format assertion, exit status, CM-004 repeated-open evidence, CM-005 interruption evidence, current-run provenance, and a negative validator self-check.
- No user-facing CLI output, stderr, command, or exit-code changes were introduced; `docs/cli_contract.md` remains unchanged.
- `docs/file_format.md` already contains the required WAL sidecar compatibility note covering absent sidecars, incomplete trailing frame cleanup, committed prefix replay, idempotence, and complete corrupt frame errors.

## Verification Evidence

- `cargo test --test crash_matrix`: passed, 7 tests.
- `./scripts/verify_crash_matrix`: passed and regenerated `target/crash_matrix/crash_matrix_report.md`.
- `./scripts/verify`: passed, including fmt, clippy, full test suite, and help smoke.
- `cargo test --test wal_recovery`: passed, 5 tests.
- `cargo test --test cli_contract`: passed, 5 tests.

## Remaining Implementation Work

None. Implementation is verifier-ready.
