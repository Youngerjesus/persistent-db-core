# V1 Benchmark Acceptance

This document defines the repo-local benchmark evidence for V1 acceptance. The authoritative command is:

```bash
scripts/verify_bench_acceptance
```

The command writes machine-readable evidence to:

```text
target/bench_acceptance/v1-bench-docs-acceptance.json
```

The evidence id for final reporting is `evidence-v1-benchmark-lower-bounds`.

## Scope

The benchmark measures the existing single-process Rust CLI path by invoking `cargo run --quiet --bin db -- exec <temp-db> <sql>`. It does not add or call a public `db bench` command; `db bench` remains a reserved future CLI command and is not available for users.

The workload uses a deterministic temporary database and this table:

```sql
CREATE TABLE bench_items(id INT, value TEXT);
```

Each scenario uses 1,000 rows with `id` values from `1` through `1000` and `value` strings from `value-0001` through `value-1000`.

## Scenarios

| Scenario | Measured operation | Required validation | Lower-bound floor |
| --- | --- | --- | --- |
| `bench_insert_1k` | Create `bench_items(id INT, value TEXT)` and insert 1,000 rows through `db exec`. | The command exits successfully and stderr is empty. | `insert_rows_per_second >= 25` |
| `bench_reopen_select_1k` | Reopen the populated database in a new `db exec` process and run `SELECT * FROM bench_items;`. | The output has header `id|value`, 1,000 data rows, first row `1|value-0001`, last row `1000|value-1000`, and stderr is empty. | `select_rows_per_second >= 50` |

## Measurement Policy

Each scenario runs one warmup iteration that is not included in pass/fail evidence, followed by three measured iterations. Every measured iteration uses a fresh temporary database. The pass/fail rule uses the minimum measured rows per second, recorded as `observed_min_rows_per_second`; average, median, or best-case values are not sufficient. If any measured iteration is below its floor, `scripts/verify_bench_acceptance` exits non-zero.

These floors are acceptance lower bounds for the repo-local V1 smoke workload. They are intentionally conservative and do not claim throughput on arbitrary hardware.

## Current Evidence

The authoritative current-run values are the `observed_min_rows_per_second` fields in `target/bench_acceptance/v1-bench-docs-acceptance.json`. The latest local implementation evidence used for this acceptance update recorded:

| Scenario | Current observed minimum | Required floor | Interpretation |
| --- | --- | --- | --- |
| `bench_insert_1k` | `2793.296` rows/second | `insert_rows_per_second >= 25` | The minimum measured insert iteration exceeded the acceptance floor. |
| `bench_reopen_select_1k` | `4065.041` rows/second | `select_rows_per_second >= 50` | The minimum measured reopen/select iteration exceeded the acceptance floor. |

The final verifier should rerun `scripts/verify_bench_acceptance` and treat the regenerated JSON as the current source of truth if the numbers differ from this documented implementation-run snapshot.

## Environment Assumptions

The evidence is local-machine benchmark evidence. The generated JSON records the concrete OS, architecture, CPU, `rustc` version, `cargo` version, and logical CPU count for the run. The implementation-run snapshot above was produced on Darwin arm64 with an Apple M2 Max CPU and Rust/Cargo 1.84.0 toolchain. Different OS, CPU, thermal state, storage, or toolchain conditions may produce different measured values; acceptance is based on meeting the conservative floors in the current verifier run, not on reproducing the exact snapshot values.

## JSON Schema

The JSON artifact includes at least these top-level fields:

```text
schema_version
evidence_id
repo_sha
created_at
command
environment
policy
scenarios
overall_passed
```

`environment` records OS, architecture, `rustc` version, `cargo` version, logical CPU count, and CPU model when available.

Each `scenarios[]` entry includes:

```text
id
row_count
warmup_iterations
measured_iterations
threshold_rows_per_second
observed_min_rows_per_second
iterations
passed
```

Each `iterations[]` entry includes:

```text
iteration
duration_ms
rows_per_second
exit_status
```

## Non-Guarantees

This benchmark is not a network benchmark, a multi-process concurrency benchmark, a durability stress benchmark, or a general hardware performance guarantee. It does not measure unsupported SQL, secondary indexes, concurrent writers, or the reserved `db bench` surface.
