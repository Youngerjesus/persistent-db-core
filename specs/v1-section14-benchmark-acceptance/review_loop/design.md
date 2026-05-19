# Section 14 100k 벤치마크 및 hard-fail 수용 증거

## 정규화된 후보
- Rank: 1
- Feature slug: v1-section14-benchmark-acceptance
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
현재 V1 core 기능은 여러 milestone으로 구현·검증되었지만, current artifact는 100000-row 벤치마크, index-vs-scan 하한, WAL recovery proportionality, hard-fail rejection, performance report evidence를 아직 satisfied로 보지 않는다. 기존 bench evidence는 1k lower-bound 중심으로 보이며 Section 14 acceptance 계약을 충족하기에 부족하다.

## 지금 해야 하는 이유
SQL, WAL, crash matrix, db check, differential/property, primary/secondary index baseline이 이미 존재하고 queue가 비어 있다. 남은 launch-readiness 병목은 새 기능 추가보다 V1 성능·수용 증거를 current digest와 artifact requirement ID에 맞춰 재현 가능하게 만드는 것이다.

## 기대 산출물 변화
`db bench`와 `scripts/verify_bench_acceptance`가 Section 14 100000-row workload, required metric fields, index-use proof, lower-bound hard fail, WAL/recovery metrics를 생성·검증하고, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`가 해당 requirement IDs와 command evidence를 명시한다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/bench.rs
- tests/bench_acceptance.rs
- scripts/verify_bench_acceptance
- docs/benchmarks.md
- docs/performance_report.md
- docs/v1_acceptance.md
- docs/bug_diary.md
- route:db bench
- flow:section14-performance-acceptance

## Risk flags
- benchmark_runtime_cost
- performance_flakiness_risk

## 근거
- Root Progress Projection에서 `artifact_status`는 `open`이고 `gate-v1-bench-docs-acceptance`는 missing satisfied requirement rows 상태다.
- Root Progress Projection의 open requirement에 `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`, `EVID-15`, `EVID-16-7` rows가 포함되어 있다.
- `ssot/current-plan.md`의 `gap-v1-bench-docs-acceptance`는 benchmark lower bounds와 required V1 docs를 current-plan gap으로 정의한다.
- `ssot/current-artifact.md`는 Section 14 성능 rows에 100000-row dataset, index-vs-scan lower bounds, WAL/recovery metrics, hard-fail rejection review를 요구한다.
- Active managed repo snapshot의 `work_queue/progress.md`는 SQL, recovery, check, differential, benchmark, index baseline 위의 remaining V1 acceptance blocker를 다음 handoff 대상으로 지목한다.
- 기존 bench evidence ref는 `target/bench_acceptance/v1-bench-docs-acceptance.json`의 1k insert/reopen lower-bound 중심으로 요약되어 current artifact의 100000-row Section 14 계약과 차이가 있다.
- Queue Snapshot은 비어 있고 managed repo git status는 clean이다.
