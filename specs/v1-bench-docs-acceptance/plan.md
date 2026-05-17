# Implementation Plan: v1-bench-docs-acceptance

## Scope
Implement the approved V1 benchmark lower-bound and acceptance documentation package without expanding the user-facing CLI.

## Authoritative Inputs
- `spec.md`
- `contracts.md`
- `/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/autopilot/ssot/current-artifact.md`
- Repo rules in `AGENTS.md`

## Target Delta
- Add `scripts/verify_bench_acceptance`.
- Add `docs/benchmarks.md`.
- Add `docs/v1_acceptance.md`.
- Minimally update `docs/v1_spec.md` or `docs/cli_contract.md` only if needed to clarify that benchmark evidence is script-local and `db bench` remains unsupported.
- Add focused tests only if needed to pin script contract or reserved CLI behavior.
- Produce runtime evidence at `target/bench_acceptance/v1-bench-docs-acceptance.json` during verification.

## Non-Goals
- No `db bench` implementation.
- No new public CLI command, output, or exit code.
- No networked, multi-process, or concurrency benchmark.
- No claims that V1 full 100k benchmark suite, secondary indexes, or arbitrary hardware performance is complete unless current evidence exists.
- No edits to protected `ssot/` or `policies/`.

## Implementation Sequence
1. Reconfirm current worktree state and relevant files.
2. Implement `scripts/verify_bench_acceptance` as an executable repo-local verification script.
3. Run the script once locally to generate `target/bench_acceptance/v1-bench-docs-acceptance.json`; fix only script defects, not threshold policy.
4. Add `docs/benchmarks.md` describing command, workload, schema, thresholds, output schema, measured lower-bound interpretation, environment assumptions, and non-guarantees.
5. Add `docs/v1_acceptance.md` with evidence id `evidence-v1-acceptance-docs` and all required launch gate rows.
6. Add minimal durable doc references only where required by consistency.
7. Run `scripts/verify` and `scripts/verify_bench_acceptance`.

## Verification Commands
- `scripts/verify`
- `scripts/verify_bench_acceptance`

## Acceptance Evidence To Preserve
- `evidence-v1-benchmark-lower-bounds`: command output plus `target/bench_acceptance/v1-bench-docs-acceptance.json` and `docs/benchmarks.md`.
- `evidence-v1-acceptance-docs`: `docs/v1_acceptance.md` mapping every required launch gate and requirement id to evidence, blocker, or out-of-scope reason.

## Risk Controls
- Benchmark variability: use intentionally low thresholds and minimum measured iteration, as required.
- Documentation overclaim: every doc claim must point to executable evidence or be marked incomplete/out of scope.
- CLI scope creep: do not touch command dispatch except to preserve/verify unsupported `bench`.
