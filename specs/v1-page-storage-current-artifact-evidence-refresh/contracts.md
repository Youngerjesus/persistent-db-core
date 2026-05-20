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
- 생성 대상 코드 또는 문서: 디스크 페이지 스토리지 current-artifact 증거 재검증에 대한 closure.
- 생성 대상 테스트 또는 verification output: scheduler terminal result와 supporting evidence.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- tests/page_storage.rs 또는 동등한 black-box/focused test가 4096-byte 기본 page layout, page-level read/write abstraction, file header/page inspection을 검증한다.
- restart 후 기존 record를 다시 읽는 deterministic test가 REQ-6-data-must-survive-process-restart-0471a233를 명시적으로 매핑한다.
- memory-only dump-at-end 설계와 every-write full-file rewrite 설계를 거부할 수 있는 bounded mutation/file-inspection evidence가 FAIL-6 요구사항 2개를 명시적으로 매핑한다.
- docs/file_format.md 또는 docs/v1_acceptance.md가 current-artifact requirement IDs와 검증 명령을 연결한다.
- cargo test --test page_storage와 scripts/verify가 managed repo 기준으로 통과한다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.