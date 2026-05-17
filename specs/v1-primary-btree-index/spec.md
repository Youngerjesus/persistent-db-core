# Primary B-tree 인덱스 기반 행 조회와 정렬 스캔

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-17-22-43-31
- Task ID: task-2026-05-17-22-43-31-v1-primary-btree-index
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: Primary B-tree 인덱스 기반 행 조회와 정렬 스캔
- Artifact: v1-primary-btree-index

## 목표
- 현재 `db exec`는 durable storage 위에서 기본 SQL schema/insert/select를 수행하지만 primary key 기반 lookup과 ordered traversal evidence가 없습니다. V1 query correctness는 단순 full scan 예시만으로는 부족하며, persisted row를 대상으로 한 primary index proof가 필요합니다.

## 지금 해야 하는 이유
- Root Progress Projection에서 CLI, disk page storage, SQL schema/exec gate는 `projected_complete`이고, `gate-v1-indexes`는 `req-v1-primary-index-proof`와 `req-v1-secondary-index-proof`가 open입니다. current plan도 `gap-v1-primary-btree-index`를 high priority로 두며 primary key insert/find/scan tests with deterministic ordering을 다음 slice로 제시합니다.

## 기대 산출물 변화
- managed repo에 persisted row 기반 primary B-tree index primitive와 SQL/storage integration proof를 추가하고, restart 후 primary lookup과 ordered scan이 deterministic하게 유지됨을 tests와 docs로 증명합니다.

## 의도한 변경 대상
- src/index.rs
- src/lib.rs
- src/main.rs
- src/storage.rs
- tests/primary_index.rs
- tests/sql_exec.rs
- docs/file_format.md
- docs/sql_subset.md
- docs/cli_contract.md

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 847a4bcdcc2c265452ddfdf01a23584301586728
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/lib.rs, src/main.rs, src/storage.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md, AGENTS.md, work_queue/progress.md, docs/history_archives/history.md, .codex/agents/decision-brake-readiness-reviewer.toml, .codex/agents/project-reviewer.toml, .codex/agents/task-master.toml

## Risk flags
- persisted_index_compatibility_note_required

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `artifact_status=open`이며 open requirement에 `req-v1-primary-index-proof`와 `req-v1-secondary-index-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-cli-smoke`, `gate-v1-disk-page-storage`, `gate-v1-sql-schema-exec`는 `projected_complete`로 완료 slice에서 제외됩니다.
- Current Plan: `gap-v1-primary-btree-index`는 high priority이고 linked metric은 `metric-v1-query-correctness`, linked artifact gate는 `gate-v1-indexes`입니다.
- Current Plan Gap Details: primary key index는 insert, find, ordered scan, deterministic persistence, query path integration evidence를 요구합니다.
- Managed repo progress: minimal SQL schema/execute path가 완료되었고 다음 작은 handoff는 indexing, recovery, validation gap 중 하나라고 기록되어 있습니다.
- Queue Snapshot: `[]`로 active/reserved duplicate task가 없습니다.
- Active Managed Repo Snapshot: git status가 clean이고 최근 history는 2026-05-17 minimal SQL milestone 완료를 기록합니다.
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- persistent-db-core_worktree/main/src/lib.rs
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/src/sql.rs
- persistent-db-core_worktree/main/src/storage.rs
- persistent-db-core_worktree/main/tests/sql_exec.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/sql_subset.md
- persistent-db-core_worktree/main/docs/file_format.md
- persistent-db-core_worktree/main/docs/v1_spec.md
- gap-v1-primary-btree-index
- gate-v1-indexes
- req-v1-primary-index-proof
- req-v1-secondary-index-proof
- PageStore
- PageStore::append_record
- PageStore::read_records
- sql::execute
- Database::from_records
- execute_insert
- execute_select
- Statement::SelectAll
- autopilot/project_manager/tasks/tasks.json#task-2026-05-15-16-06-54-v1-bootstrap-cli-contract
- autopilot/project_manager/tasks/tasks.json#task-2026-05-16-13-58-47-v1-page-storage-record-format
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-19-38-21-v1-sql-parser-schema-exec
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/spec.md
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/contracts.md

