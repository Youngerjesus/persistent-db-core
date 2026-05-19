# Section 14 100k 벤치마크 및 hard-fail 수용 증거

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-19-15-18-42
- Task ID: task-2026-05-19-15-18-42-v1-section14-benchmark-acceptance
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: Section 14 100k 벤치마크 및 hard-fail 수용 증거
- Artifact: v1-section14-benchmark-acceptance

## 목표
- 현재 V1 core 기능은 여러 milestone으로 구현·검증되었지만, current artifact는 100000-row 벤치마크, index-vs-scan 하한, WAL recovery proportionality, hard-fail rejection, performance report evidence를 아직 satisfied로 보지 않는다. 기존 bench evidence는 1k lower-bound 중심으로 보이며 Section 14 acceptance 계약을 충족하기에 부족하다.

## 지금 해야 하는 이유
- SQL, WAL, crash matrix, db check, differential/property, primary/secondary index baseline이 이미 존재하고 queue가 비어 있다. 남은 launch-readiness 병목은 새 기능 추가보다 V1 성능·수용 증거를 current digest와 artifact requirement ID에 맞춰 재현 가능하게 만드는 것이다.

## 기대 산출물 변화
- public CLI인 `db bench`와 `scripts/verify_bench_acceptance`가 Section 14 100000-row workload, required metric fields, index-use proof, lower-bound hard fail, WAL/recovery metrics를 생성·검증하고, `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`가 해당 requirement IDs와 command evidence를 명시한다.
- `db --help`는 `bench` command를 노출해야 하며, 기존 reserved future command 기대는 public command 기대와 exit-code/stdout 계약으로 전환되어야 한다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/bench.rs
- tests/bench_acceptance.rs
- tests/cli_contract.rs
- scripts/verify_bench_acceptance
- docs/cli_contract.md
- docs/benchmarks.md
- docs/performance_report.md
- docs/v1_acceptance.md
- docs/bug_diary.md
- route:db bench
- flow:section14-performance-acceptance

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: d943f7404a992203822d00ef9a8194e766f15f87
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, scripts/verify_bench_acceptance, docs/benchmarks.md, docs/v1_acceptance.md, work_queue/progress.md, tests/bench_acceptance_contract.rs, tests/cli_contract.rs, docs/v1_spec.md

