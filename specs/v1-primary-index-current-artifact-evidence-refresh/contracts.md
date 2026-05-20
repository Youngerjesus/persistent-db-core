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
- 생성 대상 코드 또는 문서: Primary index current-artifact evidence refresh에 대한 closure.
- 생성 대상 테스트 또는 verification output: `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify`의 current managed repo SHA 기준 통과 증거.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.
- 생성 대상 evidence path: `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`, `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`, 필요 시 `docs/v1_acceptance.md`.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `artifact_requirement_ids`는 `REQ-7-implement-integer-primary-key-as-9c698e08`만 명시하고 `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` 완료를 주장하지 않습니다.
- current managed repo SHA에서 `cargo test --test primary_index`가 통과해야 합니다.
- `primary_index` evidence는 `PrimaryIndex::insert`의 `2 -> 0`, `1 -> 1` 삽입, `get(2) == Some(0)`, `get(1) == Some(1)`, `get(3) == None`, duplicate `insert(2, 99)` 오류, duplicate 후 기존 `get(2) == Some(0)` 유지, `ordered_positions()`의 `[1, 2, 0]`, empty ordered positions `[]`를 검증해야 합니다.
- `primary_index` evidence는 persisted SQL rows에서 reopen/rebuild된 primary index가 `SELECT * FROM users WHERE id = 2;`에 대해 stdout `id|name\n2|bea\n`, exit code `0`, 비어 있는 stderr를 반환하고, `SELECT * FROM users;`에 대해 stdout `id|name\n1|ada\n2|bea\n3|cal\n`를 반환함을 검증해야 합니다.
- current managed repo SHA에서 `cargo test --test sql_exec primary_key`가 통과해야 합니다.
- `sql_exec primary_key` evidence는 `CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal'); SELECT * FROM users; SELECT * FROM users WHERE id = 2; SELECT * FROM users WHERE id = 9;` 입력의 exit code `0`, 비어 있는 stderr, stdout `id|name\n1|ada\n2|bea\n3|cal\nid|name\n2|bea\nid|name\n`를 검증해야 합니다.
- `sql_exec primary_key` evidence는 같은 database path를 새 `db exec` process에서 reopen한 뒤 `SELECT * FROM users;`와 `SELECT * FROM users WHERE id = 2;` 결과가 동일한 ordering과 exact lookup을 유지함을 검증해야 합니다.
- duplicate primary key 입력 `INSERT INTO users VALUES (2, 'dupe');`는 exit code `2`, 비어 있는 stdout, stderr `error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n`를 반환하고 기존 row를 변경하지 않아야 합니다.
- persisted duplicate primary-key row 재개방 fixture는 유효한 SQL storage catalog record와 같은 table의 유효한 SQL row record 두 개를 사용해야 하며, 두 row는 동일한 primary key `2`와 서로 다른 payload `bea`, `dupe`를 가져야 합니다. malformed record tag, unknown record tag, 깨진 prefix, 손상된 length field는 이 duplicate invariant evidence를 대체할 수 없습니다.
- persisted duplicate primary-key row fixture를 새 `db exec` process에서 reopen/rebuild하는 경로는 exit code `1`, 비어 있는 stdout, stderr `error: invalid SQL storage record: duplicate primary key for table users: 2\nhint: primary key values must be unique in persisted SQL storage.\n`로 실패해야 합니다.
- baseline `scripts/verify`가 current managed repo SHA에서 통과해야 합니다.
- final evidence가 `gate-v1-indexes`와 `REQ-7-implement-integer-primary-key-as-9c698e08`를 명시적으로 연결해야 합니다.

## Evidence Path Contract
- `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`는 `REQ-7-implement-integer-primary-key-as-9c698e08`, `gate-v1-indexes`, acceptance scenario, required command를 scenario별로 매핑해야 합니다.
- `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`는 current managed repo SHA, `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify`의 exit code와 pass/fail 결과, final review mapping을 포함해야 합니다.
- `docs/v1_acceptance.md`를 갱신하는 경우 해당 row는 `gate-v1-indexes`, `REQ-7-implement-integer-primary-key-as-9c698e08`, current managed repo SHA, final evidence path를 함께 기록해야 하며 다른 requirement 완료를 주장하지 않아야 합니다.
- scheduler terminal result, queue delta, run report는 위 evidence path와 command result를 인용하는 보조 증거이며, 단독으로 artifact gate completion을 대체할 수 없습니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
