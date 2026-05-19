# Design: v1-section14-benchmark-acceptance

## Overview
The implementation introduces a public `db bench` route backed by a narrow Section 14 benchmark module. The module owns deterministic workload generation, measurement, evidence JSON creation, and hard-fail validation. The verification script remains the task-specific acceptance command, but it must call the public CLI instead of running an internal benchmark path.

## Component Boundaries
| Component | Responsibility | Files |
|---|---|---|
| CLI dispatch | Parse `db bench`, print exact sentinels, map pass/fail to exit codes, expose help text | `src/main.rs`, `docs/cli_contract.md`, `tests/cli_contract.rs` |
| SQL type alias | Accept `INTEGER` as a narrow alias for existing `INT` in `CREATE TABLE` so the benchmark uses the frozen schema through public SQL | `src/sql.rs`, `tests/sql_exec.rs`, `docs/sql_subset.md`, `docs/cli_contract.md` |
| Benchmark engine | Build deterministic fixture, run measurements, validate thresholds, produce evidence JSON | `src/bench.rs`, `src/lib.rs` |
| SQL/index proof | Use existing SQL execution and query path proof for indexed paths; explicit harness scan for comparison | `src/sql.rs` API reuse, `src/bench.rs` |
| Verification script | Execute public `db bench`, validate evidence schema and thresholds, emit `BENCH_ACCEPTANCE` sentinel | `scripts/verify_bench_acceptance` |
| Documentation | Trace requirement IDs to commands, evidence file, formulas, hard-fail policy, and regression/bug state | `docs/cli_contract.md`, `docs/benchmarks.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md` |

## CLI Contract
- Supported command list gains `db bench`.
- `db bench` takes no path argument for this fixed Section 14 evidence run.
- Success stdout is exactly `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json\n`.
- Failure stdout starts with `DB_BENCH: FAIL check=<check_id> reason=<reason>\n`.
- Success exits `0`; failure exits non-zero.
- `open <path>` remains reserved; `bench <path>` is no longer documented as reserved.

## SQL Fixture Compatibility
- The benchmark must create the table with the contracted SQL spelling: `CREATE TABLE bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT);`.
- Implementation adds only a `CREATE TABLE` parser alias where `INTEGER` maps to the existing integer column type used for `INT`.
- Tests must prove `INTEGER PRIMARY KEY` supports the same insert/select/primary-key lookup behavior as `INT PRIMARY KEY`.
- Docs must state that `INTEGER` is accepted as an alias for `INT`; no other SQLite type aliases or affinity behavior are introduced.

## Evidence Schema
Top-level fields must include:

```text
schema_version
artifact
row_count
primary_key_type
secondary_index_column
secondary_index_name
text_bytes_min
text_bytes_max
deterministic_seed
warmup_runs
measurement_runs
runtime_cap_seconds
commands
workload
metrics
recovery
index_use_evidence
hard_fail_checks
result
```

The evidence path is fixed to `target/bench_acceptance/section14-benchmark-acceptance.json`.

## Evidence Command Lifecycle
1. `db bench` runs the benchmark, validates benchmark-local hard-fail checks, writes evidence to the fixed path, and records:
   - `commands.db_bench.status="pass"`
   - `commands.db_bench.evidence_path="target/bench_acceptance/section14-benchmark-acceptance.json"`
   - `commands.db_bench.stdout_sentinel="DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json"`
   - `commands.verify_bench_acceptance.status="pending"`
   - `result="pending_verifier"`
2. `scripts/verify_bench_acceptance` runs public `db bench`, validates the file, then atomically rewrites the same file to add:
   - `commands.verify_bench_acceptance.status="pass"`
   - `commands.verify_bench_acceptance.evidence_path="target/bench_acceptance/section14-benchmark-acceptance.json"`
   - `commands.verify_bench_acceptance.stdout_sentinel="BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json"`
   - `result="pass"`
3. The script revalidates the updated file before printing `BENCH_ACCEPTANCE: PASS ...`.
4. Any failed validation prints `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` and must not write final pass state.

## Workload Constants
- Table: `bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)`.
- Secondary index: `idx_section14_bench_group_key` on `group_key`.
- Row count: `100000`.
- Seed: `140014`.
- Payload length: `8 + ((id * 17 + deterministic_seed) % 57)`.
- `group_key`: `((id * 7919 + deterministic_seed) % 10000) + 1`.
- Warmup runs: `1`.
- Measurement runs: `5`.
- Measured query count: `400` per run, `2000` total excluding warmup.

## Measurement Flow
1. Create a fresh benchmark DB under `target/bench_acceptance/tmp/`.
2. Generate and execute sequential insert workload for 100000 rows and the secondary index.
3. Run one warmup query set.
4. Run five measured query sets with fixed primary, equality, equality scan, range, and range scan query definitions.
5. Compute medians for indexed and scan timings.
6. Compute:
   - `equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms`
   - `range_index_speedup = range_scan_median_ms / range_indexed_median_ms`
7. Collect `db_file_bytes` and `wal_file_bytes`.
8. Run separate 10000 committed transaction recovery scenario and verify proportionality bound.
9. Validate all hard-fail checks before writing a pass result.

## Indexed Path Proof
- For each representative equality/range predicate, record `query_kind`, `predicate`, `expected_access_path`, `observed_access_path`, `used_index`, `scan_rejected`, `indexed_result_count`, `scan_result_count`, and `result_hash_match`.
- `observed_access_path` must be `secondary_index_equality` or `secondary_index_range` for eligible indexed paths.
- Explicit scan comparison may only be used as benchmark harness comparison evidence; public indexed SQL path must not full-scan.

## Hard-Fail Checks
Required checks include:
- `dataset_contract`
- `required_fields`
- `positive_metrics`
- `runtime_cap`
- `equality_speedup`
- `range_speedup`
- `indexed_equality_no_full_scan`
- `indexed_range_no_full_scan`
- `scan_hash_match`
- `recovery_row_count`
- `recovery_proportionality`
- `no_retry_required`

Every check must be recorded in `hard_fail_checks` with `status=pass` on success. Any failure returns the exact CLI/script failure sentinel.

## Hard-Fail Negative Regression Design
- `src/bench.rs` exposes a narrow validator function for test construction, such as `validate_index_use_evidence_for_test` or a small evidence validation API.
- `tests/bench_acceptance.rs` constructs invalid eligible indexed equality evidence with `observed_access_path="full_scan"` and asserts failure `check_id="indexed_equality_no_full_scan"`.
- `tests/bench_acceptance.rs` constructs invalid eligible indexed range evidence with `observed_access_path="full_scan"` and asserts failure `check_id="indexed_range_no_full_scan"`.
- These tests prove the hard-fail mechanism rejects bad evidence instead of only checking optimistic pass output.

## Documentation Design
Docs must be updated after code behavior is in place so they do not overclaim. `docs/performance_report.md` should summarize the regenerated evidence and formulas. `docs/v1_acceptance.md` should map current targeted requirement IDs only. `docs/bug_diary.md` should either record concrete bugs found during implementation or a no-bug rationale plus regression tests.
