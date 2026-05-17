# V1 benchmark lower bounds 및 acceptance docs 고정

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-18-05-57-03
- Task ID: task-2026-05-18-05-57-03-v1-bench-docs-acceptance
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: V1 benchmark lower bounds 및 acceptance docs 고정
- Artifact: v1-bench-docs-acceptance

## 목표
- V1의 핵심 구현, 복구, crash, invariant, differential evidence는 대부분 충족되었지만, benchmark lower bounds와 launch gate별 acceptance documentation이 없어 최종 artifact completion 판단이 열려 있습니다.

## 지금 해야 하는 이유
- Root Progress Projection에서 `gate-v1-bench-docs-acceptance`만 open이고, Queue Snapshot은 비어 있으며, current plan도 같은 gap을 다음 V1 acceptance evidence 후보로 지정합니다.

## 기대 산출물 변화
- managed repo에 repo-local runnable benchmark verification surface, benchmark lower-bound 문서, V1 acceptance guide를 추가하고, 해당 문서가 모든 launch gate와 evidence requirement를 현재 증거에 매핑하도록 만듭니다.
- benchmark surface는 `scripts/verify_bench_acceptance`로 고정합니다. 이번 task는 user-facing `db bench` CLI를 추가하지 않으며, `db bench`는 계속 unsupported future command로 남아야 합니다.

## 의도한 변경 대상
- scripts/verify_bench_acceptance
- docs/benchmarks.md
- docs/v1_acceptance.md
- docs/v1_spec.md
- docs/cli_contract.md: `db bench` unsupported 상태를 유지하는 참조 갱신이 필요할 때만 변경합니다.
- tests/

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: a5d22e67f3f2feefae757ee12c25d6b88849a849
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: docs/v1_spec.md, docs/cli_contract.md, tests/cli_contract.rs, tests/crash_matrix.rs, tests/db_check.rs, tests/differential_property.rs, tests/fixtures/crash_matrix/README.md, tests/page_storage.rs, tests/primary_index.rs

## Risk flags
- benchmark_environment_variability
- docs_must_map_only_verified_evidence
- avoid_unproven_performance_claims

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `artifact_status=open`, `gate-v1-bench-docs-acceptance` status=`open`, missing_requirement_ids=`req-v1-benchmark-lower-bounds`, `req-v1-acceptance-docs`.
- Current Artifact: `req-v1-benchmark-lower-bounds`는 benchmark output, benchmark command, lower-bound documentation을 요구합니다.
- Current Artifact: `req-v1-acceptance-docs`는 V1 usage와 acceptance docs가 각 launch gate를 evidence에 매핑해야 한다고 정의합니다.
- Current Plan: `gap-v1-bench-docs-acceptance`는 `metric-v1-acceptance-evidence`와 `gate-v1-bench-docs-acceptance`에 연결되어 있으며 next candidate hint가 benchmark harness, lower-bound docs, acceptance guide입니다.
- Active Managed Repo Snapshot: `git_status=clean`, Queue Snapshot=`[]`.
- Managed repo progress: 다음 작은 handoff 후보로 benchmark/acceptance docs를 언급하며, 기존 SQL, recovery, check, differential baseline 위에서 진행 가능하다고 기록합니다.
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/progress/launch-readiness.md
- autopilot/progress/launch-readiness.json
- autopilot/project_manager/tasks/tasks.json
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/tests/cli_contract.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/v1_spec.md
- persistent-db-core_worktree/main/work_queue/progress.md
- HELP
- main
- bench_reserved_future_command_remains_unsupported
- autopilot/project_manager/tasks/tasks.json: candidate-v1-bench-docs-acceptance 검색 결과 없음
- autopilot/project_manager/tasks/tasks.json: gap-v1-bench-docs-acceptance 검색 결과 없음
- autopilot/project_manager/specs: v1-bench-docs-acceptance spec directory 없음

## 범위
- In scope: selected candidate only.
- Out of scope: unrelated breadth features.
- In scope: `scripts/verify_bench_acceptance` 기반의 repo-local benchmark verification, `docs/benchmarks.md`, `docs/v1_acceptance.md`, 필요한 최소 문서 참조 갱신, benchmark script 검증을 위한 focused tests.
- Out of scope: user-facing `db bench` CLI 추가, CLI output 또는 exit code 확장, networked benchmark, multi-process concurrency benchmark, V1 acceptance와 무관한 성능 개선.

## Benchmark verification surface
- 필수 실행 명령은 `scripts/verify_bench_acceptance`입니다. 이 script는 managed repo의 clean checkout에서 실행 가능해야 하며, caller cwd가 repo root가 아니어도 repo root를 찾아 실행해야 합니다.
- `scripts/verify_bench_acceptance`는 `cargo run --quiet --bin db -- exec <temp-db> <sql>` 기반으로 측정해야 하며, user-facing benchmark subcommand를 호출하거나 추가하면 안 됩니다.
- benchmark workload는 deterministic temp database에서 실행합니다. 입력 데이터는 `bench_items(id INT, value TEXT)` table과 `id=1..1000`, `value='value-0001'..'value-1000'` 형식의 1,000 rows입니다.
- warmup 정책은 전체 scenario 1회를 warmup으로 실행하고 결과를 pass/fail에 포함하지 않는 것입니다. 측정은 같은 scenario를 3회 반복하며, 각 반복은 새 temp database를 사용해야 합니다.
- 필수 scenario는 다음과 같습니다.
  - `bench_insert_1k`: table 생성과 1,000 row insert를 실행하고, 성공 exit code와 empty stderr를 확인합니다.
  - `bench_reopen_select_1k`: insert 이후 새 process로 같은 database를 reopen하여 `SELECT * FROM bench_items;`를 실행하고, header와 1,000 rows, 첫 row와 마지막 row의 deterministic value를 확인합니다.
