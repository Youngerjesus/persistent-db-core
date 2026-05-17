# AGENTS (persistent-db-core)

Primary audience: coding agents and maintainers working inside this product repo.

## Product Direction

Build a small, deterministic Rust CLI database binary named `db`. V1 should grow toward durable single-process database behavior while keeping the documented CLI contract stable.

## Engineering Rules

- Keep changes scoped to the active task and its spec package.
- Prefer deterministic behavior, deterministic tests, and explicit persisted-data fixtures over implicit state.
- Treat persisted data compatibility, CLI output, exit codes, and documented error behavior as stable contracts once introduced.
- Make failure modes explicit. Avoid panics for user-facing CLI errors and persisted-data handling unless the invariant violation is unrecoverable programmer error.
- Do not broaden dependencies without a task-level reason. For V1, standard library implementations are preferred unless the spec explicitly calls for a crate.
- Do not add network services, background daemons, or remote-service requirements for V1.
- Keep CLI output stable once documented in `docs/cli_contract.md` or covered by integration tests.

## SDD Workflow

- Start each implementation from the task-provided `spec.md` and `contracts.md`.
- Treat `contracts.md` as the completion contract: every listed behavior, non-goal, required check, and acceptance artifact must be satisfied or explicitly blocked.
- Use repo-local `specs/**` as product history, templates, or local execution artifacts unless the active task explicitly selects one as the current spec.
- Before editing, identify the affected product contract: CLI behavior, persisted data compatibility, documented errors, tests, or durable docs.
- Preserve existing public behavior unless the active spec explicitly changes it.
- Add or update focused tests with behavior changes, including negative and edge cases for persisted data, recovery, indexing, and CLI contract work.
- Update durable docs only when the user-facing or compatibility contract changes.
- If the spec, contract, and repo reality conflict, stop and report the conflict instead of silently changing scope.
- Treat SDD pipeline `result_*.md` files as phase status reports. Their status line or `PM_RESULT:` sentinel expresses next owner and readiness; it is not a substitute for contract evidence.
- Treat latest review/report files as verifier or reviewer SSOT: `qa_prep_review.md`, `impl_review.md`, `impl_brake_review.md`, `code_review.md`, and `final_review.md`.
- Execution and repair phases may read the latest review/report files as input, but must not check off, delete, or overwrite reviewer findings unless the phase explicitly owns that review/report.
- On retry or repair re-entry, read the latest review/report first and repair only the actionable open items such as `Repair Targets`, `Must Fix Now`, `Next Action`, or `Verify Risks`.
- Preserve previous review/report contents in the matching `.history.md` file when the owning verifier or reviewer refreshes the latest SSOT; the latest file should contain only current open findings and verdict context.
- Use `development_state.md` only as high-density implementation state between passes. It supports handoff, but completion still depends on the active contract, latest review/report state, and required verification evidence.
- Task completion is blocked until `scripts/verify` and any contract-required checks pass.

## Architecture Boundary

Current structure is intentionally small:

- `src/main.rs`: CLI entrypoint and current implementation surface.
- `tests/`: integration and behavior tests; prefer black-box CLI behavior here.
- `docs/`: durable product documentation and contract references.
- `docs/history_archives/history.md`: append-only product history.
- `work_queue/progress.md`: current product progress view.
- `specs/`: product history or local execution artifacts from completed work.

No deeper module boundary map exists yet. When meaningful layers or modules are introduced, document the current boundary map here and enforce it through `scripts/verify`.

When a boundary map is introduced, dependencies must be one-way: higher-level or more volatile modules may depend on lower-level or more stable modules, but lower-level modules must not import back upward. Cross-boundary calls should go through the owning boundary's public interface, not through private implementation details. Cycles are architecture violations and must be caught by `scripts/verify` once boundaries exist.

## Verification Scripts

- `scripts/verify` is the baseline local verification entrypoint and must work from any caller cwd, including absolute-path invocation from outside the repo.
- It runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`.
- It must fail on missing tools and failed checks; do not silently skip required verification.
- Future `contracts.md` and task `verification.commands` entries must use `scripts/verify` as the baseline evidence command. Add task-specific commands only for narrower or deeper evidence, such as `cargo test --test page_storage`.
- DB-specific deeper verification, such as crash/restart, property, differential, or benchmark checks, should live in additional `scripts/verify*` scripts when too slow or specialized for the baseline.
- Any `scripts/verify*` or contract-required verification failure blocks completion.

## Environment And Secrets

- This repo currently requires no secrets for baseline development or `scripts/verify`.
- Keep real `.env` and `.env.*` files uncommitted.
- Track only placeholder examples such as `.env.example`.
- Do not copy external automation runtime state, secrets, logs, generated task state, or machine-specific paths into this repo.
- Secret-dependent or external checks must be separate `scripts/verify*` scripts with explicit env requirements.

## Document Map

- `docs/cli_contract.md`: current documented CLI behavior and smoke contract.
- `docs/v1_spec.md`: durable V1 product direction.
- `docs/history_archives/history.md`: append-only product history.
- `work_queue/progress.md`: current product progress summary.
- `.codex/`: repo-local coding agents, skills, and operating context installed from the managed repo bootstrap template.
- `.specify/`: spec-kit templates and scripts for local spec/plan/task workflows.
- `specs/**`: product history, spec templates, or local execution artifacts; use a spec here only when the active task selects it.
- Task-provided `spec.md`: current implementation requirements.
- Task-provided `contracts.md`: current acceptance contract and required verification.
- `tests/`: executable behavior coverage for the CLI and future database behavior.

## AGENTS.md Update Policy

Update this file when durable repo-level product direction, engineering rules, architecture boundaries, verification commands, env policy, document locations, or dependency policy changes. Do not put temporary task notes, secrets, local machine paths, scheduler state, generated logs, or runtime output in this file.
