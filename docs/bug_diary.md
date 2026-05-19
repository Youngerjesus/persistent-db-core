# Bug Diary

## Section 14 Benchmark Acceptance

Requirement IDs: `EVID-16-7`, with related coverage for `METRIC-14-1`,
`METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, and `EVID-15`.

Bug found: the previous benchmark acceptance path was script-only and reused a
1,000-row `db exec` workload instead of public `db bench` Section 14 evidence.
The CLI help also treated benchmark execution as reserved.

Cause: the older acceptance slice predated the Section 14 hard-fail contract and
therefore had no public benchmark command, no 100,000-row fixture evidence, and
no validator for index-vs-scan speedup or recovery proportionality.

Fix: add public `db bench`, generate
`target/bench_acceptance/section14-benchmark-acceptance.json`, make
`scripts/verify_bench_acceptance` invoke public `db bench`, validate the
hard-fail checks, and document the CLI/evidence contract.

Regression status: covered by `tests/cli_contract.rs`,
`tests/bench_acceptance.rs`, `tests/bench_acceptance_contract.rs`,
`tests/sql_exec.rs`, `scripts/verify_bench_acceptance`, and baseline
`scripts/verify`.

Bug found: the real Section 14 100,000-row persisted workload exposed storage
append/replay cursor scaling problems. The benchmark path needs durable record
counts and WAL frame ids, but the earlier storage path could derive those values
by replaying or recounting persisted records during append/replay-sensitive
flows.

Cause: the storage layer did not carry enough append cursor state as durable
metadata for the benchmark-scale workload. That made Section 14 evidence
generation depend on repeated persisted-file inspection instead of proportional
append and replay bookkeeping.

Fix: track WAL frame ids and durable record counts incrementally in the storage
path used by the benchmark fixture, and keep Section 14 evidence generation on
that proportional append/replay cursor rather than repeated full-file counting.

Regression status: covered by `tests/bench_acceptance.rs`,
`tests/bench_acceptance_contract.rs`, `scripts/verify_bench_acceptance`, and
baseline `scripts/verify`. The generated evidence records
`row_count=100000`, positive persisted size metrics, and passing hard-fail rows.

Bug found: WAL replay scaling was benchmark-facing for Section 14 recovery
evidence. The 10,000-transaction recovery slice must prove proportional reopen
behavior, but earlier replay behavior could rescan page-file state for every WAL
frame in the measured recovery path.

Cause: replay validation and append-position discovery were coupled to
page-file scans instead of the replay cursor. That kept correctness intact for
small fixtures but made the Section 14 recovery metric vulnerable to
non-proportional work.

Fix: make WAL replay advance from recorded frame/cursor state and avoid
rescanning the page file for every frame in the benchmark recovery path. The
verifier now rejects recovery evidence unless `wal_replay_applied_records=10000`,
`recovery.recovered_row_count=10000`, representative lookup matches the
committed row, and `recovery.recovery_ms <= max(2000, recovery.wal_file_bytes /
4096)`.

Regression status: covered by `tests/bench_acceptance.rs`,
`scripts/verify_bench_acceptance`, and baseline `scripts/verify`. Current
evidence includes `recovery.recovery_ms=525.482`,
`recovery.wal_replay_applied_records=10000`, and
`recovery.wal_file_bytes=1838052` from
`target/bench_acceptance/section14-benchmark-acceptance.json`.