## Risk flags
- benchmark_runtime_cost
- performance_flakiness_risk

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection에서 `artifact_status`는 `open`이고 `gate-v1-bench-docs-acceptance`는 missing satisfied requirement rows 상태다.
- Root Progress Projection의 open requirement에 `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, `EVID-16-7` rows가 포함되어 있다.
- `ssot/current-plan.md`의 `gap-v1-bench-docs-acceptance`는 benchmark lower bounds와 required V1 docs를 current-plan gap으로 정의한다.
- `ssot/current-artifact.md`는 Section 14 성능 rows에 100000-row dataset, index-vs-scan lower bounds, WAL/recovery metrics, hard-fail rejection review를 요구한다.
- Active managed repo snapshot의 `work_queue/progress.md`는 SQL, recovery, check, differential, benchmark, index baseline 위의 remaining V1 acceptance blocker를 다음 handoff 대상으로 지목한다.
- 기존 bench evidence ref는 `target/bench_acceptance/v1-bench-docs-acceptance.json`의 1k insert/reopen lower-bound 중심으로 요약되어 current artifact의 100000-row Section 14 계약과 차이가 있다.
- Queue Snapshot은 비어 있고 managed repo git status는 clean이다.
- src/main.rs
- scripts/verify_bench_acceptance
- tests/bench_acceptance_contract.rs
- tests/cli_contract.rs
- docs/benchmarks.md
- docs/v1_acceptance.md
- docs/v1_spec.md
- docs/cli_contract.md
- specs/v1-bench-docs-acceptance/contracts.md
- specs/v1-bench-docs-acceptance/impl_review.md
- HELP
- benchmark_acceptance_script_contract_is_pinned
- benchmark_documentation_contract_is_pinned
- bench_reserved_future_command_remains_unsupported
- row_count=1000
- bench_insert_1k
- bench_reopen_select_1k
- project_manager/tasks/tasks.json#task-2026-05-18-05-57-03-v1-bench-docs-acceptance
- task-2026-05-18-05-57-03-v1-bench-docs-acceptance: SUCCESS, artifact_requirement_ids=[req-v1-benchmark-lower-bounds, req-v1-acceptance-docs]
- 현재 queue에는 candidate-v1-section14-benchmark-acceptance 또는 v1-section14-benchmark-acceptance active/reserved task가 없다
- project_manager/specs/v1-bench-docs-acceptance/spec.md
- project_manager/specs/v1-bench-docs-acceptance/contracts.md
- managed repo specs/v1-bench-docs-acceptance/contracts.md

## 범위
- In scope: Section 14 benchmark acceptance에 필요한 public `db bench` CLI behavior, `scripts/verify_bench_acceptance`, focused benchmark/contract tests, `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md` 업데이트.
- In scope: 100000-row workload shape, index-vs-scan lower-bound verification, WAL/recovery metric evidence, eligible indexed query의 full-scan hard-fail rejection, requirement ID별 evidence traceability.
- Out of scope: unrelated SQL semantics 변경, storage format rewrite, non-Section 14 feature work, network/server behavior, multi-process concurrency, distributed storage, query optimizer 확장.
- Out of scope: 이번 target이 아닌 V1 source-required obligations의 완료 처리. 비대상 V1 항목은 carry-forward로 남기며 Section 14 evidence로 대체하지 않습니다.

## 수용 기준
- 완료 판정은 구현 존재 여부가 아니라 `contracts.md`의 command, evidence schema, threshold, recovery, documentation traceability가 모두 충족되는지로 한다.
- `scripts/verify_bench_acceptance`와 baseline `scripts/verify`가 필수 evidence command이며, 둘 중 하나라도 실패하면 task는 완료될 수 없다.
- `db bench`는 public CLI로 도입해야 하며 script-only internal benchmark path로 대체할 수 없다.
- 최종 리포트와 문서 업데이트는 requirement ID별 evidence file, command, hard-fail policy를 추적 가능하게 연결해야 한다.

## Candidate Acceptance Criteria
- `db bench`는 public CLI command이며 `target/bench_acceptance/section14-benchmark-acceptance.json`을 생성해야 한다. 성공 시 exit code 0과 stdout `DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`, 실패 시 non-zero exit code와 stdout `DB_BENCH: FAIL check=<check_id> reason=<reason>`을 사용한다.
- `scripts/verify_bench_acceptance`는 반드시 public `db bench`를 실행·검증해야 하며, 동등한 task-specific internal path로 대체할 수 없다. 성공 시 stdout에 `BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json`, 실패 시 `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` sentinel을 출력한다.
- `docs/cli_contract.md`와 `db --help`는 `db bench` command, evidence path, stdout sentinel, exit-code behavior를 문서화해야 한다. 기존 `bench_reserved_future_command_remains_unsupported` 계열 CLI test는 reserved-command 기대를 제거하고 public `db bench` 기대를 검증하도록 전환해야 한다.
- Evidence JSON은 `row_count=100000`, `primary_key_type="INTEGER"`, `secondary_index_column`, `text_bytes_min=8`, `text_bytes_max=64`, `deterministic_seed`, `warmup_runs`, `measurement_runs`, elapsed time, throughput, `db_file_bytes`, `wal_file_bytes`, `recovery_ms`, index-versus-full-scan latency, `equality_index_speedup`, `range_index_speedup`, `index_use_evidence`, `hard_fail_checks`를 machine-readable field로 포함한다.
- Benchmark workload는 `contracts.md`의 Section 14 benchmark fixture contract를 그대로 따라야 한다. 이 contract는 table schema, `secondary_index_column`, secondary index name, deterministic value generation rule, equality query key 생성 규칙, range predicate/window, scan comparison mode, measured query count를 고정한다.
- Benchmark workload는 동일 dataset과 동일 query set에서 sequential inserts, primary-key lookups, secondary equality lookups, indexed range scans, full scan versus indexed scan, 10000 committed transactions 이후 reopen/recovery를 포함한다. Worker가 threshold 통과에 유리한 dataset, query key, range selectivity, 반복 수를 임의 선택하면 hard fail이다.
- Speedup 산식은 `equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms`, `range_index_speedup = range_scan_median_ms / range_indexed_median_ms`로 고정한다. 두 산식 모두 동일 workload, 동일 dataset, 동일 deterministic seed, `warmup_runs=1`, `measurement_runs=5`, measured run median 기준으로 계산한다.
- 검증은 `equality_index_speedup >= 5.0`, `range_index_speedup >= 3.0`, eligible indexed query의 full-scan 사용 금지, runtime cap 초과, evidence field 누락, flakiness 재시도 필요 상태를 hard fail로 판정한다.
- Recovery evidence는 `committed_transaction_count=10000` 이후 close/reopen command로 생성하며 `wal_file_bytes`, `recovery_ms`, `recovered_row_count=10000`, representative lookup result, `recovery_ms <= max(2000, wal_file_bytes / 4096)` proportionality bound를 기록하고 검증한다.
- `scripts/verify_bench_acceptance`와 baseline `scripts/verify`가 repo root에서 통과하고, `scripts/verify_bench_acceptance`는 repo 밖 caller cwd에서 absolute path invocation으로도 동일 evidence path와 sentinel을 생성해야 한다.
- `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`가 targeted artifact requirement IDs, command evidence, evidence file path, threshold 산식, hard-fail policy, 발견 bug의 원인·수정·회귀 테스트 상태를 명시한다.

## Requirement Traceability

| Requirement ID | Acceptance item | Required command/manual evidence | Expected output file/doc section | Hard-fail condition |
| --- | --- | --- | --- | --- |
| `METRIC-14-1` | 100000-row dataset shape와 deterministic workload | `scripts/verify_bench_acceptance` | `target/bench_acceptance/section14-benchmark-acceptance.json`의 `row_count`, `primary_key_type`, `secondary_index_column`, `text_bytes_min`, `text_bytes_max`, `deterministic_seed` | row count, column type, text byte bounds, seed field 중 하나라도 누락 또는 불일치 |
| `METRIC-14-2` | sequential insert, lookup, throughput, persisted size metric | `db bench`, `scripts/verify_bench_acceptance` | Evidence JSON의 elapsed/throughput/file-size fields와 `docs/performance_report.md` Section 14 row | required metric 누락, non-positive value, 문서 command evidence 누락 |
| `METRIC-14-3` | equality/range index-vs-scan lower bounds | `scripts/verify_bench_acceptance` | Evidence JSON의 `equality_index_speedup`, `range_index_speedup`, indexed/scan median fields | equality speedup `< 5.0`, range speedup `< 3.0`, 동일 dataset 비교 증거 누락 |
| `METRIC-14-4` | 10000 committed transactions 이후 WAL/recovery evidence | `scripts/verify_bench_acceptance` | Evidence JSON의 `committed_transaction_count`, `wal_file_bytes`, `recovery_ms`, `recovered_row_count`, `representative_lookup_result` | recovery bound 초과, recovered row count 불일치, representative lookup 실패 |
| `FAIL-14-5` | eligible indexed query full-scan hard-fail rejection | `scripts/verify_bench_acceptance`와 focused regression test | Evidence JSON의 `index_use_evidence`, `hard_fail_checks`, test output | eligible indexed query가 full scan으로 실행되거나 hard-fail check가 `pass`가 아님 |
| `EVID-15` | CLI contract, performance report, V1 acceptance 문서 evidence 연결 | Manual doc review plus `scripts/verify` | `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md` | requirement ID, command, evidence path, threshold 산식, CLI stdout/exit-code contract 중 하나라도 문서 누락 |
| `EVID-16-7` | bug diary와 regression 상태 연결 | Manual doc review plus `scripts/verify` | `docs/bug_diary.md` | 발견 bug의 원인, 수정, regression test 또는 no-bug rationale 누락 |

## 검증 계획
- Commands to run: `scripts/verify_bench_acceptance`, `scripts/verify`.
- Caller cwd check: repo 밖 임시 cwd에서 `<repo>/scripts/verify_bench_acceptance` absolute path invocation을 실행하고 동일 evidence path와 sentinel을 확인한다.
- CLI contract check: `db --help`가 `bench`를 노출하고, CLI regression test가 `db bench`의 public stdout/exit-code contract와 evidence path를 검증해야 한다.
- 기대 증거: `target/bench_acceptance/section14-benchmark-acceptance.json`, `DB_BENCH` stdout sentinel, `BENCH_ACCEPTANCE` stdout sentinel, `docs/cli_contract.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`, scheduler run report.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
