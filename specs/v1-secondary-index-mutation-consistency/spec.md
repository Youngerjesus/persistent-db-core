# 보조 인덱스 mutation 일관성 검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-19-03-12-23
- Task ID: task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 보조 인덱스 mutation 일관성 검증
- Artifact: v1-secondary-index-mutation-consistency

## 목표
- 현재 secondary index는 `CREATE INDEX`, equality/range scan, reopen/WAL replay, `db check` 검증까지 진척됐지만, artifact 계약의 `REQ-7-insert-update-and-delete-must-997871f9`는 mutation 이후 table row와 primary/secondary index가 함께 유지되는 증거를 요구합니다.

## 지금 해야 하는 이유
- managed repo progress가 다음 최소 handoff로 mutation-maintained secondary-index behavior를 지목하고 있고, queue가 비어 있으며, 직전 secondary-index milestone 직후에 같은 코드 표면에서 가장 작게 닫을 수 있는 남은 index gate slice입니다.

## 기대 산출물 변화
- secondary indexed column 또는 indexed row에 대한 UPDATE/DELETE 시나리오를 추가하고, 재시작 후 equality/range query와 `db check`가 stale index entry, missing indexed row, dangling pointer를 잡는 deterministic test evidence를 남깁니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/secondary_index.rs
- tests/sql_exec.rs
- tests/db_check.rs
- docs/cli_contract.md

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 12731d4424d199c40d05d611c077a9be30b96ece
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, src/storage.rs, tests/secondary_index.rs, tests/sql_exec.rs, tests/db_check.rs, docs/cli_contract.md, work_queue/progress.md, docs/history_archives/history.md, src/sql.rs

## Risk flags
- index_consistency_regression_risk
- requires_no_storage_format_change_or_explicit_compat_note
- requires_restart_and_wal_replay_coverage
- requires_db_check_negative_fixture

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- `work_queue/progress.md` Current State는 다음 최소 handoff가 mutation-maintained secondary-index behavior 또는 더 좁은 acceptance blocker라고 기록합니다.
- `docs/history_archives/history.md`의 2026-05-19 항목은 secondary index가 CREATE INDEX, equality/range scan, reopen/WAL replay, db check invariant validation까지 추가됐다고 기록하지만 UPDATE/DELETE mutation 유지 증거는 명시하지 않습니다.
- `ssot/current-artifact.md`는 `REQ-7-insert-update-and-delete-must-997871f9`와 `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`를 `gate-v1-indexes`의 source-required evidence로 요구합니다.
- Root Progress Projection은 `gate-v1-indexes`를 `open`으로 두고 해당 requirement rows를 missing으로 계산합니다.
- Queue Snapshot은 빈 배열이며 active/reserved task 중 같은 feature_slug 후보와 충돌하는 항목이 없습니다.
- src/sql.rs
- src/check.rs
- tests/secondary_index.rs
- docs/cli_contract.md
- docs/sql_subset.md
- docs/v1_acceptance.md
- work_queue/progress.md
- docs/history_archives/history.md
- specs/v1-secondary-index-range-scan/plan.md
- specs/v1-secondary-index-range-scan/final_review.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- Statement::CreateIndex
- Statement::Insert
- Statement::SelectAll
- Statement::SelectWhere
- parse_statement
- execute_insert
- validate_secondary_indexes
- QueryPath::SecondaryIndexEquality
- QueryPath::SecondaryIndexRange
- check_database
- autopilot/project_manager/tasks/tasks.json:533 task-2026-05-19-01-26-09-v1-secondary-index-range-scan SUCCESS
- autopilot/project_manager/tasks/tasks.json:565-570 기존 secondary-index task는 후보 요구사항을 future_requirement_ids로 둠
- autopilot/project_manager/tasks/tasks.json:576-578 기존 secondary-index task의 artifact_requirement_ids는 REQ-7-create-index-must-create-disk-3b71a7dc만 포함
- autopilot/project_manager/tasks/task_status_events.jsonl:11 task-2026-05-19-01-26-09-v1-secondary-index-range-scan SUCCESS
- specs/v1-secondary-index-range-scan/plan.md:6-9 UPDATE, DELETE, mutation beyond INSERT는 non-goal
- specs/v1-secondary-index-range-scan/final_review.md:5 기존 closure는 REQ-7-create-index-must-create-disk-3b71a7dc로 제한
- specs/v1-secondary-index-range-scan/final_review.md:18-19 update/delete maintenance와 broader gate completion은 inference로 닫지 않음
- autopilot/project_manager/specs/v1-secondary-index-range-scan/spec.md:96 기존 evidence는 REQ-7-create-index-must-create-disk-3b71a7dc에만 명시 매핑

## 범위
- 범위 포함: secondary indexed column `UPDATE`, secondary indexed row `DELETE`, restart/WAL replay 후 index consistency, `db check` secondary-index invariant 검증, 필요한 CLI/SQL 문서 갱신.
- 범위 제외: unrelated breadth features, network server, multi-process concurrency, distributed storage, query optimization, visual/UI evidence.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- 고정 fixture는 다음 schema와 seed rows를 사용합니다.

