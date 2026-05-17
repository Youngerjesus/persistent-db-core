# Technical Design: v1-bench-docs-acceptance

## Boundary
This task adds verification and documentation evidence around the existing single-process Rust CLI. It does not add database engine features.

## `scripts/verify_bench_acceptance`

### Responsibilities
- Locate repo root from the script path and work from repo root, independent of caller cwd.
- Create deterministic temp database paths under a temp directory.
- Execute all benchmark work through:

```bash
cargo run --quiet --bin db -- exec <temp-db> <sql>
```

- Never call or create `db bench`.
- Run one warmup per scenario, excluded from pass/fail.
- Run three measured iterations per scenario, each with a fresh temp database.
- Fail non-zero when any measured iteration exits non-zero, writes stderr unexpectedly, produces invalid select output, or falls below threshold.
- Write machine-readable JSON to `target/bench_acceptance/v1-bench-docs-acceptance.json`.
- Print only a concise human summary to stdout.

### Workload
- Table: `bench_items(id INT, value TEXT)`.
- Rows: 1,000.
- Values: `id=1..1000`, `value='value-0001'..'value-1000'`.
- Insert SQL should create the table and insert all rows in deterministic order.

### Scenarios
| Scenario | Validation | Threshold |
|---|---|---|
| `bench_insert_1k` | Successful exit status and empty stderr for create+insert SQL. | `insert_rows_per_second >= 25` |
| `bench_reopen_select_1k` | Insert with one process, reopen same DB in a new `cargo run` process, `SELECT * FROM bench_items;`, header plus 1,000 rows, first row `1|value-0001`, last row `1000|value-1000`, empty stderr. | `select_rows_per_second >= 50` |

### JSON Schema Minimum
Top-level fields:
- `schema_version`
- `evidence_id`
- `repo_sha`
- `created_at`
- `command`
- `environment`
- `policy`
- `scenarios`
- `overall_passed`

`environment` fields:
- `os`
- `architecture`
- `rustc_version`
- `cargo_version`
- `logical_cpu_count`
- `cpu_model` when stable and available

Each scenario:
- `id`
- `row_count`
- `warmup_iterations`
- `measured_iterations`
- `threshold_rows_per_second`
- `observed_min_rows_per_second`
- `iterations`
- `passed`

Each measured iteration:
- `iteration`
- `duration_ms`
- `rows_per_second`
- `exit_status`

## `docs/benchmarks.md`
- Declare `scripts/verify_bench_acceptance` and `target/bench_acceptance/v1-bench-docs-acceptance.json` as the benchmark acceptance evidence surface.
- Document workload size, schema, warmup/measurement policy, thresholds, output schema, and interpretation.
- State that the lower bound is a repo-local acceptance floor from measured minimums, not a universal performance guarantee.
- State non-guarantees: arbitrary hardware, network, concurrency, multi-process behavior, full 100k V1 benchmark suite, secondary-index benchmark, and public `db bench`.

## `docs/v1_acceptance.md`
- Include evidence id `evidence-v1-acceptance-docs`.
- State source: task handoff snapshot of `autopilot/ssot/current-artifact.md`.
- Include every required gate and requirement id.
- Columns: gate id, requirement id, evidence path, verification command or manual review evidence, status.
- Status rules:
  - `verified` only when current repo evidence path and command/manual evidence exist.
  - `pending_current_task_verification` for new benchmark/docs rows until verification has run.
  - `blocked_missing_evidence` for known missing evidence such as secondary index proof.
  - `out_of_scope_for_this_task` only when the requirement is real but not completed by this task.

## Minimal Tests
Tests are optional unless implementation reveals a regression risk. If added, keep them focused:
- Reserved `db bench` remains unsupported through existing CLI contract tests.
- Script file exists, is executable, and emits JSON with required fields can be covered by the script itself rather than a separate integration test.

## Final Report Linkage
The implementation completion report must name:
- `evidence-v1-benchmark-lower-bounds`
- `evidence-v1-acceptance-docs`
