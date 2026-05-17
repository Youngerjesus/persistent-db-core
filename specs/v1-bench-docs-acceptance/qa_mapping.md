# QA Mapping: v1-bench-docs-acceptance

## Scope

This QA prep artifact covers the approved `v1-bench-docs-acceptance` package only. It prepares verification contracts for benchmark lower-bound evidence and V1 acceptance documentation without implementing `scripts/verify_bench_acceptance`, adding user-facing `db bench`, or changing app/core database behavior.

## Evidence-Heavy Classification

Classification: evidence-heavy.

Reason: acceptance depends on a generated benchmark JSON artifact, launch-gate evidence mapping, command transcripts, and current-run provenance rather than only code or static checks.

## Provenance Contract

- Evidence root: `target/bench_acceptance/`.
- Required artifacts:
  - `target/bench_acceptance/v1-bench-docs-acceptance.json`
  - `docs/benchmarks.md`
  - `docs/v1_acceptance.md`
  - final report entries for `evidence-v1-benchmark-lower-bounds` and `evidence-v1-acceptance-docs`
- Scenario ids / evidence ids:
  - `bench_insert_1k`
  - `bench_reopen_select_1k`
  - `evidence-v1-benchmark-lower-bounds`
  - `evidence-v1-acceptance-docs`
- Current-run id source: the active scheduler run id `qa_prep_exec_fresh_20260518_061428_904371_9c063293`; implementation/verification may also include the script-generated UTC timestamp and current `git rev-parse HEAD` in the benchmark JSON.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass is deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: `target/bench_acceptance/v1-bench-docs-acceptance.json` from a prior run must not be copied forward or treated as current proof; rerun `scripts/verify_bench_acceptance` to produce the current artifact.
- Writer/validator separation expectation: implementation writes the script/docs/artifact; verifier independently runs `scripts/verify`, runs `scripts/verify_bench_acceptance` from repo root and from a non-root cwd, inspects the generated JSON schema and thresholds, and checks docs against the gate source.
- Redaction target list: no secrets are expected. Evidence must not include machine-specific temp database paths beyond the generated benchmark artifact path, external runtime state, secrets, `.env` values, or protected `ssot/` / `policies/` contents.

## Scenario Expansion Lens

- Happy path: script creates deterministic fresh temp databases, runs one warmup plus three measured iterations per scenario, emits JSON, and docs map every required launch gate.
- Invalid input / unsupported surface: `db bench` remains reserved and unsupported; benchmark script must not call it.
- Empty or partial state: missing script/docs/JSON must fail focused checks and must not be represented as completed evidence.
- Duplicate or already-done action: rerunning benchmark evidence replaces/regenerates current-run JSON rather than reusing old proof.
- Dependency failure / timeout: missing `cargo`, `rustc`, or failed `db exec` exits must cause `scripts/verify_bench_acceptance` to fail non-zero; benchmark iterations below threshold fail even if average/best passes.
- Permission / trust boundary: script must be executable, resolve repo root from its own path, and avoid protected `ssot/` / `policies/` edits.
- Retry / re-entry: repair passes must regenerate current evidence and keep historical artifacts only as audit evidence.

## Task Mapping

### T1. Revalidate Implementation Context

- Status: QA mapped.
- Verification Layers: repository state inspection; CLI contract inspection; existing reserved-command test.
- Test Files: `tests/cli_contract.rs`.
- Preferred Commands: `git status --short --branch`; `cargo test --test cli_contract bench_reserved_future_command_remains_unsupported`.
- Task-Scoped Green: current HEAD and dirty state are known; `db bench` remains unsupported with exit code `2`; no implementation context conflict is found.
- Notes: existing `bench_reserved_future_command_remains_unsupported` covers the public CLI non-expansion guard.

### T2. Add Benchmark Acceptance Script

- Status: red scaffold added.
- Verification Layers: static script contract test; operational benchmark command; JSON artifact validation by script and verifier.
- Test Files: `tests/bench_acceptance_contract.rs`; `scripts/verify_bench_acceptance`; `target/bench_acceptance/v1-bench-docs-acceptance.json`.
- Preferred Commands: `cargo test --test bench_acceptance_contract benchmark_acceptance_script_contract_is_pinned`; `scripts/verify_bench_acceptance`; `(cd /tmp && <repo>/scripts/verify_bench_acceptance)`.
- Task-Scoped Green: script exists, is executable, uses strict Bash, resolves repo root, uses `cargo run --quiet --bin db -- exec`, never calls `db bench`, implements `bench_insert_1k` and `bench_reopen_select_1k`, uses deterministic 1,000-row `bench_items(id INT, value TEXT)` data, runs one warmup and three measured fresh-DB iterations per scenario, validates stderr/stdout/exit codes, applies minimum-iteration thresholds, and emits required JSON fields.
- Notes: red scaffold intentionally fails before the script exists.

### T3. Add Benchmark Documentation

