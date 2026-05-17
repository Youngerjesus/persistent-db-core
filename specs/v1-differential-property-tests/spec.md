# 지원 SQL 부분집합 differential/property 테스트 추가

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-18-04-47-56
- Task ID: task-2026-05-18-04-47-56-v1-differential-property-tests
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 지원 SQL 부분집합 differential/property 테스트 추가
- Artifact: v1-differential-property-tests

## 목표
- V1 artifact에서 SQL 실행, primary index, WAL recovery, crash matrix, invariant check는 검증되었지만, 지원 SQL 부분집합을 생성 시퀀스 기반으로 검증하는 differential/property evidence가 아직 없습니다.

## 지금 해야 하는 이유
- 남은 open requirement 중 `req-v1-differential-property-proof`는 executable test evidence로 좁게 닫을 수 있고, benchmark/docs acceptance보다 먼저 쿼리 correctness의 신뢰도를 높입니다.

## 기대 산출물 변화
- managed repo에 deterministic operation generator, SQLite-backed differential check, seed/failing-case capture, task-specific verification command를 추가하고 `scripts/verify` 통과를 유지합니다.
- SQLite oracle은 필수입니다. `db` 실행 결과는 SQLite 기준 expected result와 비교되어야 하며, 임의 in-memory comparison oracle만으로 대체할 수 없습니다.

## 의도한 변경 대상
- tests/differential_property.rs
- scripts/verify_differential_property
- docs/testing.md
- Cargo.toml
- 확인만 필요한 대상: docs/cli_contract.md

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 2a330122edc833f214ab1727e677afbea0236f56
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: docs/cli_contract.md, Cargo.toml, scripts/verify, src/sql.rs, src/main.rs, src/lib.rs, tests/sql_exec.rs, tests/primary_index.rs, tests/wal_recovery.rs, tests/crash_matrix.rs, tests/db_check.rs

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `artifact_status=open`, open requirement에 `req-v1-differential-property-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-differential-property-tests`는 `status=open`, blocker는 `missing satisfied requirement rows`입니다.
- Current Plan: `gap-v1-differential-property-tests`는 `metric-v1-acceptance-evidence`와 `gate-v1-differential-property-tests`에 매핑됩니다.
- Current Artifact: `req-v1-differential-property-proof`는 deterministic seed capture가 있는 SQLite differential/property test evidence를 요구합니다.
- Managed repo progress: `gap-v1-differential-property-tests`는 `missing_evidence`이며 SQLite differential/property harness가 아직 없습니다.
- Queue Snapshot: active 또는 reserved task가 없습니다.
- Cargo.toml
- src/sql.rs
- src/main.rs
- src/lib.rs
- tests/sql_exec.rs
- tests/primary_index.rs
- tests/wal_recovery.rs
- tests/crash_matrix.rs
- tests/db_check.rs
- docs/cli_contract.md
- docs/sql_subset.md
- docs/v1_spec.md
- work_queue/progress.md
- sql::execute
- sql::execute_select
- sql::execute_select_primary_key
- sql::parse_statements
- main::main
- autopilot/project_manager/tasks/tasks.json: no task with current_plan_gap_id gap-v1-differential-property-tests found by bounded search
- task-2026-05-17-19-38-21-v1-sql-parser-schema-exec
- task-2026-05-17-22-43-31-v1-primary-btree-index
- task-2026-05-17-23-45-17-v1-transaction-wal-recovery
- task-2026-05-18-02-23-10-v1-deterministic-crash-matrix
- task-2026-05-18-03-29-23-v1-db-check-invariants
- autopilot/project_manager/specs: no v1-differential-property-tests package present
- persistent-db-core_worktree/main/specs: no v1-differential-property-tests package tracked

