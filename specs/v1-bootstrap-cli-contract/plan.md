# Plan: V1 `db` CLI Contract And Smoke Baseline

## Phase Boundary
- Current phase: plan execution.
- Do not edit production code, tests, final docs, SSOT, policies, or runtime state in this phase.
- Planning artifact only: this file.
- Requested `sdd-autopilot` skill is unavailable in the session skill list, so this plan follows the provided spec package, repo `AGENTS.md`, and observed repository state.

## Inputs Reviewed
- `AGENTS.md`
- `specs/v1-bootstrap-cli-contract/spec.md`
- `specs/v1-bootstrap-cli-contract/contracts.md`
- `specs/v1-bootstrap-cli-contract/review_loop/code_context.md`
- `specs/v1-bootstrap-cli-contract/review_loop/design.md`
- Current `Cargo.toml`
- Current `src/main.rs`

## Current Repository State
- HEAD verified: `f3fa75a95ba099d7145ab01175713b56664a25bb`.
- Worktree state at planning time: untracked `specs/` package inputs are present.
- Binary target already exists: `[[bin]] name = "db"`, path `src/main.rs`.
- Current CLI skeleton is not contract-compliant:
  - Empty args currently print help, but the approved contract only names `db --help` and `db help` as supported.
  - `-h` currently prints help, but `-h` is not part of the approved supported command surface.
  - Any arg list containing `--help` currently prints help, but dispatch should be deterministic against the approved command forms.
  - Unsupported stderr currently uses `db: unsupported arguments: ...` and `Run ...`, but the contract requires the exact two-line `error:` and `hint:` format with the first unsupported token.

## Implementation Boundary For Next Execution Step
- Edit `src/main.rs` only for the V1 CLI dispatch contract.
- Add `tests/cli_contract.rs` for deterministic command dispatch tests.
- Add `docs/cli_contract.md` documenting the observable CLI contract.
- Leave `Cargo.toml` unchanged unless execution discovers a test-only need that cannot be met with the standard library. The current package already defines the required `db` binary.
- Do not implement storage pages, SQL execution, indexes, transactions, WAL, recovery, network service, multi-process behavior, or distributed behavior.
- Do not modify `ssot/`, `policies/`, queue topology, or Autopilot runtime state.

## CLI Dispatch Plan
- Define a single `HELP` string in `src/main.rs` containing the required help output core lines in the required order.
- Parse `std::env::args().skip(1)` into a vector.
- Supported cases:
  - `["--help"]`: print `HELP` to stdout, write nothing to stderr, exit `0`.
  - `["help"]`: print exactly the same `HELP` to stdout, write nothing to stderr, exit `0`.
- Unsupported cases:
  - Any other arg list, including empty args, `["-h"]`, `["--unknown"]`, `["open", "demo.db"]`, and mixed forms.
  - Select the first unsupported token as `args.first()`. If the arg list is empty, use a stable token such as `<none>` unless execution/spec review decides empty invocation should be an additional documented behavior. This is the only ambiguity found in the approved package.
  - Print exactly:

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

  - Write unsupported output only to stderr, stdout empty, exit `2`.
- Reserved commands `open`, `exec`, `check`, and `bench` must remain unsupported in this slice and must not start future behavior.

## Automated Test Plan
- Add `tests/cli_contract.rs` as an integration test.
- Use the Cargo-provided binary path environment variable `CARGO_BIN_EXE_db` to run the compiled `db` binary without adding dependencies.
- Test cases:
  - `db --help` exits `0`, stderr is empty, stdout contains the required help lines in order.
  - `db help` exits `0`, stderr is empty, stdout equals `db --help` stdout.
  - `db --unknown` exits `2`, stdout is empty, stderr equals the required unsupported format with token `--unknown`.
  - `db open demo.db` exits `2`, stdout is empty, stderr equals the required unsupported format with token `open`.
- Keep assertions deterministic and compare normalized UTF-8 strings from `std::process::Command` output.

## Documentation Plan
- Add `docs/cli_contract.md`.
- Include:
  - Current supported scope: only `db --help` and `db help`.
  - Required help stdout core lines.
  - Exit codes: `0` for supported help forms, `2` for unsupported arguments or commands.
  - Unsupported stderr format and first-token selection.
  - Reserved future commands: `open <path>`, `exec <path> <sql>`, `check <path>`, `bench <path>`.
  - Non-goals: no storage, SQL, indexes, transactions, WAL, recovery behavior in this slice; no network server, multi-process concurrency, or distributed storage.

## Verification Strategy
- Required commands for execution evidence:
  - `cargo test`
  - `cargo run --bin db -- --help`
  - `cargo run --bin db -- help`
  - `cargo run --bin db -- --unknown`
  - `cargo run --bin db -- open demo.db`
- Final report evidence should connect each acceptance criterion to:
  - Test output from `cargo test`.
  - Exit code, stdout summary, and stderr summary from each smoke command.
  - Presence of `tests/cli_contract.rs`.
  - Presence of `docs/cli_contract.md`.

## Risks And Handling
- Ambiguous empty invocation: the approved contract does not define `db` with no args. Treat it as unsupported in execution because the contract lists only `--help` and `help` as supported. Record the chosen behavior in implementation notes or escalate only if verifier rejects it.
- `CARGO_BIN_EXE_db` availability: integration tests should use Cargo's standard binary test environment. If unavailable under the local toolchain, fall back to an integration test helper that locates `target/debug/db` only after confirming this does not add nondeterminism.
- Scope creep risk: reserved future commands are intentionally unsupported. Tests should lock `open demo.db` to exit `2` to prevent accidental implementation during this slice.

## Execution-Ready Checklist
- [ ] Update `src/main.rs` dispatch and exact output strings.
- [ ] Add deterministic integration tests in `tests/cli_contract.rs`.
- [ ] Add `docs/cli_contract.md`.
- [ ] Run all required verification commands and capture exit/stdout/stderr evidence.
- [ ] Final report maps evidence to each acceptance criterion.