- Status: red scaffold added.
- Verification Layers: static documentation contract test; manual review for overclaiming.
- Test Files: `tests/bench_acceptance_contract.rs`; `docs/benchmarks.md`.
- Preferred Commands: `cargo test --test bench_acceptance_contract benchmark_documentation_contract_is_pinned`; manual review of `docs/benchmarks.md`.
- Task-Scoped Green: document names `scripts/verify_bench_acceptance`, artifact path, workload schema, row count, warmup/measurement policy, thresholds, JSON schema minimum, measured-minimum interpretation, environment assumptions, non-guarantees, and no public `db bench` availability.
- Notes: docs must describe lower bounds as repo-local acceptance floors, not arbitrary-hardware guarantees.

### T4. Add V1 Acceptance Guide

- Status: red scaffold added.
- Verification Layers: static acceptance guide test; manual review against handoff gate source.
- Test Files: `tests/bench_acceptance_contract.rs`; `docs/v1_acceptance.md`.
- Preferred Commands: `cargo test --test bench_acceptance_contract v1_acceptance_guide_maps_required_gates_to_evidence`; manual comparison with `/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/autopilot/ssot/current-artifact.md`.
- Task-Scoped Green: guide includes `evidence-v1-acceptance-docs`, names `autopilot/ssot/current-artifact.md`, includes every required gate/requirement pair, and gives each row an evidence path plus command/manual evidence and current status.
- Notes: rows without evidence must be marked `blocked_missing_evidence`, `out_of_scope_for_this_task`, or `pending_current_task_verification`; `req-v1-secondary-index-proof` must not be falsely completed.

### T5. Minimal Reference Updates

- Status: QA mapped.
- Verification Layers: doc diff review; CLI contract test.
- Test Files: `tests/cli_contract.rs`; optional `docs/v1_spec.md`; optional `docs/cli_contract.md`.
- Preferred Commands: `git diff -- docs/v1_spec.md docs/cli_contract.md`; `cargo test --test cli_contract bench_reserved_future_command_remains_unsupported`.
- Task-Scoped Green: public CLI surface remains unchanged; any durable doc update is limited to script-local benchmark evidence and preserves `bench <path>` as reserved future command.
- Notes: no update is required if new `docs/benchmarks.md` and `docs/v1_acceptance.md` make the boundary clear.

### T6. Focused Tests If Needed

- Status: red scaffold added.
- Verification Layers: focused integration contract tests only.
- Test Files: `tests/bench_acceptance_contract.rs`; existing `tests/cli_contract.rs`.
- Preferred Commands: `cargo test --test bench_acceptance_contract`; `cargo test --test cli_contract bench_reserved_future_command_remains_unsupported`.
- Task-Scoped Green: tests pin script/docs/evidence mapping and reserved CLI behavior without broad engine assertions or unrelated feature coverage.
- Notes: scaffold is intentionally static for script/docs contracts; operational benchmark correctness remains owned by `scripts/verify_bench_acceptance`.

### T7. Verification And Evidence Capture

- Status: QA mapped; red evidence pending implementation.
- Verification Layers: baseline verification; benchmark verification; generated artifact inspection; final report evidence linkage.
- Test Files: `scripts/verify`; `scripts/verify_bench_acceptance`; `target/bench_acceptance/v1-bench-docs-acceptance.json`; final report.
- Preferred Commands: `scripts/verify`; `scripts/verify_bench_acceptance`; inspect `target/bench_acceptance/v1-bench-docs-acceptance.json`.
- Task-Scoped Green: both required commands pass; benchmark JSON exists with required schema and passing threshold values; final report names `evidence-v1-benchmark-lower-bounds` and `evidence-v1-acceptance-docs`.
- Notes: implementation phase must generate current-run benchmark evidence rather than reusing historical output.

## Testing-Review Lens

- All task IDs T1-T7 are mapped.
- Preferred commands are concrete and runnable.
- Task-scoped green criteria are specific per task.
- Negative and boundary coverage is included for missing artifacts, reserved `db bench`, incomplete evidence statuses, benchmark variability, rerun/reuse, permission/executable bit, and dependency failures.
- Scaffold avoids application/core implementation changes and does not make the benchmark green.

## Red Evidence

- `cargo test --test bench_acceptance_contract` exits `101` as expected before implementation:
  - `benchmark_acceptance_script_contract_is_pinned` fails because `scripts/verify_bench_acceptance` is missing.
  - `benchmark_documentation_contract_is_pinned` fails because `docs/benchmarks.md` is missing.
  - `v1_acceptance_guide_maps_required_gates_to_evidence` fails because `docs/v1_acceptance.md` is missing.
- `scripts/verify_bench_acceptance` exits `127` as expected before implementation because the command does not exist yet.
- `cargo test --test cli_contract bench_reserved_future_command_remains_unsupported` exits `0`, confirming the existing public CLI still treats `db bench` as unsupported.
