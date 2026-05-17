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
- 생성 대상 코드 또는 문서: 현재 SHA 기준 WAL 복구 증거 재검증에 대한 closure.
- 생성 대상 테스트 또는 verification output: scheduler terminal result와 supporting evidence.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria 항목은 test output, command output, WAL file-state evidence, manual review evidence 또는 explicit blocker에 연결되어야 합니다.
- spec hardening 중 candidate acceptance criteria를 generic completion wording으로 약화하거나 병합하거나 대체하지 않습니다.
- Browser evidence, DOM capture, screenshot artifacts, rendered route state, UX design review는 이 Rust CLI WAL recovery task의 acceptance evidence가 아닙니다.
- `cargo test --test wal_recovery`가 현재 task worktree HEAD에서 통과하고, committed mutation이 별도 `db exec` process의 reopen/replay 뒤 살아남는 케이스를 검증합니다.
- uncommitted change absence는 별도 deterministic scenario로 검증해야 합니다. 공개 CLI에 rollback 또는 uncommitted transaction command가 없으면 WAL fixture를 직접 작성할 수 있지만, evidence는 해당 fixture가 V1에서 관찰 가능한 uncommitted WAL state를 대표하는 이유를 기록해야 합니다.
- incomplete trailing WAL entry exclusion은 별도 deterministic scenario로 검증해야 합니다. evidence는 incomplete tail의 ghost row가 recovery 후 결과 rows 또는 storage row set에 나타나지 않고, WAL sidecar가 향후 replay 가능한 상태로 유지되거나 cleanup되었음을 기록해야 합니다.
- `./scripts/verify`가 통과해 fmt, clippy, full test suite, `db --help` smoke를 함께 보장합니다.
- CLI smoke transcript는 temp DB path를 생성한 뒤 `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`를 실행하고, exit code `0`, stdout `""`, stderr `""`를 기록해야 합니다.
- CLI smoke transcript는 별도 process에서 `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"`를 실행하고, exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 기록해야 합니다.
- CLI smoke transcript는 create/insert 후 `$DB_PATH.wal` 존재 여부와 byte length, reopen select 후 `$DB_PATH.wal` 존재 여부와 byte length를 기록해야 합니다. 구현이 complete WAL frames를 retained sidecar로 유지한다면 sidecar는 존재하고 non-empty여야 합니다.
- 최종 report 또는 evidence transcript는 `git rev-parse HEAD`, `git status --short`, 실행한 `cargo test --test wal_recovery`, `./scripts/verify`, CLI smoke command의 실행 명령과 stdout/stderr 또는 transcript path를 포함해야 합니다.
- 최종 evidence는 `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, `req-v1-wal-recovery-proof`에 명시적으로 매핑되어야 합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
