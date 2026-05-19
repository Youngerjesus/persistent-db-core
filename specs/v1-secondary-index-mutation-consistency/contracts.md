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
- 생성 대상 코드 또는 문서: 보조 인덱스 mutation 일관성 검증에 대한 closure.
- 생성 대상 테스트 또는 verification output: `./scripts/verify`, `cargo test --test secondary_index -- --nocapture`, 필요 시 `cargo test --test db_check -- --nocapture` 실행 증거.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.
- final report 또는 scheduler run artifact는 command별 exit status, stdout/stderr 요약, 관련 test name, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` 매핑을 포함해야 합니다.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria item은 test output, command output, manual review evidence, 또는 explicit blocker에 연결되어야 합니다.
- spec hardening 중 candidate acceptance criteria를 generic completion wording으로 약화, 병합, 대체하지 않습니다.
- 이 task package는 managed repo contract가 요구하는 repo-local verification command를 명시할 수 있습니다. command는 task worktree root에서 실행해야 하며, 외부 서비스나 secret에 의존하지 않아야 합니다.
- 고정 fixture는 다음 SQL로 시작해야 합니다.

```sql
CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT);
INSERT INTO users VALUES (1, 10, 'ada');
INSERT INTO users VALUES (2, 20, 'bea');
INSERT INTO users VALUES (3, 20, 'cal');
INSERT INTO users VALUES (4, 30, 'dia');
CREATE INDEX idx_users_age ON users(age);
```

- `UPDATE users SET age = 30 WHERE id = 2;`는 exit `0`, empty stdout, empty stderr여야 합니다.
- UPDATE 이후 old key equality query `SELECT * FROM users WHERE age = 20;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
3|20|cal
```

- UPDATE 이후 new key equality query `SELECT * FROM users WHERE age = 30;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
2|30|bea
4|30|dia
```

- UPDATE 이후 range query `SELECT * FROM users WHERE age BETWEEN 20 AND 30;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
3|20|cal
2|30|bea
4|30|dia
```

- UPDATE 이후 primary-key lookup `SELECT * FROM users WHERE id = 2;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
2|30|bea
```

- UPDATE 이후 table scan `SELECT * FROM users;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
3|20|cal
4|30|dia
```

- `DELETE FROM users WHERE id = 3;`는 exit `0`, empty stdout, empty stderr여야 합니다.
- DELETE 이후 secondary equality query `SELECT * FROM users WHERE age = 20;`은 header-only stdout이어야 합니다.

```text
id|age|name
```

- DELETE 이후 secondary range query `SELECT * FROM users WHERE age BETWEEN 10 AND 30;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
4|30|dia
```

- DELETE 이후 primary-key lookup `SELECT * FROM users WHERE id = 3;`은 header-only stdout이어야 합니다.

```text
id|age|name
```

- DELETE 이후 table scan `SELECT * FROM users;`은 정확히 다음 stdout이어야 합니다.

```text
id|age|name
1|10|ada
2|30|bea
4|30|dia
```

- restart/reopen 증거는 setup, UPDATE, DELETE, query, `db check`가 각각 별도 `db` process invocation으로 실행됐음을 보여야 합니다.
- UPDATE/DELETE 후 `db check <path>`는 exit `0`, stdout `ok: db check passed\n`, empty stderr여야 합니다.
- WAL replay 증거는 page file과 `<path>.wal` sidecar가 존재하는 상태에서 별도 process invocation으로 reopen query와 `db check <path>`를 실행해 남겨야 합니다.
- stale secondary entry fixture는 visible row `id=2`가 `age=30`으로 update된 뒤 old key `20` entry가 남은 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- dangling row pointer fixture는 committed secondary index entry가 존재하지 않는 row position 또는 deleted row를 참조하는 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- missing indexed visible row fixture는 visible indexed row에 대응하는 committed secondary entry가 없는 상태를 만들어야 합니다. `db check <path>` expected result는 exit `1`, empty stdout, stderr containing exactly `error: db check failed: secondary index\n`입니다.
- Negative fixture는 checked-in binary fixture 또는 deterministic fixture builder 중 하나로 제공할 수 있습니다. deterministic fixture builder를 쓰는 경우 test code 안에서 fixture construction 절차가 재현 가능해야 합니다.
- storage format을 바꾸지 않는 경우 final report에 `no storage format change`와 기존 database compatibility 확인을 기록해야 합니다. storage format을 바꾸는 경우 `docs/file_format.md`, `docs/sql_subset.md`, `docs/cli_contract.md`를 갱신하고 기존 row-only 및 existing secondary-index database compatibility note를 기록해야 합니다.

## Verification Evidence Contract
- 필수 baseline command: `./scripts/verify`
- 필수 focused command: `cargo test --test secondary_index -- --nocapture`
- `db check` negative fixture coverage가 `tests/db_check.rs`에 분리되면 추가 command `cargo test --test db_check -- --nocapture`를 실행해야 합니다.
- Verification evidence는 run-local `result.md` 또는 `final.md`에 command, exit status, stdout/stderr 요약, 관련 test name, requirement/evidence id 매핑을 남겨야 합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