## 범위
- In scope: selected candidate only.
- In scope: 지원 SQL 부분집합에 대한 deterministic operation generator, SQLite-backed expected result oracle, 실패 seed 및 재현 sequence capture, dedicated verification script, verification 문서화.
- In scope: `Cargo.toml`에는 SQLite oracle을 위한 test-only dev-dependency만 추가할 수 있습니다.
- Out of scope: unrelated breadth features.
- Out of scope: 사용자-facing CLI behavior 변경, SQL syntax 확장, query optimizer 변경, network/server behavior, multi-process concurrency, production dependency 추가.
- Out of scope: `docs/cli_contract.md` 변경. 이번 task는 CLI contract를 바꾸지 않는 테스트 harness 작업이며, worker는 CLI contract 변경이 필요하다고 판단되면 구현하지 말고 conflict로 보고해야 합니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `./scripts/verify_differential_property`가 exact task-specific command로 존재하고 성공해야 합니다. 이 script는 repo root 기준에서 실행 가능해야 하며 `cargo test --test differential_property -- --nocapture`를 실행해야 합니다.
- `cargo test --test differential_property -- --nocapture`는 deterministic seed로 생성한 지원 SQL operation sequence를 실행하고, 각 assertion에서 `db` 결과와 SQLite expected result를 비교해야 합니다.
- SQLite oracle 구현은 Rust test-only dev-dependency로 제한합니다. `rusqlite`를 추가할 수 있으며 production dependency로 추가하면 안 됩니다. 외부 `sqlite3` binary는 필수 환경 전제로 삼지 않습니다.
- 최소 SQL subset은 `CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)`, `INSERT INTO kv (id, value) VALUES (?, ?)`, `SELECT * FROM kv`, `SELECT * FROM kv WHERE id = ?` semantics를 모두 포함해야 합니다.
- key domain은 deterministic seed에서 생성한 `i64` 범위의 unique primary key이며, value domain은 seed에서 생성한 ASCII text입니다. 최소 coverage는 seed당 25개 이상 row와 100개 이상 operation입니다.
- generated sequence는 duplicate primary key insert와 missing key lookup을 모두 포함해야 합니다. duplicate key는 `db`와 SQLite가 모두 error로 처리해야 하며, missing key lookup은 빈 결과로 비교해야 합니다.
- `SELECT *`는 ordered scan으로 검증해야 하며 결과 ordering은 `id` 오름차순이어야 합니다. 현재 `db`가 문서화된 ordering을 다르게 보장한다면 worker는 구현을 진행하지 말고 spec conflict로 보고해야 합니다.
- 실패 시 stdout에는 seed, failing operation index, 최소 재현 가능한 operation sequence, SQLite expected rows, `db` actual rows, 재실행 command가 포함되어야 합니다. 최소 재현 가능한 sequence는 같은 seed와 operation prefix로 실패를 재현할 수 있는 가장 짧은 prefix를 의미합니다.
- 실패 evidence artifact를 생성할 경우 위치는 `target/differential_property/failures/<seed>.json`이어야 하며, stdout에도 동일 경로와 재실행 command를 출력해야 합니다. 이 artifact는 generated local evidence이며 durable docs나 SSOT가 아닙니다.
- `./scripts/verify`가 기존 fmt, clippy, full test, help smoke baseline을 계속 통과해야 합니다.
- `docs/testing.md`를 생성하거나 갱신해 `./scripts/verify_differential_property`, seed 재실행 방식, failure evidence 위치를 짧게 문서화해야 합니다. `docs/cli_contract.md`는 변경하지 않아야 합니다.
- 최종 evidence가 `gate-v1-differential-property-tests`와 `req-v1-differential-property-proof`에 명시적으로 매핑되어야 합니다.

## 검증 계획
- Required commands:
  - `./scripts/verify`
  - `./scripts/verify_differential_property`
- `./scripts/verify_differential_property` 성공 기준: repo root 외부 cwd에서도 실행 가능하고, `cargo test --test differential_property -- --nocapture`가 deterministic seed suite를 통과하며, 실패 시 위 failure evidence contract를 출력합니다.
- 기대 증거: 두 command의 stdout/stderr, scheduler terminal result, final report의 `gate-v1-differential-property-tests` 및 `req-v1-differential-property-proof` 매핑.

## 리스크 및 에스컬레이션
- 알려진 리스크: SQLite test-only dev-dependency 도입이 repo policy와 충돌한다고 판단되면 worker는 임의 oracle로 우회하지 말고 dependency conflict로 escalate해야 합니다.
- 알려진 리스크: 현재 `db`의 `SELECT *` ordering 보장이 `id` 오름차순과 다르면 worker는 test expectation을 낮추지 말고 spec conflict로 보고해야 합니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
