# Implementation Verification Review: v1-transaction-wal-recovery

Verdict: PASS

## Scope

Verified the current worktree implementation for task `task-2026-05-17-23-45-17-v1-transaction-wal-recovery` against `spec.md`, `contracts.md`, `tasks.md`, `qa_mapping.md`, and the latest `impl_brake_review.md`.

Implementation verification was read-only for production code and tests. The only files written by this phase are this latest verification report and the current run result file.

`git log --oneline main..HEAD` and `git diff --stat main...HEAD` were empty because the implementation is present as uncommitted worktree changes. Verification therefore reviewed `git diff` plus untracked task artifacts: `tests/wal_recovery.rs` and `specs/v1-transaction-wal-recovery/**`.

## Executed Checks

- `git status --short`: showed scoped changes in `src/storage.rs`, `docs/file_format.md`, `docs/cli_contract.md`, new `tests/wal_recovery.rs`, and task-scoped `specs/v1-transaction-wal-recovery/**`.
- `git diff -- src/storage.rs docs/file_format.md docs/cli_contract.md`: reviewed WAL implementation and doc deltas.
- `cargo test --test wal_recovery`: passed, 3 tests.
- `cargo test`: passed, 53 integration tests plus empty unit/doc suites.
- `./scripts/verify`: passed, including format, clippy, tests, and `db --help` smoke.
- `git diff --check`: passed.
- CLI smoke through `cargo run --quiet --bin db -- exec <temp-db> ...`: create/insert exit `0`, stdout `""`, stderr `""`; reopen select exit `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`; retained WAL sidecar existed with size `202` bytes.
- Literal `cargo run --bin db -- exec ...` provenance check: command exit was `0` and db stdout was correct, but Cargo itself wrote wrapper status lines to stderr. This is not `db` process stderr, so CLI stderr evidence uses the quiet Cargo invocation and integration tests that execute `CARGO_BIN_EXE_db` directly.

## Evidence

- `tests/wal_recovery.rs` exists and covers Scenario A (`committed_wal_replay_survives_reopen_via_cli`), Scenario B incomplete-tail absence (`incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`), repeated reopen idempotence, retained WAL sidecar existence, and ahead-of-store deterministic corruption.
- `src/storage.rs` defines the frozen WAL constants, writes committed page-append frames before page append, replays sidecar frames on open, applies committed frames exactly once by `record_count_before`, skips rolled-back frames, ignores incomplete trailing frames, and errors on ahead-of-store committed frames.
- `docs/file_format.md` documents sidecar path `<database-path>.wal`, frame layout/framing, append-order replay, committed/rolled-back/incomplete handling, retained WAL/idempotence behavior, and existing database files without WAL opening normally.
- `docs/cli_contract.md` changed only to document successful mutation durability across later `db exec` starts and to remove stale WAL/recovery non-goal wording. It does not add commands or change stdout, stderr, or exit code contracts.
- Protected `ssot/` and `policies/` paths were not modified.

## Primary Success Claims

1. Complete committed WAL mutations are durable across reopen and visible through `db exec`; incomplete trailing WAL mutations are not exposed as durable rows.
2. WAL replay semantics and compatibility are documented with enough detail to preserve the file-format contract for this V1 slice.
3. The implementation satisfies the QA mapping and acceptance evidence commands without changing public CLI stdout, stderr, exit codes, or command grammar.

## Evidence Used

- `cargo test --test wal_recovery`: passed with `committed_wal_replay_survives_reopen_via_cli`, `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`, and `committed_wal_frame_ahead_of_page_store_fails_deterministically`.
- `cargo test`: passed all current CLI, storage, SQL, primary-index, and WAL integration tests.
- `./scripts/verify`: passed the repository baseline verification entrypoint.
- Runtime smoke with a fresh temp database: create/insert command produced exit `0`, empty stdout/stderr; reopen select produced exit `0`, empty stderr, and stdout exactly `id|name\n1|ada\n2|bea\n`; `<temp-db>.wal` existed and was `202` bytes.
- Manual artifact review of `tests/wal_recovery.rs`, `src/storage.rs`, `docs/file_format.md`, `docs/cli_contract.md`, `qa_mapping.md`, and `impl_brake_review.md`.

## Proxy Gap / Reward-Hacking Risk

- Scenario A could pass from normal page-file durability even if WAL replay were not actually applying frames.
- The task intentionally adds a test/fixture file, so green tests alone could be a false pass if the fixture layout diverged from production WAL framing.
- Literal `cargo run --bin db -- exec` emits Cargo wrapper lines on stderr, which could be mistaken for CLI stderr failure or hidden by imprecise evidence.
- A complete rollback-frame fixture is not directly tested, but the approved contract requires rollback or incomplete proof. Current Scenario B exercises the incomplete branch through CLI, while `src/storage.rs` and `docs/file_format.md` cover rollback skip semantics.

## Gap-Closing Check

- WAL replay was proven independently of page-file durability by `tests/wal_recovery.rs`: after CLI creates only the catalog, the test writes a committed WAL frame for `1|ada` plus a truncated committed ghost frame for `9|ghost`; two subsequent `SELECT * FROM users;` CLI runs return exactly `id|name\n1|ada\n`, proving replay of the committed fixture row, exclusion of the incomplete ghost, and retained-frame idempotence.
- Fixture/implementation layout alignment was checked against concrete artifacts: `tests/wal_recovery.rs` uses `PDBWAL1\0`, version `1`, header length `36`, committed state `0x01`, payload kind `0x01`, and the same checksum rule documented in `docs/file_format.md`; `src/storage.rs` defines and consumes the same WAL constants and checksum offsets.
- Actual CLI stream behavior was checked with `cargo run --quiet --bin db -- exec <temp-db> ...` and with integration tests using `CARGO_BIN_EXE_db`, avoiding Cargo wrapper stderr while still recording the literal wrapper behavior as provenance.
- The `docs/cli_contract.md` delta was reviewed as documentation of a new durability behavior, not a stdout/stderr/exit-code or command-surface change.

## Open Findings

None.

## Repair Targets

None.

## Next Action

Record `success` for the current implementation verification run and proceed to the next scheduler phase.

## Updated At

2026-05-18 00:36:50 KST
