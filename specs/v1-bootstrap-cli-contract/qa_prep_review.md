# QA Prep Retry Review

## Result
- Prep worker decision: `ready_for_qa_verification`
- Retry run: `qa_prep_retry_1_resume_20260515_162224_982382_b4f683d2`
- Latest verifier SSOT reviewed: `specs/v1-bootstrap-cli-contract/qa_prep_review.md` from verification run `qa_prep_verify_1_fresh_20260515_161954_770533_b862e003`.
- Required skill note: `.codex/skills/qa-prep/SKILL.md` was requested but was not present in this worktree or parent skill paths. This retry reapplies the requested lens order directly.

## Retry Checklist Closure
| Verifier Finding | Retry Action | Status |
| --- | --- | --- |
| `tasks.md` missing, preventing independent confirmation that every task is covered. | Added `specs/v1-bootstrap-cli-contract/tasks.md` with the single intended task entry: `task-2026-05-15-16-06-54-v1-bootstrap-cli-contract`. | Closed |
| `qa_mapping.md` should reference each `tasks.md` task entry with explicit preferred commands and task-scoped green criteria. | Updated `qa_mapping.md` to name `tasks.md` as task source, keep the single task ID in the QA Generation Lens table, and include explicit preferred commands and green criteria. | Closed |
| QA prep output should include a worker-generated review report, not only verifier-created review output. | Regenerated this file as the prep worker review artifact and retained the verifier checklist closure record. | Closed |
| Keep `tests/cli_contract.rs` aligned with spec and contract. | Rechecked scaffold; it covers `db --help`, `db help`, `db --unknown`, and `db open demo.db` exactly per approved contract. | Closed |
| Preserve red evidence until implementation changes `src/main.rs` and adds `docs/cli_contract.md`. | Reran required commands in this retry; current skeleton remains red as expected. | Closed |

## Scenario Expansion Lens
- Happy paths remain `CLI-SC-001` (`db --help`) and `CLI-SC-002` (`db help`), with exit `0`, empty stderr, and required help stdout ordering.
- Invalid and reserved command paths remain `CLI-SC-003` (`db --unknown`) and `CLI-SC-004` (`db open demo.db`), with exit `2`, empty stdout, and exact unsupported stderr format.
- Boundary pressure remains assigned to `CLI-SC-005` through `CLI-SC-009`: full reserved command family, empty invocation/`-h`/mixed args, trust boundary, retry/re-entry determinism, and dependency failure.
- No hidden additional task entries exist in the task package after adding `tasks.md`; the single declared task is the single row in the QA Generation Lens mapping.

## QA Generation Lens
- `tests/cli_contract.rs` remains the deterministic automated scaffold for required command dispatch behavior.
- Preferred commands remain:
  - `cargo test`
  - `cargo run --bin db -- --help`
  - `cargo run --bin db -- help`
  - `cargo run --bin db -- --unknown`
  - `cargo run --bin db -- open demo.db`
- Task-scoped green is unchanged: tests pass, smoke commands match exit/stdout/stderr contracts, `docs/cli_contract.md` documents the contract and non-goals, and implementation scope stays limited to the approved CLI slice.

## Testing-Review Lens
- Task coverage: `tasks.md` has one task ID and `qa_mapping.md` maps that same ID with explicit commands and green criteria.
- Required command coverage: all five preferred commands are represented in `qa_mapping.md` and were rerun in this retry.
- Negative/boundary coverage: invalid argument, reserved future command, empty/short-help/mixed surface, trust boundary, retry/re-entry, and dependency failure paths remain represented.
- Red evidence is expected and valid for QA prep: current implementation still prints old skeleton help and old unsupported stderr, so `cargo test` fails until implementation updates production code.

## Red Evidence Summary
- `cargo test`: exit `101`; integration scaffold ran 4 tests and all 4 failed against current skeleton behavior.
- `cargo run --bin db -- --help`: exit `0`; current output still starts with `db 0.1.0`, not the approved contract title.
- `cargo run --bin db -- help`: exit `0`; current output still matches old skeleton help.
- `cargo run --bin db -- --unknown`: exit `2`; current stderr still uses old `db: unsupported arguments: --unknown` format.
- `cargo run --bin db -- open demo.db`: exit `2`; current stderr still uses old `db: unsupported arguments: open demo.db` format instead of first token `open`.
