# Implementation Brake Review: V1 `db` CLI Contract

## Verdict: PASS

No open `verify-blocking` finding remains. This pass means the implementation is ready to enter independent `impl_verify`; it is not final acceptance proof.

Next action: proceed to `impl_verify`.

Updated At: 2026-05-15T16:32:44+0900

## Scope

- Reviewed approved spec and contract: `specs/v1-bootstrap-cli-contract/spec.md`, `specs/v1-bootstrap-cli-contract/contracts.md`.
- Reviewed QA mapping: `specs/v1-bootstrap-cli-contract/qa_mapping.md`.
- Reviewed current implementation artifacts: `src/main.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`.
- Reviewed latest implementation result: `autopilot/project_manager/tasks/task-2026-05-15-16-06-54-v1-bootstrap-cli-contract/runs/impl_exec_fresh_20260515_162655_220457_4f6e466d/result.md`.
- Reviewed current diff and worktree status. No protected `ssot/` or `policies/` changes were observed.
- Ran brake-readiness verification commands without repairing product code or tests.
- Companion review: requested named companion roles are not exposed in this runtime; used a read-only default companion fallback. The companion reported no verify-blocking or verify-risk findings, with the same residual Cargo wrapper caveat recorded below.

## Finding Checklist

- `BRK-RISK-001`
  - Status: `open`
  - Severity: `verify-risk`
  - Kind: `verification gap`
  - Risk category: `evidence provenance`
  - Source attempt: `impl_brake_exec_fresh_20260515_162925_448810_160c201d`
  - Evidence: `cargo run --bin db -- --help`, `cargo run --bin db -- help`, `cargo run --bin db -- --unknown`, and `cargo run --bin db -- open demo.db` executed during this brake pass return the binary exit codes and payloads expected by the contract, but Cargo wrapper/status lines are emitted around the binary output when using exact `cargo run` commands. The latest implementation result also notes this wrapper behavior.
  - Repair target: none for implementation brake; verifier should decide how to separate Cargo wrapper output from binary-level stdout/stderr, or use the built binary / Cargo integration-test binary evidence for stream-level assertions.
  - Disposition: `can defer`
  - Closure evidence: none; forward to `impl_verify`.

## Must Fix Now

None.

## Verify Risks

- `BRK-RISK-001`: Exact `cargo run` smoke commands include Cargo-owned wrapper/status lines in the terminal stream even when the `db` binary itself satisfies empty stderr/stdout requirements. `impl_verify` should explicitly judge whether acceptance evidence is based on binary-level streams, integration tests using `CARGO_BIN_EXE_db`, or raw `cargo run` terminal output. This is not `verify-blocking` because the implementation is still executable, the binary-level behavior is directly tested in `tests/cli_contract.rs`, and no product-code contradiction is visible.

## Blocked On Evidence

None.

## Blocked On Human Decision

None.

## Repair Targets

None.

## Closure Evidence

- `src/main.rs:4-21` defines the required help core lines in the approved order.
- `src/main.rs:26-39` dispatches only exact `--help` / `help`, rejects unsupported first tokens, and exits `2` for unsupported input.
- `tests/cli_contract.rs:37-95` covers help output, help alias equality, unsupported `--unknown`, and reserved `open demo.db`.
- `docs/cli_contract.md:5-70` documents current support, help stdout, exit codes, unsupported stderr format, reserved future commands, and non-goals.
- `cargo test`: exit `0`; integration suite `tests/cli_contract.rs` ran 4 tests and all passed.
- `cargo run --bin db -- --help`: exit `0`; output includes the required help contract lines.
- `cargo run --bin db -- help`: exit `0`; output includes the same help contract lines.
- `cargo run --bin db -- --unknown`: exit `2`; binary reports `error: unsupported argument or command: --unknown`.
- `cargo run --bin db -- open demo.db`: exit `2`; binary reports `error: unsupported argument or command: open`.
- Companion fallback review completed with no verify-blocking findings.

## Residual Risks

- Named companion roles `implementation-brake-reviewer` and `code-reviewer` were not available in the exposed runtime roles, so a read-only default companion was used as fallback.
- `impl_verify` should perform independent stream capture and final acceptance/provenance checks; this brake pass does not certify final task completion.

## Next Action

Proceed to strict independent `impl_verify`.

## Updated At

2026-05-15T16:32:44+0900
