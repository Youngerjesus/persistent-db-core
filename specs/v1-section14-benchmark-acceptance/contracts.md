# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: Section 14 benchmark acceptance에 필요한 public `db bench` CLI behavior, `scripts/verify_bench_acceptance`, focused tests, `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`.
- 생성 대상 verification output: `target/bench_acceptance/section14-benchmark-acceptance.json`, `db bench` stdout sentinel, `scripts/verify_bench_acceptance` stdout sentinel, `scripts/verify` 성공 output.
- 생성 대상 리포트 업데이트: scheduler run report에는 필수 command, evidence file path, requirement ID별 pass/fail 근거가 연결되어야 합니다.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `db bench`는 public CLI command로 도입해야 하며 script-only internal benchmark path로 대체할 수 없습니다.
- 필수 command는 `scripts/verify_bench_acceptance`와 `scripts/verify`입니다. 두 command는 repo root에서 exit code 0이어야 하며, `scripts/verify_bench_acceptance`는 repo 밖 caller cwd에서 absolute path invocation으로도 통과해야 합니다.
- `db bench`는 `target/bench_acceptance/section14-benchmark-acceptance.json`을 생성하고 stdout에 성공 시 `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`, 실패 시 `DB_BENCH: FAIL check=<check_id> reason=<reason>`을 출력해야 합니다. 성공은 exit code 0, 실패는 non-zero exit code여야 합니다.
- `scripts/verify_bench_acceptance`는 public `db bench`를 실행해야 하며, 성공 시 `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`, 실패 시 `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>`을 출력해야 합니다.
- Evidence JSON의 top-level required fields는 `schema_version`, `artifact`, `row_count`, `primary_key_type`, `secondary_index_column`, `secondary_index_name`, `text_bytes_min`, `text_bytes_max`, `deterministic_seed`, `warmup_runs`, `measurement_runs`, `runtime_cap_seconds`, `commands`, `workload`, `metrics`, `recovery`, `index_use_evidence`, `hard_fail_checks`, `result`입니다.
- `commands.db_bench.status`는 `pass`, `commands.db_bench.evidence_path`는 `target/bench_acceptance/section14-benchmark-acceptance.json`, `commands.db_bench.stdout_sentinel`은 `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`과 일치해야 합니다.
- `commands.verify_bench_acceptance.status`는 `pass`, `commands.verify_bench_acceptance.evidence_path`는 `target/bench_acceptance/section14-benchmark-acceptance.json`, `commands.verify_bench_acceptance.stdout_sentinel`은 성공 sentinel과 일치해야 합니다.
- CLI documentation contract: `docs/cli_contract.md`와 `db --help`는 `bench` command, evidence path, stdout sentinel, exit-code behavior를 포함해야 합니다. 기존 `bench_reserved_future_command_remains_unsupported` 계열 test는 public `db bench` contract 검증으로 전환해야 하며, `bench`를 unsupported/reserved command로 검증하면 hard fail입니다.
- Dataset contract: `row_count=100000`, `primary_key_type="INTEGER"`, `secondary_index_column="group_key"`, `secondary_index_name="idx_section14_bench_group_key"`, `text_bytes_min=8`, `text_bytes_max=64`, `deterministic_seed=140014`, `warmup_runs=1`, `measurement_runs=5`, `runtime_cap_seconds<=300`이어야 합니다.

