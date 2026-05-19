# Tasks: v1-section14-benchmark-acceptance

## T1. Revalidate Implementation Context [completed]
- Files: read-only.
- Steps:
  - Run `git status --short --branch` and `git rev-parse HEAD`.
  - Re-read `spec.md`, `contracts.md`, `src/main.rs`, `src/lib.rs`, `src/sql.rs`, `scripts/verify_bench_acceptance`, `docs/cli_contract.md`, `docs/benchmarks.md`, `docs/v1_acceptance.md`, and CLI/benchmark tests.
  - Confirm no protected `ssot/` or `policies/` edits are needed.
- Acceptance:
  - Implementation notes identify existing reserved `bench` references and older 1k benchmark script as replacement targets.

## T2. Add Red CLI Contract Coverage [completed]
- Files: `tests/cli_contract.rs`.
- Subtasks:
  - Update required help lines to include `db bench` under supported commands.
  - Remove `bench_reserved_future_command_remains_unsupported` expectation.
  - Add public `db bench` contract test for exit `0`, empty stderr, exact `DB_BENCH: PASS ...` stdout, and evidence file path.
  - Keep `open <path>` reserved unsupported.
- Acceptance:
  - Test fails before implementation because current CLI still rejects `bench`.

## T3. Add Red Benchmark Acceptance Coverage [completed]
- Files: `tests/bench_acceptance.rs`.
- Subtasks:
  - Verify evidence file is created at `target/bench_acceptance/section14-benchmark-acceptance.json`.
  - Verify required top-level fields exist.
  - Verify fixed dataset constants: row count, primary key type, secondary index column/name, text byte bounds, seed, warmup/measurement runs, runtime cap.
  - Verify speedup fields and recovery fields exist and satisfy thresholds/bounds.
  - Verify `index_use_evidence` proves secondary equality/range indexed path and scan result hash match.
  - Verify `hard_fail_checks` records pass for all required check ids.
  - Add negative validator regression where eligible equality evidence reports `observed_access_path="full_scan"` and assert failure `check_id="indexed_equality_no_full_scan"`.
  - Add negative validator regression where eligible range evidence reports `observed_access_path="full_scan"` and assert failure `check_id="indexed_range_no_full_scan"`.
- Acceptance:
  - Coverage is focused on Section 14 evidence contract, not broad SQL behavior.
  - FAIL-14-5 is proven by a failure-path test, not only by positive evidence.

## T4. Add Narrow `INTEGER` SQL Alias [completed]
- Files: `src/sql.rs`, `tests/sql_exec.rs`, `docs/sql_subset.md`, `docs/cli_contract.md`.
- Subtasks:
  - Add a parser-level alias so `INTEGER` maps to the existing integer column type currently represented by `INT`.
  - Add focused regression for `CREATE TABLE bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT);` followed by insert/select and primary-key lookup.
  - Document `INTEGER` as an accepted spelling alias for `INT`.
  - Do not add other SQL type aliases, affinity rules, or storage format changes.
- Acceptance:
  - Contracted benchmark schema can be created through public SQL.
  - Existing `INT` behavior and docs remain valid.

## T5. Implement Benchmark Module [completed]
- Files: `src/bench.rs`, `src/lib.rs`.
- Subtasks:
  - Add constants for artifact name, evidence path, table schema, index name, row count, seed, text byte bounds, warmup/measurement counts, runtime cap, and thresholds.
  - Implement deterministic payload generator and query set generators exactly from `contracts.md`.
  - Implement benchmark DB setup with sequential inserts and `CREATE INDEX idx_section14_bench_group_key ON bench_items(group_key);`.
  - Implement measured primary-key lookup, secondary equality indexed, secondary equality scan, range indexed, and range scan flows.
  - Implement explicit scan evaluator for scan comparison with deterministic result hash.
  - Reuse existing SQL/query-path proof for indexed path evidence.
  - Implement median, throughput, elapsed, file-size, and runtime cap tracking.
  - Implement Section 14 evidence JSON writer with std-only escaping.
  - Implement evidence command lifecycle support: initial `db bench` output records verifier status as `pending`; post-verifier update records verifier status as `pass` and final result as `pass`.
  - Implement hard-fail validation returning stable `check_id` and `reason`.
  - Expose a narrow test-only or public validation helper that lets `tests/bench_acceptance.rs` feed invalid `index_use_evidence` and assert hard-fail check IDs.