- lower-bound threshold는 measured iteration의 최소값 기준으로 판정합니다. `bench_insert_1k`는 `insert_rows_per_second >= 25`, `bench_reopen_select_1k`는 `select_rows_per_second >= 50`이어야 합니다.
- 변동성 처리는 보수적인 threshold와 최소 측정값 판정으로 제한합니다. 평균이나 최선값만으로 통과할 수 없고, 기준 미달 iteration이 하나라도 있으면 script는 non-zero로 실패해야 합니다.
- benchmark output은 machine-readable JSON으로 `target/bench_acceptance/v1-bench-docs-acceptance.json`에 기록해야 하며, stdout에는 사람이 읽을 수 있는 짧은 summary를 출력할 수 있습니다.
- JSON output schema는 최소한 `schema_version`, `evidence_id`, `repo_sha`, `created_at`, `command`, `environment`, `policy`, `scenarios`, `overall_passed`를 포함해야 합니다.
- `environment`는 최소한 OS, architecture, `rustc` version, `cargo` version, logical CPU count를 포함해야 합니다. CPU model을 안정적으로 얻을 수 있으면 함께 기록합니다.
- 각 `scenarios[]` 항목은 `id`, `row_count`, `warmup_iterations`, `measured_iterations`, `threshold_rows_per_second`, `observed_min_rows_per_second`, `iterations`, `passed`를 포함해야 합니다.
- 각 `iterations[]` 항목은 `iteration`, `duration_ms`, `rows_per_second`, `exit_status`를 포함해야 합니다.
- `docs/benchmarks.md`는 이 script와 JSON artifact path를 권위 있는 benchmark evidence로 설명하고, 측정된 최소 lower bound와 환경 전제를 기록해야 합니다. 문서는 concurrency, network, multi-process, 임의 hardware에서의 성능 보장을 주장하면 안 됩니다.

## V1 acceptance guide contract
- `docs/v1_acceptance.md`의 권위 있는 gate source는 task handoff 시점의 `autopilot/ssot/current-artifact.md`에 있는 Launch Gate Evidence Contract와 Evidence Requirements입니다.
- guide는 다음 launch gate와 requirement id를 모두 포함해야 합니다.
  - `gate-v1-cli-smoke`: `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`
  - `gate-v1-disk-page-storage`: `req-v1-page-storage-restart`, `req-v1-record-format-doc`
  - `gate-v1-sql-schema-exec`: `req-v1-sql-exec-examples`
  - `gate-v1-indexes`: `req-v1-primary-index-proof`, `req-v1-secondary-index-proof`
  - `gate-v1-transactions-wal-recovery`: `req-v1-wal-recovery-proof`
  - `gate-v1-crash-testing`: `req-v1-crash-matrix-output`
  - `gate-v1-differential-property-tests`: `req-v1-differential-property-proof`
  - `gate-v1-db-check-invariants`: `req-v1-db-check-proof`
  - `gate-v1-bench-docs-acceptance`: `req-v1-benchmark-lower-bounds`, `req-v1-acceptance-docs`
- 각 gate row는 gate id, requirement id, evidence path, verification command 또는 manual review evidence, 현재 status를 포함해야 합니다.
- 현재 evidence가 없는 항목은 progress projection만으로 완료 처리하면 안 됩니다. 반드시 explicit blocker 또는 out-of-scope reason을 적어야 합니다.
- `docs/v1_acceptance.md`는 final report에서 참조할 evidence id `evidence-v1-acceptance-docs`를 문서 내에 명시해야 합니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `scripts/verify`가 계속 통과합니다.
- `scripts/verify_bench_acceptance`가 clean checkout에서 실행 가능하며, `target/bench_acceptance/v1-bench-docs-acceptance.json`에 lower-bound 판단에 필요한 수치와 환경 전제를 기록합니다.
- `scripts/verify_bench_acceptance`는 `bench_insert_1k`와 `bench_reopen_select_1k`를 각각 1 warmup, 3 measured iterations로 실행하고, 최소 측정값 기준으로 `insert_rows_per_second >= 25`, `select_rows_per_second >= 50`을 판정합니다.
- benchmark lower bounds 문서가 측정 대상, 입력 규모, 출력 schema, 통과 기준, 변동성 전제, 측정된 최소 보장, 비보장 항목을 명시합니다.
- V1 acceptance docs가 `gate-v1-cli-smoke`부터 `gate-v1-bench-docs-acceptance`까지 모든 launch gate를 현재 evidence requirement, evidence path, 검증 명령 또는 manual review evidence에 매핑합니다.
- V1 acceptance docs가 evidence 없는 항목을 완료로 표시하지 않고 explicit blocker 또는 out-of-scope reason으로 남깁니다.
- 새 문서는 V1 단일 프로세스 Rust CLI 경계를 유지하고, 아직 검증되지 않은 성능 또는 기능 완료를 과장하지 않습니다.
- `db bench`는 계속 unsupported future command로 유지되고, 이번 task는 user-facing CLI surface를 확장하지 않습니다.

## 검증 계획
- Commands to run:
  - `scripts/verify`
  - `scripts/verify_bench_acceptance`
- 필수 evidence path:
  - `docs/benchmarks.md`
  - `docs/v1_acceptance.md`
  - `target/bench_acceptance/v1-bench-docs-acceptance.json`
  - final report의 `evidence-v1-benchmark-lower-bounds`
  - final report의 `evidence-v1-acceptance-docs`
- 기대 증거: scheduler run report, command output, benchmark JSON artifact, acceptance guide 문서 diff, final report evidence id 연결.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
