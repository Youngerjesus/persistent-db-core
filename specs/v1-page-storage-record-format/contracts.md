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
- 생성 대상 코드 또는 문서: `src/storage.rs`의 page storage primitive와 `docs/file_format.md`의 file/page/record format 문서.
- 생성 대상 테스트 또는 verification output: `tests/page_storage.rs`, `cargo test`, `cargo test --test page_storage`, `cargo run --bin db -- --help` 결과.
- 생성 대상 리포트 업데이트: run report의 acceptance criterion별 proof entry, command output summary, 변경 파일 목록, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria item은 test output, command output, document path, run report evidence 또는 explicit blocker와 연결되어야 합니다.
- spec hardening 중 candidate acceptance criteria를 generic completion wording으로 약화하거나 병합하거나 대체하면 안 됩니다.
- 새 storage 전용 user-facing CLI command는 이 task의 산출물이 아닙니다. CLI evidence는 기존 `db --help` contract 보존 확인으로 제한합니다.
- 고정 크기 page file 생성과 deterministic binary append/read는 `cargo test --test page_storage`의 append/read test output과 `tests/page_storage.rs`로 증명해야 합니다.
- restart read verification은 같은 database path를 reopen한 뒤 append 순서와 byte 값이 동일함을 검증하는 `cargo test --test page_storage` test output으로 증명해야 합니다.
- record encoding은 empty payload, ASCII payload, binary byte payload를 포함한 `tests/page_storage.rs` assertions로 증명해야 합니다.
- file/page/record format 문서는 `docs/file_format.md`에서 page size, file header 또는 page header, slot layout 또는 record length layout, endian, record encoding, compatibility note 섹션을 확인해 증명해야 합니다.
- compatibility note는 V1 pre-launch에서 기존 user data backward compatibility를 보장하지 않는 전제와, 이후 format 변경 시 문서와 테스트 갱신이 필요하다는 제약을 포함해야 합니다.
- failure-mode behavior는 truncated file/page, invalid magic/header, unsupported format version, page overflow record, corrupt record length가 각각 panic이나 silent success 없이 deterministic error를 반환함을 `cargo test --test page_storage`의 별도 assertions로 증명해야 합니다.
- unsupported format version 증거는 invalid magic/header 증거와 병합할 수 없으며, format version field를 지원하지 않는 값으로 바꾼 독립 test output과 assertion으로 연결해야 합니다.
- 전체 회귀는 `cargo test` 통과로 증명해야 합니다.
- CLI contract 보존은 `cargo run --bin db -- --help` command output과 기존 CLI contract test 통과로 증명해야 합니다.
- 최종 run report는 위 항목별 command output, document path, evidence summary를 acceptance criterion별로 연결해야 합니다.

## Visual Evidence Contract
- 적용 제외입니다. 이 task는 Rust CLI storage/file-format 작업이며 reference bundle, DOM route, viewport, screenshot, UX design review를 요구하지 않습니다.
- Visual evidence 부재는 blocker가 아닙니다. 대신 deterministic test output, command output, `docs/file_format.md`, run report evidence가 필수 proof layer입니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
