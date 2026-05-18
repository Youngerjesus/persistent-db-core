# 보조 인덱스 생성 및 범위 스캔 검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-19-01-26-09
- Task ID: task-2026-05-19-01-26-09-v1-secondary-index-range-scan
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 보조 인덱스 생성 및 범위 스캔 검증
- Artifact: v1-secondary-index-range-scan

## 목표
- V1 artifact의 `gate-v1-indexes`는 primary index evidence만으로 닫을 수 없으며, `CREATE INDEX` 기반 disk-backed secondary index와 range scan proof가 아직 명시 completion target으로 남아 있다.

## 지금 해야 하는 이유
- storage, SQL execution, primary index, WAL recovery, crash matrix, `db check`, differential/property, benchmark/docs baseline이 이미 supporting evidence로 존재하고 queue도 비어 있다. 남은 구현 gap 중 보조 인덱스는 current objective의 query correctness를 직접 올리는 가장 작은 coherent slice다.

## 기대 산출물 변화
- managed repo에 secondary index metadata/storage, indexed equality/range query path evidence, restart persistence tests, black-box CLI examples, 필요한 문서 업데이트를 추가하되 `gate-v1-indexes`의 sibling requirement completion은 자동 추론하지 않는다.

## 의도한 변경 대상
- src/
- tests/secondary_index.rs
- tests/sql_exec.rs
- docs/file_format.md
- docs/cli_contract.md

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: ddff99d9f8f89ed69aca56a436693ccd5870b4cb
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/check.rs, src/index.rs, src/lib.rs, src/main.rs, src/sql.rs, src/storage.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md, work_queue/progress.md, AGENTS.md

## Risk flags
- persisted_format_compatibility_tests_required
- shared_gate_partial_slice_only
- do_not_close_primary_or_mutation_sibling_requirements
- stale_existing_evidence_must_not_be_reused_as_completion

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- `ssot/current-plan.md`는 `gap-v1-secondary-index-range-scan`을 `metric-v1-query-correctness`, `gate-v1-indexes`, expected target `REQ-7-create-index-must-create-disk-3b71a7dc`에 매핑한다.
- `ssot/current-artifact.md`의 `REQ-7-create-index-must-create-disk-3b71a7dc`는 disk-backed secondary indexes, equality lookup, bounded range scan, ordering, persistence, query path use를 요구한다.
- managed repo `work_queue/progress.md`는 `gap-v1-secondary-index-range-scan`을 `missing_evidence`로 두고, 다음 작은 handoff로 secondary indexes 또는 좁은 acceptance blocker를 제안한다.
- managed repo history는 storage, SQL execution, primary index, WAL recovery, crash matrix, `db check`, differential/property, benchmark/docs baseline이 이미 추가되었음을 기록한다.
- Queue Snapshot은 `[]`이고 Git Status는 `clean`이므로 active duplicate나 dirty worktree blocker가 없다.
- autopilot/ssot/current-plan.md:36
- autopilot/ssot/current-artifact.md:54
- persistent-db-core_worktree/main/src/sql.rs:93
- persistent-db-core_worktree/main/src/sql.rs:686
- persistent-db-core_worktree/main/src/index.rs:4
- persistent-db-core_worktree/main/docs/v1_acceptance.md:17
- persistent-db-core_worktree/main/docs/cli_contract.md:198
- persistent-db-core_worktree/main/work_queue/progress.md:15
- Statement::SelectPrimaryKey
- parse_select
- execute_select_primary_key
- PrimaryIndex
- PrimaryIndex::get
- PrimaryIndex::ordered_positions
- autopilot/project_manager/tasks/tasks.json:666
- autopilot/project_manager/tasks/tasks.json:706
- autopilot/project_manager/tasks/tasks.json:728
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/spec.md:93
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/analysis_report.md:7
- persistent-db-core_worktree/main/specs/v1-primary-btree-index/final_review.md:10

