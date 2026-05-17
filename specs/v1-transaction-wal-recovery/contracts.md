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
- 생성 대상 코드 또는 문서: 트랜잭션 WAL 복구 최소 증거 추가에 대한 closure.
- 생성 대상 테스트: `tests/wal_recovery.rs`.
- 생성 대상 문서: WAL compatibility note가 포함된 `docs/file_format.md`.
- 조건부 생성 대상 문서: public CLI output, exit code, stderr contract가 변경될 때의 `docs/cli_contract.md`.
- 생성 대상 verification output: `cargo test`, `cargo test --test wal_recovery`, `./scripts/verify`, canonical CLI smoke command output.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `cargo test`가 통과합니다.
- `./scripts/verify`가 통과합니다.
- `cargo test --test wal_recovery`가 통과합니다.
- CLI-visible Scenario A는 temp database path에서 `CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');` 실행 후 새 `db exec` process로 `SELECT * FROM users;`를 수행합니다.
- Scenario A의 create/insert command는 exit code `0`, stdout `""`, stderr `""`를 증거로 남깁니다.
- Scenario A의 select command는 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 증거로 남깁니다.
- Scenario B는 committed row `1|ada`와 rollback 또는 incomplete row `9|ghost`를 포함한 deterministic WAL fixture를 reopen/replay한 뒤 `9|ghost`가 조회 결과나 storage row set에 없음을 증명합니다.
- Scenario B가 CLI fixture로 검증되면 select command는 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n`를 증거로 남깁니다.
- Scenario B가 storage-level fixture로만 검증되면 test는 row set이 `[(1, "ada")]`와 동등함을 assert하고, CLI fixture가 아닌 이유를 test 이름 또는 주석에 남깁니다.
- `docs/file_format.md`는 WAL 파일명 또는 위치, record layout 또는 framing, replay 순서, committed/rollback/incomplete entry 처리, 기존 database 파일을 열 때의 기대 동작을 모두 포함합니다.
- `docs/cli_contract.md`는 public CLI behavior가 달라질 때만 갱신하며, 달라지지 않으면 final report에 변경 없음 이유를 남깁니다.
- final report 또는 phase result는 `tests/wal_recovery.rs`, `docs/file_format.md`, 조건부 `docs/cli_contract.md`, verification command output, WAL file-state evidence summary를 연결합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
