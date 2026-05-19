# QA Prep Verification Review

Verdict: PASS

## Scope

- Verified `qa_mapping.md` against `tasks.md` for `T1` through `T10`.
- Checked that `Preferred Commands` and `Task-Scoped Green` are concrete enough for implementation handoff.
- Rechecked negative and boundary coverage against `spec.md`, `contracts.md`, `plan.md`, and `design.md`.
- Refreshed representative red evidence for the current pre-implementation tree.

## Executed Checks

- `cargo test --test bench_acceptance_contract`
- `cargo test --test bench_acceptance`
- `cargo test --test cli_contract`
- `cargo test --test sql_exec integer_alias_primary_key_supports_section14_benchmark_schema`
- `scripts/verify_bench_acceptance`
- Static read of `qa_mapping.md`, `tasks.md`, `tests/bench_acceptance.rs`, `tests/bench_acceptance_contract.rs`, `tests/cli_contract.rs`, `tests/sql_exec.rs`, `plan.md`, and `design.md`

## Evidence

- `qa_mapping.md` maps all tasks `T1` through `T10`.
- Every task entry has concrete `Preferred Commands` and observable `Task-Scoped Green` criteria.
- Scenario coverage includes public CLI happy path, malformed `db bench <path>`, stale evidence regeneration, old 1k artifact rejection, missing fields, fixed workload constants, full-scan hard fail, below-threshold speedups, recovery proportionality failure, outside-cwd script invocation, writer/verifier separation, SQL `INTEGER` schema compatibility, and documentation traceability.
- `tests/bench_acceptance_contract.rs` now pins the Section 14 script/docs contract and rejects the obsolete 1k/current-doc wording.
- `tests/bench_acceptance.rs` now includes negative validator scaffolds for `indexed_equality_no_full_scan`, `indexed_range_no_full_scan`, `equality_speedup`, `range_speedup`, and `recovery_proportionality`.
- `tests/cli_contract.rs` now expects public `db bench` help and sentinel behavior while keeping `open <path>` reserved and `db bench <extra>` rejected.
- `tests/sql_exec.rs` includes the contracted `bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)` public SQL regression.

## Verified Red Evidence

- `cargo test --test bench_acceptance_contract` fails as expected because the current implementation script still does not invoke public `db bench` and `docs/benchmarks.md` is still old 1k wording.
- `cargo test --test bench_acceptance` fails to compile as expected because `persistent_db_core::bench` and its validator helpers do not exist yet.
- `cargo test --test cli_contract` fails as expected because current help omits `db bench`, still lists `bench <path>` as reserved, and bare `db bench` exits `2`.
- `cargo test --test sql_exec integer_alias_primary_key_supports_section14_benchmark_schema` fails as expected because `INTEGER` is currently rejected by the SQL parser.
- `scripts/verify_bench_acceptance` still exits `0` for the obsolete 1k evidence path, which is now caught by the Section 14 static contract test.

## Open Findings

- None for QA-prep handoff. The remaining failures are intentional red scaffolds for implementation.

## Next Action

- Proceed to implementation. The implementation pass should satisfy the mapped red tests and required commands without weakening the Section 14 contract.

## Updated At

- 2026-05-19T07:22:39Z
