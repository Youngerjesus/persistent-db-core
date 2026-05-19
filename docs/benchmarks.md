# V1 Section 14 Benchmark Acceptance

Section 14 benchmark acceptance is a public CLI contract. The writer command is:

```bash
db bench
```

The verifier command is:

```bash
scripts/verify_bench_acceptance
```

Both commands use the canonical evidence file:

```text
target/bench_acceptance/section14-benchmark-acceptance.json
```

Successful `db bench` prints:

```text
DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json
```

Successful `scripts/verify_bench_acceptance` prints:

```text
BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json
```

Failures use `DB_BENCH: FAIL check=<check_id> reason=<reason>` or
`BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>`.

## Section 14 Fixture

The benchmark fixture table is:

```text
bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)
```

The dataset has 100,000 rows, `primary_key_type="INTEGER"`, secondary index
`idx_section14_bench_group_key`, `secondary_index_column="group_key"`,
`deterministic_seed=140014`, `text_bytes_min=8`, `text_bytes_max=64`,
`warmup_runs=1`, and `measurement_runs=5`.

Generation is fixed:

```text
group_key = ((id * 7919 + deterministic_seed) % 10000) + 1
payload length = 8 + ((id * 17 + deterministic_seed) % 57)
```

Measured query sets are fixed by the Section 14 contract: 100 primary-key
lookups, 100 secondary equality indexed lookups, 100 secondary equality scan
comparisons, 50 secondary range indexed scans, and 50 range scan comparisons per
measured run. The range window is 50 distinct `group_key` values. With five
measured runs, warmup excluded, the measured query count is 2,000.

## Metrics And Thresholds

The evidence `schema_version` includes top-level `metrics`, `recovery`,
`index_use_evidence`, `hard_fail_checks`, and `result` fields.

Required metric fields are:

```text
sequential_insert_elapsed_ms
insert_throughput_rows_per_sec
primary_key_lookup_median_ms
secondary_equality_indexed_median_ms
secondary_equality_scan_median_ms
range_indexed_median_ms
range_scan_median_ms
equality_index_speedup
range_index_speedup
db_file_bytes
wal_file_bytes
```

The lower-bound formulas are fixed:

```text
equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms
range_index_speedup = range_scan_median_ms / range_indexed_median_ms
```

Acceptance requires:

```text
equality_index_speedup >= 5.0
range_index_speedup >= 3.0
recovery_ms <= max(2000, wal_file_bytes / 4096)
```

Recovery evidence records `committed_transaction_count=10000`,
`wal_replay_applied_records=10000`, `recovered_row_count=10000`, positive
`recovery_ms`, numeric `wal_file_bytes`, and a matching
`representative_lookup_result`.

## Hard-Fail Policy

The verifier rejects missing required fields, non-positive required numeric
metrics, threshold misses, runtime cap violations, retry-required evidence,
wrong deterministic fixture constants, and any eligible indexed equality or
range query whose indexed measurement path observes a full scan. Passing
`index_use_evidence` rows record `query_kind`, `predicate`,
`expected_access_path`, `observed_access_path`, `used_index`, `scan_rejected`,
`indexed_result_count`, `scan_result_count`, and `result_hash_match`.

Requirement traceability:

| Requirement ID | Evidence |
| --- | --- |
| `METRIC-14-1` | 100,000-row dataset fields and workload fixture fields in the JSON |
| `METRIC-14-2` | elapsed, throughput, lookup, and file-size metrics |
| `METRIC-14-3` | equality/range speedup formulas, medians, and scan comparison evidence |
| `METRIC-14-4` | 10,000-transaction recovery fields and proportionality check |
| `FAIL-14-5` | full-scan rejection rows in `index_use_evidence` and `hard_fail_checks` |
| `EVID-15` | CLI contract, performance report, and V1 acceptance links |
| `EVID-16-7` | bug diary cause/fix/regression or no-bug rationale |

## Non-Guarantees

This benchmark is local Section 14 acceptance evidence for the single-process
Rust CLI. It is not a network benchmark, multi-process concurrency benchmark,
distributed storage benchmark, or a claim about arbitrary hardware throughput.
