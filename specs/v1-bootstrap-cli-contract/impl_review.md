# Implementation Verification Review: V1 `db` CLI Contract

## Verdict: PASS

The current worktree satisfies the approved V1 `db` CLI contract and smoke baseline. The product binary exposes only the approved help routes, rejects unsupported and reserved future commands with exit code `2`, includes deterministic automated dispatch tests, and documents the CLI surface and non-goals.

## Scope

- Verified implementation against `specs/v1-bootstrap-cli-contract/spec.md` and `specs/v1-bootstrap-cli-contract/contracts.md`.
- Read `specs/v1-bootstrap-cli-contract/qa_mapping.md`; the single declared task ID is represented with preferred commands and task-scoped green criteria.
- Reviewed `specs/v1-bootstrap-cli-contract/impl_brake_review.md`; the only carried risk is Cargo wrapper stderr around `cargo run`, not a product-binary stream mismatch.
- Inspected implementation artifacts: `src/main.rs`, `tests/cli_contract.rs`, and `docs/cli_contract.md`.
- Checked protected-area scope: no `ssot/` or `policies/` changes are present.
- Observed `git log --oneline main..HEAD` and `git diff --stat main...HEAD` are empty because implementation changes are currently uncommitted worktree changes; `git status --short --untracked-files=all` shows the task artifacts.

## Executed Checks

- `cargo test`: exit `0`; `tests/cli_contract.rs` ran 4 tests and all passed.
- `cargo run --bin db -- --help`: exit `0`; stdout includes the required help contract. Raw terminal stderr includes Cargo-owned build/run status lines.
- `cargo run --bin db -- help`: exit `0`; stdout includes the same help contract. Raw terminal stderr includes Cargo-owned build/run status lines.
- `cargo run --bin db -- --unknown`: exit `2`; binary reports the required unsupported-token error for `--unknown`. Raw terminal stderr includes Cargo-owned build/run status lines before the binary stderr.
- `cargo run --bin db -- open demo.db`: exit `2`; binary reports the required unsupported-token error for `open`. Raw terminal stderr includes Cargo-owned build/run status lines before the binary stderr.
- `cargo build --bin db`: exit `0`.
- Direct binary stream capture after build:
  - `target/debug/db --help`: exit `0`, stdout 499 bytes, stderr 0 bytes.
  - `target/debug/db help`: exit `0`, stdout 499 bytes, stderr 0 bytes, stdout matches `--help`.
  - `target/debug/db --unknown`: exit `2`, stdout 0 bytes, stderr exactly `error: unsupported argument or command: --unknown` and the required hint line.
  - `target/debug/db open demo.db`: exit `2`, stdout 0 bytes, stderr exactly `error: unsupported argument or command: open` and the required hint line.

## Evidence

- `src/main.rs:4-21` defines the required help stdout core lines in the approved order.
- `src/main.rs:26-39` dispatches only exact `--help` and `help`, rejects the first unsupported token, and exits `2` for unsupported input.
- `tests/cli_contract.rs:3-20` repeats the required help core lines; `tests/cli_contract.rs:37-95` verifies help output, help alias equality, unsupported `--unknown`, and reserved `open demo.db`.
- `docs/cli_contract.md:5-70` documents supported commands, help stdout, exit codes, unsupported stderr format, future command reservation, and non-goals.
- `git diff -- src/main.rs tests/cli_contract.rs docs/cli_contract.md Cargo.toml` shows product changes are limited to the CLI implementation plus required docs/tests; `Cargo.toml` has no observed delta.

## Primary Success Claims

1. The `db` binary now exposes the approved deterministic help contract for `db --help` and `db help`, with exit code `0`, empty binary stderr, and identical help stdout.
2. Unsupported arguments and reserved future commands remain non-executable in this slice, returning exit code `2`, empty binary stdout, and the required two-line stderr format using the first unsupported token.
3. The task includes durable smoke baseline artifacts: deterministic integration tests in `tests/cli_contract.rs` and CLI contract documentation in `docs/cli_contract.md`, without adding storage, SQL, WAL, networking, multi-process, or distributed behavior.

## Evidence Used

- Command evidence: `cargo test`, all four required `cargo run --bin db -- ...` commands, `cargo build --bin db`, and direct `target/debug/db ...` stream-capture checks.
- Artifact evidence: `src/main.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`, `specs/v1-bootstrap-cli-contract/qa_mapping.md`, and `specs/v1-bootstrap-cli-contract/tasks.md`.
- Runtime observations: direct binary stream capture showed zero-byte stderr for help, zero-byte stdout for unsupported paths, correct exit codes, and exact unsupported error payloads.
- Scope observations: protected directories `ssot/` and `policies/` have no modified files; no network service, daemon, remote dependency, storage, SQL, index, transaction, WAL, or recovery code is present in the changed product files.

## Proxy Gap / Reward-Hacking Risk

- The integration tests could falsely pass if they merely mirrored an incorrect implementation constant or if Cargo wrapper output were mistaken for product stderr.
- The presence of `docs/cli_contract.md` and `tests/cli_contract.rs` alone could falsely imply the reserved commands are non-executable without checking runtime dispatch.

## Gap-Closing Check

- Compared `src/main.rs:4-21` against the required help core lines and `src/main.rs:26-39` against the dispatch contract, then ran direct binary stream capture: `target/debug/db --help` and `target/debug/db help` both exited `0` with `stderr_bytes=0`, and `target/debug/db open demo.db` exited `2` with `stdout_bytes=0` and stderr token `open`.
- Ran exact required `cargo run --bin db -- --help`, `cargo run --bin db -- help`, `cargo run --bin db -- --unknown`, and `cargo run --bin db -- open demo.db`; process exit codes matched the contract. Cargo-owned status lines appeared on raw terminal stderr, so binary stream correctness was closed with integration tests using `CARGO_BIN_EXE_db` plus direct built-binary stream capture.

## Open Findings

None.

## Repair Targets

None.

## Next Action

Mark the implementation verification run successful.

## Updated At

2026-05-15T16:35:46+0900