## 범위
- In scope: `db exec`의 primary key SQL slice, persisted row 기반 primary index 구성, exact primary key lookup, primary key 정렬 scan, restart 재구축 검증, 관련 durable docs 업데이트.
- Out of scope: secondary index, query optimizer, `ORDER BY`, range predicate, multi-column primary key, non-INT primary key, network server, multi-process concurrency, 별도 persisted index metadata 파일 또는 background index rebuild process.
- 이 작업은 non-visual CLI/storage task입니다. Browser, DOM capture, screenshot, rendered route, UX design review evidence는 completion evidence가 아닙니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- SQL grammar는 이 slice에서 단일 `INT PRIMARY KEY` column 선언과 exact primary key predicate만 추가합니다. 예: `CREATE TABLE users (id INT PRIMARY KEY, name TEXT);`, `SELECT * FROM users WHERE id = 2;`.
- Primary key가 있는 table의 `SELECT * FROM <table>;` 결과는 successful insert order가 아니라 primary key 오름차순 ordered scan이어야 합니다. Primary key가 없는 기존 table의 `SELECT *` insert-order behavior는 유지해야 합니다.
- Primary index는 persisted SQL row records를 대상으로 insert, exact key lookup, ordered scan을 deterministic하게 지원해야 합니다. 별도 persisted index metadata는 이 slice의 non-goal이며, process restart 또는 reopen 시 index는 durable row records에서 재구축해야 합니다.
- 데이터베이스를 닫고 다시 연 뒤에도 primary index lookup과 ordered scan 결과가 동일해야 합니다.
- SQL execution path가 primary index를 사용한다는 검증 evidence가 있어야 합니다. 최소 evidence는 `tests/primary_index.rs`의 index primitive/rebuild tests, `tests/sql_exec.rs`의 primary-key SQL behavior tests, final report의 query path mapping입니다.
- 중복 primary key, 누락 key 조회, 빈 테이블 scan 같은 negative/edge case가 deterministic test로 덮여야 합니다.
- `docs/file_format.md`, `docs/sql_subset.md`, `docs/cli_contract.md`에는 primary index persistence model을 반드시 기록해야 합니다. 문서는 별도 index metadata를 저장하지 않는지, 기존 row-only SQL database file이 어떻게 호환되는지, row records에서 index를 재구축하는지, corrupt SQL row record가 기존 invalid SQL storage record error로 실패하는지를 설명해야 합니다. Missing index metadata failure mode는 별도 index metadata가 없으므로 이 slice의 non-goal입니다.
- `./scripts/verify`, `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`가 통과해야 합니다.

## SQL 관찰 가능 행동 계약
- 공통 전제: 아래 예시는 `db exec <path> <sql>`의 단일 command invocation 기준입니다. 성공 command는 stderr가 비어 있어야 하며 exit code는 `0`이어야 합니다.
- Exact lookup setup:
  - SQL: `CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal');`
  - expected stdout: empty
  - expected stderr: empty
  - expected exit code: `0`
- Exact lookup:
  - SQL: `SELECT * FROM users WHERE id = 2;`
  - expected stdout: `id|name\n2|bea\n`
  - expected stderr: empty
  - expected exit code: `0`
- Ordered scan:
  - SQL: `SELECT * FROM users;`
  - expected stdout: `id|name\n1|ada\n2|bea\n3|cal\n`
  - expected stderr: empty
  - expected exit code: `0`
- Missing key lookup:
  - SQL: `SELECT * FROM users WHERE id = 9;`
  - expected stdout: `id|name\n`
  - expected stderr: empty
  - expected exit code: `0`
- Duplicate primary key:
  - SQL: `INSERT INTO users VALUES (2, 'dupe');`
  - expected stdout: empty
  - expected stderr: `error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n`
  - expected exit code: `2`
- Empty primary-key table scan:
  - SQL: `CREATE TABLE empty_users (id INT PRIMARY KEY, name TEXT); SELECT * FROM empty_users;`
  - expected stdout: `id|name\n`
  - expected stderr: empty
  - expected exit code: `0`

## 검증 계획
- Required commands:
  - `./scripts/verify`
  - `cargo test --test primary_index`
  - `cargo test --test sql_exec primary_key`
- Evidence mapping:
  - `./scripts/verify`: repo baseline, formatting, clippy, full tests, help smoke contract를 증명합니다.
  - `cargo test --test primary_index`: primary B-tree insert, exact lookup, ordered traversal, restart/rebuild, duplicate key, missing key, corrupt row handling을 증명합니다.
  - `cargo test --test sql_exec primary_key`: `db exec` stdout, stderr, exit code, primary key ordering, missing key, duplicate key, empty table scan의 관찰 가능 계약을 증명합니다.
- Expected evidence paths: scheduler run의 `tasks/<task_id>/runs/<run_id>/result.md`, `tasks/<task_id>/runs/<run_id>/final.md`, final report의 command output summary와 acceptance criterion mapping.
- Visual verification evidence는 요구하지 않습니다. 이 artifact에는 reference bundle이 없으며, CLI/storage behavior가 completion source입니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