```sql
CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);
INSERT INTO users VALUES (1, 10, 'ada');
INSERT INTO users VALUES (2, 20, 'bea');
INSERT INTO users VALUES (3, 20, 'cal');
INSERT INTO users VALUES (4, 30, 'dia');
CREATE INDEX idx_users_age ON users(age);
```

- `db exec <path> "UPDATE users SET age = 30 WHERE id = 2;"`는 exit `0`, empty stdout, empty stderr여야 합니다. 이후 별도 `db exec` process invocation에서 `SELECT * FROM users WHERE age = 20;`은 이전 key에서 update된 row를 제외하고 다음 stdout만 출력해야 합니다.

```text
id|age|name
3|20|cal
```

- 같은 UPDATE 이후 `SELECT * FROM users WHERE age = 30;`은 새 key equality scan 결과를 secondary key와 primary key tie-break 순서로 다음 stdout만 출력해야 합니다.

```text
id|age|name
2|30|bea
4|30|dia
```

- 같은 UPDATE 이후 `SELECT * FROM users WHERE age BETWEEN 20 AND 30;`은 inclusive range scan 결과를 secondary key ascending, primary key ascending 순서로 다음 stdout만 출력해야 합니다.

```text
id|age|name
3|20|cal
2|30|bea
4|30|dia
```

- 같은 UPDATE 이후 `SELECT * FROM users WHERE id = 2;`는 primary-key lookup으로 다음 stdout만 출력해야 합니다.

```text
id|age|name
2|30|bea
```

- 같은 UPDATE 이후 `SELECT * FROM users;`는 table scan을 primary key ascending 순서로 다음 stdout만 출력해야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
3|20|cal
4|30|dia
```

- 이어서 `db exec <path> "DELETE FROM users WHERE id = 3;"`는 exit `0`, empty stdout, empty stderr여야 합니다. 이후 별도 `db exec` process invocation에서 `SELECT * FROM users WHERE age = 20;`은 header-only stdout을 출력해야 합니다.

```text
id|age|name
```

- 같은 DELETE 이후 `SELECT * FROM users WHERE age BETWEEN 10 AND 30;`은 삭제 row를 제외하고 다음 stdout만 출력해야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
4|30|dia
```

- 같은 DELETE 이후 `SELECT * FROM users WHERE id = 3;`은 primary-key lookup에서 header-only stdout을 출력해야 합니다.

```text
id|age|name
```

- 같은 DELETE 이후 `SELECT * FROM users;`는 table scan에서 삭제 row를 제외하고 다음 stdout만 출력해야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
4|30|dia
```

- restart/reopen 증거는 setup, UPDATE, DELETE, query, `db check`를 각각 별도 `db` process invocation으로 실행해 남겨야 합니다. UPDATE와 DELETE 이후 프로세스를 완전히 종료하고 재오픈해도 table row, primary index, secondary index 상태가 위 stdout과 일치해야 하며 `db check <path>`는 exit `0`, stdout `ok: db check passed\n`, empty stderr여야 합니다.
- WAL replay 증거는 page file과 `<path>.wal` sidecar가 존재하는 상태에서 별도 process invocation으로 reopen query와 `db check <path>`를 실행해 남겨야 합니다. retained complete WAL frame을 replay해야 하는 경우에도 위 stdout과 `db check` 결과가 동일해야 합니다.
- `db check` negative fixture는 checked-in binary fixture 대신 `tests/secondary_index.rs` 또는 가장 가까운 test file의 deterministic fixture builder로 생성할 수 있습니다. fixture builder는 stale secondary entry, dangling row pointer, missing indexed visible row를 각각 독립 case로 만들어야 합니다.
- stale secondary entry fixture는 visible row `id=2`가 `age=30`으로 update된 뒤 `idx_users_age`에 old key `20` entry가 남은 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- dangling row pointer fixture는 committed secondary index entry가 존재하지 않는 row position 또는 deleted row를 참조하는 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- missing indexed visible row fixture는 visible row `id=4, age=30` 또는 동등한 visible indexed row에 대응하는 committed secondary entry가 없는 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- storage format을 바꾸지 않는 경우 final report에 `no storage format change`와 기존 secondary-index file compatibility 확인을 기록해야 합니다. UPDATE/DELETE durable record를 추가해 storage format을 바꾸는 경우 `docs/file_format.md`, `docs/sql_subset.md`, `docs/cli_contract.md`를 갱신하고 기존 row-only 및 existing secondary-index database reopen compatibility note를 final report에 남겨야 합니다.
- completion evidence는 `REQ-7-insert-update-and-delete-must-997871f9` 및 `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`를 명시해야 합니다.

## 검증 계획
- 필수 baseline command: `./scripts/verify`
- 필수 focused command: `cargo test --test secondary_index -- --nocapture`
- 필요한 경우 `db check` negative fixture coverage가 분리되어 있으면 추가 focused command로 `cargo test --test db_check -- --nocapture`를 실행합니다.
- final report 또는 scheduler run artifact는 실행한 command, exit status, stdout/stderr 요약, 관련 test name, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` 매핑을 포함해야 합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
