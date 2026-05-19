# Implementation Plan: v1-section14-benchmark-acceptance

## Scope
Implement the frozen Section 14 benchmark acceptance contract for public `db bench`, 100000-row deterministic workload evidence, index-vs-scan hard-fail checks, WAL recovery proportionality evidence, verifier script, focused tests, and required documentation.

## Authoritative Inputs
- `spec.md`
- `contracts.md`
- Repo rules in `AGENTS.md`
- Current code context revalidated from `src/main.rs`, `src/lib.rs`, `src/sql.rs`, `scripts/verify`, existing `scripts/verify_bench_acceptance`, `tests/cli_contract.rs`, `docs/cli_contract.md`, `docs/benchmarks.md`, `docs/v1_acceptance.md`, and `docs/v1_spec.md`

## Target Delta
- Convert `bench` from reserved future command to supported public command in `src/main.rs`, help output, CLI docs, and CLI tests.
- Add narrowly scoped public SQL `INTEGER` type alias support so the contracted benchmark schema `bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)` can be created through the same SQL path used by `db bench`.
- Add `src/bench.rs` and expose it from `src/lib.rs`.
- Replace `scripts/verify_bench_acceptance` with a wrapper/verifier that invokes public `db bench`, validates evidence schema/thresholds/sentinels, and works from repo root or absolute path outside repo.
- Add or update focused benchmark acceptance tests, including hard-fail regression coverage for eligible indexed query full-scan rejection.
- Update `docs/cli_contract.md`, `docs/benchmarks.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, and `docs/bug_diary.md`.
- Generate runtime evidence at `target/bench_acceptance/section14-benchmark-acceptance.json` only during implementation verification, not in this planning phase.

## Non-Goals
- No changes to unrelated SQL semantics.
- No general SQLite type-affinity behavior beyond accepting `INTEGER` as an alias for existing `INT`.
- No storage format rewrite.
- No new network service, daemon, multi-process concurrency behavior, or distributed behavior.
- No edits to protected `ssot/` or `policies/`.
- No closing non-targeted V1 obligations through Section 14 evidence.

## Implementation Sequence
1. Reconfirm latest worktree state, canonical inputs, and existing `bench` reserved-command references.
2. Add red tests for public `db bench` help/CLI sentinel/evidence path and update the old `bench_reserved_future_command_remains_unsupported` expectation.
3. Add red SQL coverage for `INTEGER` in `CREATE TABLE` as an alias of the current integer type, with docs updates limited to `docs/sql_subset.md` and `docs/cli_contract.md`.
4. Add red tests for `scripts/verify_bench_acceptance` requiring public `db bench` and for Section 14 evidence fields.
5. Add negative hard-fail tests that feed validator evidence with eligible equality/range `observed_access_path="full_scan"` and assert stable failure check IDs.
6. Add `src/bench.rs` with deterministic fixture constants, payload generator, query set generators, median/throughput helpers, explicit scan evaluator, result hashing, recovery measurement, evidence validation, JSON emission, and post-verifier command-entry update support.
7. Wire `db bench` in `src/main.rs` with exact success/failure stdout sentinels and exit codes. `db bench` writes truthful pre-verifier evidence: `commands.db_bench.status="pass"`, `commands.verify_bench_acceptance.status="pending"`, and a non-final result state.
8. Replace `scripts/verify_bench_acceptance` so it resolves repo root, runs `cargo run --bin db -- bench`, validates the regenerated evidence file, atomically updates the same file with `commands.verify_bench_acceptance.status="pass"` and final `result="pass"`, revalidates the updated file, and emits exact `BENCH_ACCEPTANCE` sentinel.
9. Update required docs with requirement IDs, command evidence, threshold formulas, evidence path, hard-fail policy, `INTEGER` alias scope, and bug diary rationale/fixes.
10. Run `cargo fmt`, focused tests, `scripts/verify_bench_acceptance`, repo-outside absolute invocation of `scripts/verify_bench_acceptance`, and `scripts/verify`.

## Verification Commands
- `cargo test --test cli_contract`
- `cargo test --test bench_acceptance`
- `scripts/verify_bench_acceptance`
- From outside repo: `<repo>/scripts/verify_bench_acceptance`
- `scripts/verify`

## Acceptance Evidence To Preserve
- `target/bench_acceptance/section14-benchmark-acceptance.json`
- `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`
- `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`
- Final JSON after `scripts/verify_bench_acceptance` must contain both `commands.db_bench.status="pass"` and `commands.verify_bench_acceptance.status="pass"`. The intermediate JSON immediately after standalone `db bench` may truthfully record verifier status as `pending`.
- Requirement rows `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, `EVID-16-7` mapped in docs and final scheduler report.

## Risk Controls
- Benchmark runtime: keep `runtime_cap_seconds<=300`; fail instead of retrying when cap is exceeded.
- Performance flakiness: deterministic dataset/query sets, one warmup, five measured runs, median-based formulas, no dynamic key selection.
- Scope creep: do not add user-configurable benchmark modes unless required for tests; keep Section 14 constants fixed.
- Evidence drift: script validates the same file and sentinel that `db bench` writes.
- Command traceability overclaim: `db bench` must not mark verifier success before the verifier script has actually passed; the script owns the final verifier command entry update.
