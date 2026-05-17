# Implementation Verification Review: v1-deterministic-crash-matrix

## Verdict: PASS

PM_RESULT: success

## Scope

Strict implementation verification for `task-2026-05-18-02-23-10-v1-deterministic-crash-matrix`.

Reviewed implementation delta and required artifacts:
- `src/storage.rs`
- `tests/crash_matrix.rs`
- `tests/fixtures/crash_matrix/README.md`
- `scripts/verify_crash_matrix`
- `docs/file_format.md`
- `target/crash_matrix/crash_matrix_report.md`

Protected areas `ssot/` and `policies/` were not modified.

## Executed Checks

- `git status --short`
- `git rev-parse HEAD`
- `git rev-parse main`
- `git log --oneline main..HEAD`
- `git diff -- src/storage.rs docs/file_format.md docs/cli_contract.md scripts/verify`
- `cargo test --test crash_matrix` - PASS, 7 tests
- `./scripts/verify_crash_matrix` - PASS, regenerated and validated `target/crash_matrix/crash_matrix_report.md`
- `./scripts/verify` - PASS, including `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help`
- `test -f tests/crash_matrix.rs`
- `test -f tests/fixtures/crash_matrix/README.md`
- `test -x scripts/verify_crash_matrix`
- `test -f docs/file_format.md`
- `test -f target/crash_matrix/crash_matrix_report.md`

## Evidence

- `tests/crash_matrix.rs` implements CM-001 through CM-006 with case-specific assertions and failure messages containing `case_id` and crash point.
- `tests/crash_matrix.rs` records per-case observed stdout rows and exit status into `target/crash_matrix/crash_matrix_report.md`.
- `scripts/verify_crash_matrix` exports a current `CRASH_MATRIX_RUN_ID`, runs `cargo test --test crash_matrix`, validates every case block for evidence id, reopen command, expected rows, actual rows, WAL/file-format assertion, and exit status, and includes a negative validator self-check.
- `target/crash_matrix/crash_matrix_report.md` generated in this verification pass has run id `verify-crash-matrix-20260517T181412Z-78170` and contains CM-001 through CM-006.
- `docs/file_format.md` lines 99-114 document retained complete WAL frame idempotence, incomplete trailing frame truncation, committed prefix replay, and complete corrupt frame error behavior.
- `docs/cli_contract.md` has no diff; `./scripts/verify` ran `tests/cli_contract.rs` successfully, so no user-facing CLI output/error contract change was observed.
- Existing WAL regression coverage remains present and passing under `tests/wal_recovery.rs` via `./scripts/verify`.

## Primary Success Claims

1. The implementation covers all required deterministic crash matrix rows CM-001 through CM-006 across pre-WAL append, partial WAL frame, no commit marker, committed WAL before data apply, interrupted recovery, and corrupt tail after committed prefix.
2. Recovery behavior satisfies the contract: uncommitted/pre-commit rows remain invisible, committed rows become visible after reopen, repeated replay is idempotent, and interrupted replay re-entry applies committed frames exactly once.
3. Required verification and evidence artifacts exist and are current-run validated without changing the public CLI contract.

## Evidence Used

- `cargo test --test crash_matrix` passed all 7 tests, including CM-001 through CM-006 and the inventory/evidence id test.
- `./scripts/verify_crash_matrix` passed after regenerating `target/crash_matrix/crash_matrix_report.md` with current run id `verify-crash-matrix-20260517T181412Z-78170`.
- `./scripts/verify` passed the baseline suite, including full tests, clippy with `-D warnings`, format check, and `db --help` smoke.
- Runtime report observations show exact expected and actual rows for every case, CM-004 first/second reopen status `Some(0)`, and CM-005 interrupted reopen status `Some(101)` followed by two successful recovery reopens.
- File inspection confirmed the crash hook is gated by `PDB_CRASH_AFTER_WAL_REPLAY_APPLIES` in `src/storage.rs` lines 255-264 and is only called after WAL replay applies a frame in lines 230-233.

## Proxy Gap / Reward-Hacking Risk

- Because the matrix test harness also writes the markdown report, a false pass could occur if the report were a static success template or stale artifact rather than observed runtime output from the same run.
- Because the task modifies verification tooling and fixtures, a false pass could occur if `scripts/verify_crash_matrix` only checked labels or global substrings instead of exact per-case rows, status, and assertions.
- The CM-005 crash injection hook is production code, so a false pass could hide a public CLI behavior change if normal runs could trigger the hook without explicit test-only environment configuration.

## Gap-Closing Check

- `scripts/verify_crash_matrix` lines 45-82 validate the current run id, command provenance, every required case block, exact evidence id, exact reopen command, exact expected rows, exact actual rows, WAL/file-format assertion text, and exit status.
- `scripts/verify_crash_matrix` lines 84-104 validate CM-004 repeated reopen evidence and CM-005 interruption/retry evidence, including `interrupted reopen exit status after first replay apply: Some(101)`.
- `scripts/verify_crash_matrix` lines 112-119 mutate CM-005 row evidence and require validation to fail, proving the validator is not only checking broad labels.
- `target/crash_matrix/crash_matrix_report.md` from run id `verify-crash-matrix-20260517T181412Z-78170` records actual visible rows and status for CM-001 through CM-006.
- `./scripts/verify` passed `tests/cli_contract.rs` and `cargo run --bin db -- --help`; `docs/cli_contract.md` has no diff, closing the normal CLI surface-change risk.

## Open Findings

None.

## Repair Targets

None.

## Next Action

Proceed to the next scheduler phase.

## Updated At

2026-05-17T18:14:31Z
