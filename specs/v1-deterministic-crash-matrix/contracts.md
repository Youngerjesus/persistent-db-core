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
- 생성 대상 코드 또는 문서: `tests/crash_matrix.rs`, `tests/fixtures/crash_matrix/`, `scripts/verify_crash_matrix`, `docs/file_format.md`의 WAL sidecar compatibility note 또는 갱신 불필요 근거, 필요 시 `docs/cli_contract.md`.
- 생성 대상 테스트 또는 verification output: `./scripts/verify`, `cargo test --test crash_matrix`, `./scripts/verify_crash_matrix` 성공 evidence.
- 생성 대상 리포트 업데이트: `target/crash_matrix/crash_matrix_report.md`, scheduler final report artifact의 verification evidence section, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.
- crash matrix case 중 하나라도 구현되지 않거나 expected visible rows, WAL/file-format assertion, evidence id가 누락되면 task는 미완료입니다.
- user-facing CLI error/output 변화가 있는데 `docs/cli_contract.md`와 관련 integration test가 갱신되지 않으면 task는 미완료입니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- Required verification commands:
  - `./scripts/verify`
  - `cargo test --test crash_matrix`
  - `./scripts/verify_crash_matrix`
- Required after-implementation evidence paths:
  - `tests/crash_matrix.rs`
  - `tests/fixtures/crash_matrix/`
  - `scripts/verify_crash_matrix`
  - `docs/file_format.md`
  - `target/crash_matrix/crash_matrix_report.md`
  - scheduler final report artifact의 verification evidence section
- `cargo test --test crash_matrix`와 `./scripts/verify_crash_matrix`가 write, WAL append, commit marker, incomplete-tail, corrupt-tail, recovery replay 경계를 검증합니다.
- 각 matrix case는 고정 seed 또는 명명된 fixture를 사용하고, 실패 시 어떤 crash point가 깨졌는지 식별 가능한 output을 남깁니다.
- commit 완료 전 중단된 row는 reopen 후 보이지 않고, commit 완료 row는 reopen 후 deterministic `SELECT` output으로 확인됩니다.
- WAL sidecar replay는 반복 실행해도 idempotent하며 기존 CLI contract와 저장 포맷 호환성을 깨지 않습니다.
- baseline `./scripts/verify`가 통과하고, 기존 WAL recovery regression coverage가 제거되거나 약화되지 않아야 합니다.
- `docs/file_format.md`는 crash matrix가 검증하는 WAL sidecar compatibility note를 포함해야 합니다. 이미 충분하면 최종 리포트에 해당 문서 위치와 갱신 불필요 근거를 남겨야 합니다.
- user-facing CLI error/output 변화가 없어야 합니다. 변화가 있으면 `docs/cli_contract.md`와 관련 integration test를 함께 갱신해야 합니다.

## 최소 Crash Matrix 계약

| case_id | crash point | setup fixture/seed | injected interruption location | reopen command | expected visible rows | expected WAL/file-format compatibility assertion | required evidence id |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CM-001 | pre-wal-append | `seed_committed_one` | 새 row의 WAL frame append 시작 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` 또는 test harness의 동일 CLI path | `[(1, 'seed')]` | WAL sidecar가 없거나 비어 있어도 reopen이 성공하고 file format version과 기존 data file을 변경하지 않습니다. | `crash-matrix-case-CM-001` |
| CM-002 | partial-wal-frame | `seed_committed_one` | WAL frame header 또는 payload를 일부만 쓴 직후 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed')]` | incomplete WAL tail은 replay되지 않고 reopen이 panic 없이 성공합니다. 허용되는 cleanup/truncation 동작은 `docs/file_format.md` compatibility note에 기록합니다. | `crash-matrix-case-CM-002` |
| CM-003 | wal-frame-without-commit-marker | `seed_committed_one` | WAL frame은 완전히 썼지만 commit marker 쓰기 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed')]` | commit marker 없는 WAL frame은 replay되지 않으며 기존 `wal_recovery` uncommitted replay regression을 깨지 않습니다. | `crash-matrix-case-CM-003` |
| CM-004 | committed-wal-before-data-apply | `seed_committed_one` | commit marker flush 이후 data file apply 또는 checkpoint 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"`를 두 번 반복 | `[(1, 'seed'), (2, 'committed_wal')]` | committed WAL replay는 첫 reopen과 두 번째 reopen 모두 idempotent하며 중복 row를 만들지 않습니다. | `crash-matrix-case-CM-004` |
| CM-005 | recovery-interrupted-after-first-apply | WAL sidecar에 `(2, 'recover_a')`, `(3, 'recover_b')` committed frame이 있고 data file에는 `seed_committed_one`만 있는 fixture | recovery replay 중 첫 committed frame 적용 후 cleanup/checkpoint 완료 전 | interruption 이후 같은 `SELECT` reopen command를 다시 실행 | `[(1, 'seed'), (2, 'recover_a'), (3, 'recover_b')]` | recovery 자체가 중단되어도 다음 reopen에서 모든 committed frame이 정확히 한 번 보이고 WAL replay는 idempotent합니다. | `crash-matrix-case-CM-005` |
| CM-006 | corrupt-tail-after-committed-frame | `seed_committed_one`과 `(2, 'committed_before_tail')` committed WAL frame 뒤에 deterministic corrupt tail이 붙은 fixture | committed frame 뒤 trailing garbage 또는 invalid length tail을 남긴 상태 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed'), (2, 'committed_before_tail')]` | committed prefix는 replay되고 corrupt tail은 user-facing CLI output 변경 없이 안전하게 무시되거나 documented error 조건으로 처리됩니다. error/output 변화가 있으면 `docs/cli_contract.md` 갱신이 필수입니다. | `crash-matrix-case-CM-006` |

## Evidence Report Contract
- `target/crash_matrix/crash_matrix_report.md`는 한국어 또는 명확한 technical identifier 중심으로 작성하고, 각 matrix 행마다 `case_id`, `required evidence id`, reopen command, expected visible rows, actual visible rows, WAL/file-format assertion result, command exit status를 기록해야 합니다.
- scheduler final report artifact는 위 report path와 세 required verification command의 통과 결과를 연결해야 합니다.
- broader verification을 생략할 수 없습니다. 세 required verification command 중 하나라도 환경 문제로 실행하지 못하면 explicit blocker로 보고해야 합니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