## Section 14 benchmark fixture contract
- Table schema는 `bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)`입니다. Secondary index는 `idx_section14_bench_group_key` on `bench_items(group_key)` 하나를 사용합니다.
- Dataset generation은 `id` 1부터 100000까지 순차 insert로 고정합니다. 각 row의 `group_key`는 `((id * 7919 + deterministic_seed) % 10000) + 1`이고, `payload`는 ASCII text이며 byte length는 `8 + ((id * 17 + deterministic_seed) % 57)`입니다.
- `payload` 내용은 deterministic이어야 하며 동일 `id`, `deterministic_seed`, `payload` length에서 항상 같은 byte sequence를 생성해야 합니다. Worker가 임의 randomness, wall-clock seed, machine-specific value를 사용하면 hard fail입니다.
- Primary-key lookup set은 measured run마다 `lookup_id(n) = ((deterministic_seed + n * 9973) % row_count) + 1` for `n=0..99`로 고정합니다.
- Secondary equality query set은 measured run마다 `equality_key(n) = ((deterministic_seed + n * 433) % 10000) + 1` for `n=0..99`로 고정하며 predicate는 `group_key = equality_key(n)`입니다.
- Range query set은 measured run마다 `range_low(n) = ((deterministic_seed + n * 137) % 9950) + 1`, `range_high(n) = range_low(n) + 49` for `n=0..49`로 고정하며 predicate는 `group_key BETWEEN range_low(n) AND range_high(n)`입니다. Range window는 50 distinct `group_key` values이고 selectivity는 전체 100000 rows 기준 약 0.5%로 검증해야 합니다.
- Indexed comparison은 동일 dataset, 동일 predicate, 동일 expected result ordering에서 indexed path와 full-scan comparison path를 모두 측정해야 합니다. Full-scan comparison path는 index를 비활성화하거나 benchmark harness의 explicit scan mode를 사용해 만들어야 하며, 결과 row count와 deterministic result hash가 indexed path와 일치해야 합니다.
- Measured query count는 measured run마다 primary-key lookup 100개, secondary equality indexed 100개, secondary equality scan 100개, range indexed 50개, range scan 50개로 총 400개입니다. `measurement_runs=5`이므로 warmup 제외 measured query count는 총 2000개입니다.
- Workload contract: `workload`는 위 fixture contract의 table schema, deterministic generation rule, query sets, range window, scan comparison mode, measured query count와 sequential inserts, 10000 committed transactions 이후 close/reopen recovery를 포함해야 합니다.
- Metrics contract: `metrics`는 `sequential_insert_elapsed_ms`, `insert_throughput_rows_per_sec`, `primary_key_lookup_median_ms`, `secondary_equality_indexed_median_ms`, `secondary_equality_scan_median_ms`, `range_indexed_median_ms`, `range_scan_median_ms`, `equality_index_speedup`, `range_index_speedup`, `db_file_bytes`, `wal_file_bytes`를 포함해야 하며 numeric metric은 모두 positive value여야 합니다.
- Speedup formula: `equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms`, `range_index_speedup = range_scan_median_ms / range_indexed_median_ms`입니다. 두 산식은 동일 dataset, 동일 query set, 동일 deterministic seed, measured run median 기준으로 계산해야 합니다.
- Threshold contract: `equality_index_speedup >= 5.0`, `range_index_speedup >= 3.0`이어야 합니다. 기준 미달, runtime cap 초과, measurement retry 필요, required field 누락, non-deterministic seed 사용은 hard fail입니다.
- Full-scan rejection contract: eligible indexed equality/range query가 indexed measurement path에서 full scan으로 실행되면 `hard_fail_checks`에 실패가 기록되어야 하고 verifier는 non-zero exit로 종료해야 합니다. 정상 통과 시 `index_use_evidence`는 query별 `query_kind`, `predicate`, `expected_access_path`, `observed_access_path`, `used_index`, `scan_rejected`, `indexed_result_count`, `scan_result_count`, `result_hash_match`를 포함해야 합니다.
- Recovery contract: `recovery.committed_transaction_count=10000`, `recovery.recovered_row_count=10000`, `recovery.recovery_ms`는 positive value, `recovery.wal_file_bytes`는 numeric value, `recovery.representative_lookup_result`는 committed row의 key/value 일치를 증명해야 합니다.
- Recovery proportionality bound: `recovery.recovery_ms <= max(2000, recovery.wal_file_bytes / 4096)`이어야 합니다. Bound 초과, recovered row count 불일치, representative lookup 실패는 hard fail입니다.
- Documentation contract: `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`는 targeted artifact requirement IDs, command evidence, evidence file path, CLI stdout/exit-code behavior, threshold 산식, hard-fail policy, 발견 bug의 원인·수정·회귀 테스트 상태를 명시해야 합니다.

## Requirement Traceability Contract

| Requirement ID | Acceptance item | Required command/manual evidence | Expected output file/doc section | Hard-fail condition |
| --- | --- | --- | --- | --- |
| `METRIC-14-1` | 100000-row dataset shape와 deterministic workload | `db bench`, `scripts/verify_bench_acceptance` | `target/bench_acceptance/section14-benchmark-acceptance.json`의 dataset fields와 `workload` fixture fields | row count, column type, index name, text byte bounds, seed, query set, range window, measured query count 누락 또는 불일치 |
| `METRIC-14-2` | insert, lookup, throughput, persisted size metric | `db bench`, `scripts/verify_bench_acceptance` | Evidence JSON `metrics`, `docs/performance_report.md` Section 14 | required metric 누락, non-positive value, 문서 evidence 누락 |
| `METRIC-14-3` | equality/range index-vs-scan lower bounds | `scripts/verify_bench_acceptance` | Evidence JSON speedup and median latency fields, scan comparison result hash | equality `< 5.0`, range `< 3.0`, 동일 dataset/query/result 비교 증거 누락 |
| `METRIC-14-4` | 10000 committed transactions 이후 WAL/recovery | `scripts/verify_bench_acceptance` | Evidence JSON `recovery` fields | proportionality bound 초과, row count 불일치, representative lookup 실패 |
| `FAIL-14-5` | eligible indexed query full-scan hard-fail rejection | `scripts/verify_bench_acceptance`, focused regression test | Evidence JSON `index_use_evidence`, `hard_fail_checks`, test output | eligible query full scan, hard-fail check 미작동 |
| `EVID-15` | CLI contract, performance report, V1 acceptance 문서 evidence | Manual doc review plus `scripts/verify` | `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md` | requirement ID, command, evidence path, formula, CLI stdout/exit-code contract 누락 |
| `EVID-16-7` | bug diary와 regression 상태 | Manual doc review plus `scripts/verify` | `docs/bug_diary.md` | 발견 bug의 원인, 수정, regression test 또는 no-bug rationale 누락 |

## 완료 정의
- 완료는 구현 존재 여부가 아니라 Acceptance Evidence Contract와 Requirement Traceability Contract의 모든 항목이 evidence로 충족될 때만 인정됩니다.
- `scripts/verify_bench_acceptance`와 `scripts/verify` 중 하나라도 실패하면 미완료입니다.
- Section 14 외 non-targeted V1 obligations는 carry-forward로 남기며 이번 task 완료 조건으로 닫지 않습니다.
- Artifact delta는 command output, evidence JSON, 문서 section, scheduler run report에 requirement ID별로 연결되어야 합니다.
