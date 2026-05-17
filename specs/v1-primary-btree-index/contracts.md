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
- 생성 대상 코드 또는 문서: Primary B-tree 인덱스 기반 행 조회와 정렬 스캔에 대한 closure.
- 생성 대상 테스트 또는 verification output: `./scripts/verify`, `cargo test --test primary_index`, `cargo test --test sql_exec primary_key` command output과 scheduler terminal result.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- 이 작업은 non-visual CLI/storage task입니다. Browser evidence, DOM capture, screenshot, rendered route, UX design review는 acceptance evidence로 요구하지 않습니다.
- SQL grammar는 단일 `INT PRIMARY KEY` column 선언과 exact primary key predicate를 지원해야 합니다. 예: `CREATE TABLE users (id INT PRIMARY KEY, name TEXT);`, `SELECT * FROM users WHERE id = 2;`.
- Primary key가 있는 table의 `SELECT * FROM <table>;`는 primary key 오름차순으로 row를 출력해야 합니다. Primary key가 없는 기존 table의 insert-order `SELECT *` behavior는 유지해야 합니다.
- Primary index는 persisted rows에 대해 insert, exact key lookup, ordered scan을 deterministic하게 지원해야 합니다.
- 데이터베이스를 닫고 다시 연 뒤에도 primary index lookup과 ordered scan 결과가 동일해야 합니다.
- 별도 persisted index metadata는 이 slice에서 만들지 않습니다. Process restart 또는 reopen 시 primary index는 durable SQL row records에서 재구축해야 하며, 기존 row-only SQL database file은 그대로 읽혀야 합니다.
- SQL execution path가 primary index를 사용한다는 검증 evidence가 있어야 합니다. 최소 evidence는 `tests/primary_index.rs`, `tests/sql_exec.rs`, final report의 query path mapping입니다.
- 중복 primary key, 누락 key 조회, 빈 테이블 scan 같은 negative/edge case가 deterministic test로 덮여야 합니다.
- `docs/file_format.md`, `docs/sql_subset.md`, `docs/cli_contract.md`에는 primary index persistence model을 반드시 기록해야 합니다. 문서는 별도 index metadata를 저장하지 않는지, 기존 row-only SQL database file이 어떻게 호환되는지, row records에서 index를 재구축하는지, corrupt SQL row record가 기존 invalid SQL storage record error로 실패하는지를 설명해야 합니다. Missing index metadata failure mode는 별도 index metadata가 없으므로 이 slice의 non-goal입니다.
- `./scripts/verify`, `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`가 통과해야 합니다.

## SQL Observable Contract
- Exact lookup setup command:
  - SQL: `CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal');`
  - expected stdout: empty
  - expected stderr: empty
  - expected exit code: `0`
- Exact lookup command:
  - SQL: `SELECT * FROM users WHERE id = 2;`
  - expected stdout: `id|name\n2|bea\n`
  - expected stderr: empty
  - expected exit code: `0`
- Ordered scan command:
  - SQL: `SELECT * FROM users;`
  - expected stdout: `id|name\n1|ada\n2|bea\n3|cal\n`
  - expected stderr: empty
  - expected exit code: `0`
- Missing key lookup command:
  - SQL: `SELECT * FROM users WHERE id = 9;`
  - expected stdout: `id|name\n`
  - expected stderr: empty
  - expected exit code: `0`
- Duplicate primary key command:
  - SQL: `INSERT INTO users VALUES (2, 'dupe');`
  - expected stdout: empty
  - expected stderr: `error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n`
  - expected exit code: `2`
- Empty primary-key table scan command:
  - SQL: `CREATE TABLE empty_users (id INT PRIMARY KEY, name TEXT); SELECT * FROM empty_users;`
  - expected stdout: `id|name\n`
  - expected stderr: empty
  - expected exit code: `0`

## Verification Evidence Contract
- `./scripts/verify`는 repo baseline, formatting, clippy, full tests, help smoke contract 증거입니다.
- `cargo test --test primary_index`는 primary B-tree insert, exact lookup, ordered traversal, restart/rebuild, duplicate key, missing key, corrupt row handling 증거입니다.
- `cargo test --test sql_exec primary_key`는 `db exec` stdout, stderr, exit code, primary key ordering, missing key, duplicate key, empty table scan 증거입니다.
- Expected evidence paths는 scheduler run의 `tasks/<task_id>/runs/<run_id>/result.md`, `tasks/<task_id>/runs/<run_id>/final.md`, final report의 command output summary입니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