- Acceptance:
  - No new dependency is required.
  - The module does not expose configurable benchmark variants outside Section 14 needs.

## T6. Implement Recovery Evidence [completed]
- Files: `src/bench.rs`.
- Subtasks:
  - Create a separate recovery DB.
  - Execute 10000 committed transactions.
  - Record `wal_file_bytes` before reopen.
  - Measure close/reopen recovery time.
  - Verify `recovered_row_count=10000`.
  - Verify representative lookup returns the expected committed row.
  - Enforce `recovery_ms <= max(2000, wal_file_bytes / 4096)`.
- Acceptance:
  - Recovery hard-fail checks are included in `hard_fail_checks`.

## T7. Wire Public `db bench` [completed]
- Files: `src/main.rs`.
- Subtasks:
  - Add `db bench` to help usage and supported command list.
  - Route `[command] if command == "bench"` to `bench::run_section14_acceptance()`.
  - Print exact pass/fail stdout sentinels.
  - Exit `0` on pass and non-zero on fail.
  - Ensure malformed `db bench extra` remains unsupported with existing error behavior.
- Acceptance:
  - `db --help` and `db help` match.
  - `db bench` creates the evidence file and prints the required sentinel.
  - Standalone `db bench` does not claim `commands.verify_bench_acceptance.status="pass"` before the verifier script runs.

## T8. Replace Benchmark Verification Script [completed]
- Files: `scripts/verify_bench_acceptance`.
- Subtasks:
  - Keep `set -euo pipefail`.
  - Resolve repo root from script path and `cd` there.
  - Run public `cargo run --bin db -- bench`.
  - Validate the `DB_BENCH` pass sentinel.
  - Validate evidence file path, top-level required fields, fixed constants, positive metrics, speedup thresholds, recovery bound, indexed path evidence, and hard-fail check statuses.
  - Atomically update the same evidence JSON with `commands.verify_bench_acceptance.status="pass"`, its evidence path, its stdout sentinel, and final `result="pass"` only after validation succeeds.
  - Revalidate the updated JSON before printing the pass sentinel.
  - Print exact `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json` on success.
  - Print `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` and exit non-zero on any failure.
- Acceptance:
  - Script does not run an internal script-only benchmark.
  - Script passes from repo root and from outside repo via absolute path invocation.
  - Final JSON contains truthful pass entries for both `commands.db_bench` and `commands.verify_bench_acceptance`.

## T9. Update Required Documentation [completed]
- Files: `docs/cli_contract.md`, `docs/benchmarks.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`.
- Subtasks:
  - Document `db bench`, evidence path, stdout sentinels, and exit codes in CLI contract.
  - Document the narrow `INTEGER` alias needed by the Section 14 schema in SQL/CLI docs.
  - Replace or clearly supersede older 1k benchmark acceptance wording in benchmark docs for current Section 14 evidence.
  - Add performance report section with formulas, thresholds, metric fields, and evidence path.
  - Map requirement IDs `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, and `EVID-16-7` in V1 acceptance docs.
  - Record bug cause/fix/regression state in bug diary, or no-bug rationale if no implementation bug is found.
- Acceptance:
  - Docs include command evidence, artifact path, hard-fail policy, and traceability.

## T10. Verify And Capture Evidence [completed]
- Commands:
  - `cargo test --test cli_contract`
  - `cargo test --test bench_acceptance`
  - `scripts/verify_bench_acceptance`
  - `tmp_cwd && <repo>/scripts/verify_bench_acceptance`
  - `scripts/verify`
- Evidence:
  - `target/bench_acceptance/section14-benchmark-acceptance.json`
  - `DB_BENCH` stdout sentinel
  - `BENCH_ACCEPTANCE` stdout sentinel
  - docs diff and final report requirement ID map
- Acceptance:
  - All required commands exit `0`.
  - Any failure blocks completion rather than being documented as success.
