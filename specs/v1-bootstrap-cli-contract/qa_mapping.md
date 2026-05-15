# QA Mapping: V1 `db` CLI Contract And Smoke Baseline

## QA Prep Context
- Task ID: `task-2026-05-15-16-06-54-v1-bootstrap-cli-contract`
- Feature slug: `v1-bootstrap-cli-contract`
- Artifact gate: `gate-v1-cli-smoke`
- Requirement IDs: `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`
- Current run ID: `qa_prep_retry_1_resume_20260515_162224_982382_b4f683d2`
- Task source: `specs/v1-bootstrap-cli-contract/tasks.md`, which declares one task entry: `task-2026-05-15-16-06-54-v1-bootstrap-cli-contract`.
- Required skill note: `.codex/skills/qa-prep/SKILL.md` was requested for this phase, but no repo-local or parent-path `qa-prep` skill file was present in this worktree. This artifact follows the requested workflow order directly: Scenario expansion lens -> QA generation lens -> Testing-review lens.

## Scenario Expansion Lens
| Scenario ID | Path | Coverage Pressure | Expected Contract |
| --- | --- | --- | --- |
| `CLI-SC-001` | `db --help` | Happy path, help route | Exit `0`; stderr empty; stdout contains required help core lines in order. |
| `CLI-SC-002` | `db help` | Alias path, duplicate command surface | Exit `0`; stderr empty; stdout exactly matches `db --help`. |
| `CLI-SC-003` | `db --unknown` | Invalid input | Exit `2`; stdout empty; stderr uses required unsupported two-line format with token `--unknown`. |
| `CLI-SC-004` | `db open demo.db` | Reserved future command and partial state prevention | Exit `2`; stdout empty; stderr uses required unsupported two-line format with token `open`; no database file behavior starts. |
| `CLI-SC-005` | `open`, `exec`, `check`, `bench` command family | Duplicate/already-done and scope creep guard | Reserved names remain documented as future commands and are not executable in this slice. |
| `CLI-SC-006` | Empty invocation, `-h`, mixed args | Boundary surface | Not accepted as a supported command unless implementation explicitly documents a compatible unsupported behavior; contract only supports exact `--help` and `help`. |
| `CLI-SC-007` | Permission/trust boundary | Trust boundary guard | No network service, daemon, remote dependency, multi-process behavior, distributed storage, storage pages, SQL execution, indexes, transactions, WAL, or recovery behavior is introduced. |
| `CLI-SC-008` | Retry/re-entry | Deterministic rerun | Repeated test and smoke command runs must produce the same exit codes and stdout/stderr summaries. |
| `CLI-SC-009` | Dependency failure | Build/test toolchain | If `cargo test` or `cargo run` cannot execute because the Rust toolchain is unavailable, record the blocker explicitly rather than weakening assertions. |

## QA Generation Lens
| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `task-2026-05-15-16-06-54-v1-bootstrap-cli-contract` | `qa-prep-retry-ready` | Integration tests for binary dispatch; smoke commands for observable CLI behavior; manual doc review for `docs/cli_contract.md`; scope review against protected areas and V1 non-goals; task list coverage review against `specs/v1-bootstrap-cli-contract/tasks.md`. | `tests/cli_contract.rs` | `cargo test`; `cargo run --bin db -- --help`; `cargo run --bin db -- help`; `cargo run --bin db -- --unknown`; `cargo run --bin db -- open demo.db` | Green only when integration tests pass, each smoke command has the required exit/stdout/stderr behavior, `docs/cli_contract.md` documents current support, exit codes, unsupported format, future reservations, and non-goals, implementation changes remain limited to the approved CLI contract/smoke baseline scope, and every task ID in `tasks.md` is represented in this table with explicit commands and green criteria. | This QA prep retry intentionally leaves current implementation red. The scaffold locks the approved contract and should fail until implementation updates `src/main.rs` and adds the required docs. |

## Test Scaffold
- `tests/cli_contract.rs` defines integration coverage for `CLI-SC-001` through `CLI-SC-004`.
- The scaffold uses Cargo's `CARGO_BIN_EXE_db` binary path and no new dependencies.
- Help assertions require the specified core lines in order while allowing implementation-only whitespace around the contract.
- Alias assertions require `db help` stdout to equal `db --help` stdout.
- Unsupported assertions require exit code `2`, empty stdout, and the exact required stderr format.

## Provenance Contract
- Evidence root: current run directory `/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/autopilot/project_manager/tasks/task-2026-05-15-16-06-54-v1-bootstrap-cli-contract/runs/qa_prep_retry_1_resume_20260515_162224_982382_b4f683d2/`.
- Required artifact list: `result.md`; task source at `specs/v1-bootstrap-cli-contract/tasks.md`; QA mapping at `specs/v1-bootstrap-cli-contract/qa_mapping.md`; QA prep review at `specs/v1-bootstrap-cli-contract/qa_prep_review.md`; red test scaffold at `tests/cli_contract.rs`; command evidence summarized in the scheduler final response; implementation-phase evidence later must include `cargo test`, `cargo run --bin db -- --help`, `cargo run --bin db -- help`, `cargo run --bin db -- --unknown`, and `cargo run --bin db -- open demo.db`.
- Scenario/evidence IDs: `CLI-SC-001` through `CLI-SC-009`; `EV-CARGO-TEST-RED`; `EV-HELP-FLAG-RED`; `EV-HELP-ALIAS-RED`; `EV-UNSUPPORTED-UNKNOWN-RED`; `EV-RESERVED-OPEN-RED`.
- Current-run ID source: task metadata `active_run_id=qa_prep_retry_1_resume_20260515_162224_982382_b4f683d2`.
- Clean generation rule: canonical launch or command evidence for a fresh repair or verification pass is deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: implementation and verification phases must not reuse prior run command output, screenshots, exported reports, or cached evaluator output as proof for this run.
- Writer/validator separation expectation: the implementation worker may write CLI/docs/tests; verifier must independently rerun the preferred commands and inspect generated artifacts before accepting the gate.
- Redaction target list: no secrets are expected; still redact absolute home-directory-sensitive tokens, environment variable values, credentials, API keys, auth tokens, and any database fixture contents if future commands accidentally emit them.

## Testing-Review Lens
- Task ID coverage: `specs/v1-bootstrap-cli-contract/tasks.md` declares one task ID, and that task ID is mapped to every required acceptance item and preferred command in the QA Generation Lens table.
- Requirement coverage: `req-v1-cli-help-smoke` is covered by `CLI-SC-001`, `CLI-SC-002`, `EV-HELP-FLAG-RED`, and `EV-HELP-ALIAS-RED`; `req-v1-cli-dispatch-tests` is covered by `CLI-SC-003`, `CLI-SC-004`, `EV-UNSUPPORTED-UNKNOWN-RED`, and `EV-RESERVED-OPEN-RED`.
- Negative/boundary coverage: invalid token, reserved future command, empty/short-help/mixed command surface, scope creep, dependency failure, retry/re-entry, and trust-boundary paths are represented.
- Preferred command coverage: all required cargo test and smoke commands are listed and must be rerun after implementation.
- Current red expectation: `cargo test` should fail against the existing skeleton because help stdout and unsupported stderr do not match the approved contract.
