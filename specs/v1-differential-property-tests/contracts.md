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
- 생성 대상 코드 또는 문서: `tests/differential_property.rs`, `scripts/verify_differential_property`, `docs/testing.md`, test-only `Cargo.toml` dev-dependency delta.
- 변경 금지 대상: `docs/cli_contract.md`. 이번 task는 CLI behavior 변경이 없는 test harness 작업입니다.
- 생성 대상 테스트 또는 verification output: `./scripts/verify`와 `./scripts/verify_differential_property` command output.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- 필수 verification command는 `./scripts/verify`와 `./scripts/verify_differential_property`입니다. 둘 중 하나라도 누락되거나 실패하면 task는 미완료입니다.
- `./scripts/verify_differential_property`는 repo root 외부 cwd에서도 실행 가능해야 하며 `cargo test --test differential_property -- --nocapture`를 실행해야 합니다.
- `cargo test --test differential_property -- --nocapture`는 deterministic seed로 지원 SQL operation sequence를 실행하고, SQLite expected result와 `db` actual result를 비교해야 합니다. SQLite-backed oracle은 필수이며 임의 in-memory comparison oracle만으로 대체할 수 없습니다.
- SQLite oracle dependency는 Rust test-only dev-dependency로만 허용합니다. `rusqlite` 추가는 `req-v1-differential-property-proof`를 닫기 위한 task-level reason으로 허용되지만 production dependency로 추가하면 안 됩니다. 외부 `sqlite3` binary는 필수 환경 전제로 삼지 않습니다.
- 최소 SQL subset은 `CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)`, `INSERT INTO kv (id, value) VALUES (?, ?)`, `SELECT * FROM kv`, `SELECT * FROM kv WHERE id = ?` semantics를 모두 포함해야 합니다.
- key domain은 deterministic seed에서 생성한 `i64` 범위의 unique primary key이고, value domain은 seed에서 생성한 ASCII text입니다. seed당 최소 25개 row와 100개 operation이 실행되어야 합니다.
- operation sequence는 successful insert, duplicate primary key insert, missing key lookup, full scan, primary-key lookup, ordered scan을 모두 포함해야 합니다. `SELECT *`는 ordered scan으로 취급합니다. duplicate primary key insert는 `db`와 SQLite가 모두 error로 처리해야 하고, missing key lookup은 빈 결과로 비교해야 합니다.
- `SELECT *` result ordering은 `id` 오름차순으로 검증해야 합니다. 현재 repo reality가 이 ordering을 보장하지 않으면 acceptance criteria를 완화하지 말고 explicit blocker로 보고해야 합니다.
- 실패 stdout에는 seed, failing operation index, 최소 재현 가능한 operation sequence, SQLite expected rows, `db` actual rows, 재실행 command가 포함되어야 합니다.
- 최소 재현 가능한 operation sequence는 같은 seed와 operation prefix로 동일 실패를 재현할 수 있는 가장 짧은 prefix입니다.
- 실패 evidence artifact를 생성할 경우 위치는 `target/differential_property/failures/<seed>.json`이어야 하며, stdout에는 artifact path와 재실행 command가 함께 출력되어야 합니다.
- `docs/testing.md`는 `./scripts/verify_differential_property`, seed 재실행 방식, failure evidence 위치를 문서화해야 합니다. `docs/cli_contract.md`는 변경하지 않았다는 점을 final report에 확인 evidence로 남겨야 합니다.
- 최종 evidence는 command output 또는 final report에서 `gate-v1-differential-property-tests`와 `req-v1-differential-property-proof`에 명시적으로 매핑되어야 합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