## 범위
- In scope: selected candidate only.
- Out of scope: unrelated breadth features.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `db exec`가 `CREATE INDEX <name> ON <table>(<integer_column>)`를 지원한다. 성공한 `CREATE INDEX`는 exit code `0`, empty stdout, empty stderr를 유지해야 하며, `docs/cli_contract.md`와 `docs/file_format.md`는 새 SQL surface와 persisted secondary-index metadata/storage compatibility note를 기록해야 한다.
- `db exec`는 missing table, missing column, unsupported type, duplicate index를 SQL semantic error로 처리해야 한다. 각 오류는 exit code `2`, empty stdout, clear stderr를 가져야 하며, integration test는 정확한 stderr prefix와 hint를 고정해야 한다.
- `CREATE INDEX` 오류 예제는 exact stderr를 고정해야 한다. Missing table은 `error: SQL semantic error: table not found: missing\nhint: create the table before INSERT, SELECT, or CREATE INDEX.\n`, missing column은 `error: SQL semantic error: column not found for index idx_users_age: age\nhint: create the index on an existing table column.\n`, unsupported type은 `error: SQL semantic error: secondary index column must be INT: name\nhint: this SQL slice supports secondary indexes only on INT columns.\n`, duplicate index는 `error: SQL semantic error: index already exists: idx_users_age\nhint: use a new index name for CREATE INDEX in this database.\n`이어야 한다.
- Black-box 또는 integration scenario는 다음 성공 예제를 포함해야 한다. `CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);`, `INSERT INTO users VALUES (3, 20, 'cal');`, `INSERT INTO users VALUES (1, 10, 'ada');`, `INSERT INTO users VALUES (2, 20, 'bea');`, `CREATE INDEX idx_users_age ON users(age);`, `SELECT * FROM users WHERE age = 20;`의 stdout은 `id|age|name\n2|20|bea\n3|20|cal\n`이어야 한다.
- `BETWEEN` bounded range scan은 inclusive boundary를 가져야 한다. 같은 fixture에서 `SELECT * FROM users WHERE age BETWEEN 10 AND 20;`의 stdout은 `id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n`이어야 한다.
- Secondary indexed INTEGER column의 equality predicate와 `BETWEEN` bounded range scan은 실제 secondary index query path를 사용해야 한다. Full table scan으로 결과만 맞춘 구현은 통과할 수 없으며, focused integration test, planner/path observable evidence, `db check` invariant evidence, 또는 내부 테스트 evidence 중 하나가 index path 사용을 명시적으로 검증해야 한다.
- 동일 secondary index key의 tie-break는 primary key가 있으면 ascending primary-key order를 사용하고, primary key가 없으면 durable row insertion order를 사용한다. 전체 결과 ordering은 secondary index key ascending, tie-break ascending 순서로 고정한다.
- Persisted format compatibility test는 기존 no-index database reopen, 기존 row 대상 `CREATE INDEX` backfill, index 생성 후 insert된 row의 equality/range 조회, process reopen 후 동일 equality/range 동작을 모두 검증해야 한다.
- `db check <path>`는 secondary index metadata와 index contents가 durable table rows와 불일치할 때 deterministic invariant failure를 보고해야 하며, 성공 경로는 기존 `ok: db check passed` stdout 계약을 유지해야 한다.
- 기존 primary index와 row-only table behavior는 regress되지 않아야 하며, 이번 evidence는 `REQ-7-create-index-must-create-disk-3b71a7dc`에만 명시 매핑한다.

## 검증 계획
- Required commands:
  - `scripts/verify`
  - `cargo test --test secondary_index -- --nocapture`
- 기대 증거:
  - `final_review.md` 또는 scheduler final report에 두 command의 stdout/stderr와 exit code가 기록되어야 한다.
  - `final_review.md` 또는 scheduler final report는 `REQ-7-create-index-must-create-disk-3b71a7dc`에 대해 CLI examples, index path use evidence, persisted format compatibility evidence, `db check` evidence를 각각 매핑해야 한다.
  - 이 task는 non-visual CLI/database task이므로 DOM capture, rendered route state, screenshot, UX design review evidence를 요구하지 않는다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
