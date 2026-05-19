## 2026-05-19T07:22:39Z - Archived QA prep verification review

# QA Prep Verification Review

Verdict: retry

## Must Fix Now

1. Replace the stale 1k benchmark contract test before implementation handoff.
   - Evidence: `tests/bench_acceptance_contract.rs:15-84` still pins `scripts/verify_bench_acceptance` to `cargo run --quiet --bin db -- exec`, requires `target/bench_acceptance/v1-bench-docs-acceptance.json`, and explicitly rejects `db bench`.
   - Conflict: `contracts.md` requires `scripts/verify_bench_acceptance` to invoke public `db bench` and emit `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`.
   - Required repair: update or replace `tests/bench_acceptance_contract.rs` during QA prep so it pins the Section 14 script/docs contract instead of the obsolete 1k contract. The mapping entry for T9 must name the repaired test as current QA coverage, not defer it with "should be revised during implementation".

2. Tighten threshold/bound QA coverage so T3's green condition is actually executable.
   - Evidence: `tests/bench_acceptance.rs:87-125` checks that metric names, recovery fields, and pass check rows appear as strings, but it does not verify numeric threshold values or proportionality bounds. `qa_mapping.md:53` and `tasks.md:28` say speedup and recovery fields must satisfy thresholds/bounds.
   - Required repair: add executable QA coverage that fails for below-threshold `equality_index_speedup`, below-threshold `range_index_speedup`, and recovery proportionality violations, either through benchmark validator fixture tests or script-level negative fixture checks with stable `check_id`s.

3. Make the script invocation proof enforce public `db bench`, not just the final sentinel.
   - Evidence: `tests/bench_acceptance.rs:128-162` accepts any script that exits 0, prints the verifier sentinel, and writes JSON containing pass strings. It does not detect an internal script-only benchmark path if the script fabricates the expected sentinel/file.
   - Required repair: add a static script contract test or dynamic proof that `scripts/verify_bench_acceptance` contains and runs `cargo run --bin db -- bench` and does not use the old `db exec` 1k path.

## Verified Red Evidence

- `cargo test --test cli_contract` is red for the expected public `db bench` gaps: help output still omits `db bench`, help still lists `bench <path>` as reserved, and bare `db bench` exits 2.
- `cargo test --test sql_exec integer_alias_primary_key_supports_section14_benchmark_schema` is red because `INTEGER` is currently rejected in the contracted schema.
- `cargo test --test bench_acceptance` is red at compile time because `persistent_db_core::bench` does not exist yet.
- `scripts/verify_bench_acceptance` still exits 0 for the old 1k artifact and prints the obsolete `target/bench_acceptance/v1-bench-docs-acceptance.json` output, confirming the stale script path remains.

## Retry Exit Criteria

- `qa_mapping.md` covers T1-T10 and continues to include concrete `Preferred Commands` and `Task-Scoped Green` entries.
- The stale 1k pinning test is repaired or replaced so `scripts/verify` will not carry a contradictory benchmark contract into implementation.
- Negative/boundary coverage includes invalid CLI shape, stale/old artifact rejection, full-scan hard fail, below-threshold speedups, recovery bound failure, outside-cwd script invocation, and documentation traceability.
- Red evidence is refreshed after the QA repairs and recorded in `qa_mapping.md`.
