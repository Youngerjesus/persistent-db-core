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
- 생성 대상 코드 또는 문서: 보조 인덱스 생성 및 범위 스캔 검증에 대한 closure.
- 생성 대상 테스트 또는 verification output: `scripts/verify`, `cargo test --test secondary_index -- --nocapture`, secondary index path use evidence, persisted format compatibility evidence.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `db exec`가 `CREATE INDEX <name> ON <table>(<integer_column>)`를 지원한다. 성공한 `CREATE INDEX`는 exit code `0`, empty stdout, empty stderr를 가져야 한다.
- `db exec`는 missing table, missing column, unsupported type, duplicate index를 SQL semantic error로 처리해야 한다. 각 오류는 exit code `2`, empty stdout, deterministic stderr prefix와 hint를 가져야 한다.
- `CREATE INDEX` 오류 예제는 exact stderr를 고정해야 한다. Missing table은 `error: SQL semantic error: table not found: missing\nhint: create the table before INSERT, SELECT, or CREATE INDEX.\n`, missing column은 `error: SQL semantic error: column not found for index idx_users_age: age\nhint: create the index on an existing table column.\n`, unsupported type은 `error: SQL semantic error: secondary index column must be INT: name\nhint: this SQL slice supports secondary indexes only on INT columns.\n`, duplicate index는 `error: SQL semantic error: index already exists: idx_users_age\nhint: use a new index name for CREATE INDEX in this database.\n`이어야 한다.
- `docs/cli_contract.md`는 `CREATE INDEX`, indexed equality predicate, `BETWEEN` inclusive bounded range scan, 성공 stdout/stderr/exit code, 오류 stdout/stderr/exit code, ordering/tie-break 규칙을 기록해야 한다.
- `docs/file_format.md`는 secondary index metadata/storage encoding, 기존 no-index database compatibility, backfill behavior, `db check` secondary index consistency validation을 기록해야 한다.
- Focused tests는 기존 no-index database reopen, 기존 row 대상 `CREATE INDEX` backfill, index 생성 후 insert된 row의 equality/range 조회, process reopen 후 동일 equality/range 동작을 검증해야 한다.
- Equality example은 `CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);`, rows `(3, 20, 'cal')`, `(1, 10, 'ada')`, `(2, 20, 'bea')`, `CREATE INDEX idx_users_age ON users(age);`, `SELECT * FROM users WHERE age = 20;`를 포함하고 stdout `id|age|name\n2|20|bea\n3|20|cal\n`을 검증해야 한다.
- Range example은 같은 fixture의 `SELECT * FROM users WHERE age BETWEEN 10 AND 20;`를 포함하고 stdout `id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n`을 검증해야 한다.
- Ordering은 secondary index key ascending, tie-break ascending으로 고정한다. Primary key가 있으면 tie-break는 primary key이고, primary key가 없으면 durable row insertion order이다.
- Secondary indexed equality와 `BETWEEN` bounded range scan은 실제 secondary index path를 사용해야 한다. Full table scan 결과 일치만으로는 acceptance를 충족할 수 없으며, focused integration test, planner/path observable evidence, `db check` invariant evidence, 또는 내부 테스트 evidence 중 하나가 index path use를 명시해야 한다.
- Required verification commands:
  - `scripts/verify`
  - `cargo test --test secondary_index -- --nocapture`
- Required evidence path 또는 report location:
  - `final_review.md` 또는 scheduler final report는 `REQ-7-create-index-must-create-disk-3b71a7dc`에 대해 CLI examples, index path use evidence, persisted format compatibility evidence, `db check` evidence, 두 required command의 stdout/stderr와 exit code를 매핑해야 한다.
- 이 task는 non-visual CLI/database task이다. DOM capture, rendered route state, screenshot, UX design review evidence는 acceptance evidence가 아니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
