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
- 생성 대상 코드 또는 문서: `scripts/verify_bench_acceptance`, `docs/benchmarks.md`, `docs/v1_acceptance.md`, 필요한 최소 문서 참조 갱신.
- 생성 대상 테스트 또는 verification output: `scripts/verify`, `scripts/verify_bench_acceptance`, `target/bench_acceptance/v1-bench-docs-acceptance.json`.
- 생성 대상 리포트 업데이트: final report에 `evidence-v1-benchmark-lower-bounds`와 `evidence-v1-acceptance-docs`를 연결합니다.
- `db bench` 또는 다른 user-facing benchmark CLI는 생성 대상이 아닙니다.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `scripts/verify`가 계속 통과합니다.
- `scripts/verify_bench_acceptance`가 clean checkout에서 실행 가능하며, caller cwd가 repo root가 아니어도 repo root를 찾아 실행됩니다.
- `scripts/verify_bench_acceptance`는 user-facing `db bench` CLI를 추가하거나 호출하지 않고 `cargo run --quiet --bin db -- exec <temp-db> <sql>` 기반으로 측정합니다.
- benchmark workload는 deterministic temp database, `bench_items(id INT, value TEXT)`, `id=1..1000`, `value='value-0001'..'value-1000'`를 사용합니다.
- benchmark policy는 전체 scenario 1 warmup, 3 measured iterations, 매 iteration 새 temp database 사용으로 고정합니다.
- 필수 scenario는 `bench_insert_1k`와 `bench_reopen_select_1k`입니다.
- lower-bound threshold는 최소 측정값 기준 `bench_insert_1k.insert_rows_per_second >= 25`, `bench_reopen_select_1k.select_rows_per_second >= 50`입니다.
- 변동성 처리는 최소 측정값 판정으로 고정합니다. 평균, 중앙값, 최선값만으로 통과할 수 없습니다.
- benchmark JSON artifact는 `target/bench_acceptance/v1-bench-docs-acceptance.json`에 생성되어야 합니다.
- benchmark JSON은 `schema_version`, `evidence_id`, `repo_sha`, `created_at`, `command`, `environment`, `policy`, `scenarios`, `overall_passed`를 포함해야 합니다.
- `environment`는 OS, architecture, `rustc` version, `cargo` version, logical CPU count를 포함해야 합니다.
- 각 scenario는 `id`, `row_count`, `warmup_iterations`, `measured_iterations`, `threshold_rows_per_second`, `observed_min_rows_per_second`, `iterations`, `passed`를 포함해야 합니다.
- 각 iteration은 `iteration`, `duration_ms`, `rows_per_second`, `exit_status`를 포함해야 합니다.
- `docs/benchmarks.md`는 측정 대상, 입력 규모, output schema, 통과 기준, 변동성 전제, 측정된 최소 lower bound, 비보장 항목, `target/bench_acceptance/v1-bench-docs-acceptance.json` artifact path를 명시합니다.
- `docs/v1_acceptance.md`의 gate source는 task handoff 시점의 `autopilot/ssot/current-artifact.md` Launch Gate Evidence Contract와 Evidence Requirements입니다.
- `docs/v1_acceptance.md`는 모든 launch gate row에 gate id, requirement id, evidence path, verification command 또는 manual review evidence, 현재 status를 포함해야 합니다.
- `docs/v1_acceptance.md`는 evidence 없는 항목을 progress projection만으로 완료 처리하지 않고 explicit blocker 또는 out-of-scope reason으로 남겨야 합니다.
- `docs/v1_acceptance.md`는 `evidence-v1-acceptance-docs`를 포함해야 합니다.
- final report는 `evidence-v1-benchmark-lower-bounds`와 `evidence-v1-acceptance-docs`를 포함해야 합니다.
- 새 문서는 V1 단일 프로세스 Rust CLI 경계를 유지하고, 아직 검증되지 않은 성능 또는 기능 완료를 과장하지 않습니다.

## Required Verification Commands
- `scripts/verify`
- `scripts/verify_bench_acceptance`

## Required Gate Rows
- `gate-v1-cli-smoke`: `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`
- `gate-v1-disk-page-storage`: `req-v1-page-storage-restart`, `req-v1-record-format-doc`
- `gate-v1-sql-schema-exec`: `req-v1-sql-exec-examples`
- `gate-v1-indexes`: `req-v1-primary-index-proof`, `req-v1-secondary-index-proof`
- `gate-v1-transactions-wal-recovery`: `req-v1-wal-recovery-proof`
- `gate-v1-crash-testing`: `req-v1-crash-matrix-output`
- `gate-v1-differential-property-tests`: `req-v1-differential-property-proof`
- `gate-v1-db-check-invariants`: `req-v1-db-check-proof`
- `gate-v1-bench-docs-acceptance`: `req-v1-benchmark-lower-bounds`, `req-v1-acceptance-docs`

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
