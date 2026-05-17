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
- 생성 대상 코드 또는 문서: `db check` invariant validation 추가에 대한 closure.
- 생성 대상 테스트 또는 verification output: scheduler terminal result와 supporting evidence.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria 항목은 test output, CLI command output, durable docs diff, scheduler run evidence, manual review evidence, 또는 explicit blocker에 연결되어야 합니다.
- Candidate Acceptance Criteria를 generic completion wording으로 약화하거나 병합해 제거하지 않습니다.
- `db check <path>`가 문서화된 CLI surface에 추가되고 정상 데이터베이스에서 exit code 0으로 성공합니다.
- deterministic fixture가 storage record parse/readability 실패, catalog/record invariant 실패, primary index consistency 실패, WAL replay consistency 실패 중 in-scope 시나리오를 각각 검증하며, `db check`가 각 실패를 비영 exit code와 안정적인 stderr prefix로 거부합니다. WAL replay consistency evidence는 `<database-path>.wal`에 complete committed page-append frame을 작성하고, 해당 frame의 `record count before`가 현재 durable record count보다 큰 ahead-of-store fixture로 고정합니다.
- 자동화 테스트가 valid fixture, corrupted fixture, 존재하지 않는 경로, temp directory를 입력으로 주는 열 수 없는 파일 경로를 포함해 exit code와 stdout/stderr 계약을 검증합니다.
- `docs/cli_contract.md` 또는 관련 durable docs가 `db check` 사용법, 성공/실패 출력, compatibility note를 갱신합니다.
- 검증 명령은 `scripts/verify` baseline과 `cargo test --test db_check` focused check를 포함합니다.

## Invariant Evidence Contract
- 정상 database file은 exit code 0, 정확히 `ok: db check passed\n`인 stdout, 빈 stderr를 증명해야 합니다.
- Storage record parse/readability 실패는 비영 exit code, 빈 stdout, `error: db check failed:` prefix가 있는 stderr로 증명해야 합니다.
- Catalog/record invariant 실패는 비영 exit code, 빈 stdout, `error: db check failed:` prefix와 invariant 종류가 있는 stderr로 증명해야 합니다.
- Primary index consistency 실패는 비영 exit code, 빈 stdout, `error: db check failed:` prefix와 `primary index` 또는 동등한 문서화된 invariant label이 있는 stderr로 증명해야 합니다.
- WAL replay consistency 실패는 현재 V1 `<database-path>.wal` sidecar format으로 만든 complete committed `0x01` page-append frame을 사용해 증명해야 합니다. 필수 fixture는 test-local 생성 코드로 database file을 만든 뒤 현재 durable record count보다 큰 `record count before` 값을 가진 WAL frame을 작성해야 하며, durable state 비교 대상은 page-store record count와 SQL row set입니다. `db check <path>`는 이 ahead-of-store WAL fixture를 비영 exit code, 빈 stdout, `error: db check failed:` prefix, `wal replay consistency` invariant label이 포함된 stderr로 거부해야 합니다. unknown payload kind, future WAL version, checkpointed WAL format처럼 현재 V1 sidecar 문서 밖의 WAL shape은 out-of-scope이지만, WAL sidecar 부재 또는 storage-only corruption은 이 evidence를 대체할 수 없습니다.
- 존재하지 않는 경로는 panic 없이 user-facing error로 실패해야 하며, stderr는 `error:` prefix와 open/read 실패 의미를 포함해야 합니다.
- 열 수 없는 파일 경로 evidence는 `cargo test --test db_check` 안에서 temp directory 자체를 `db check <path>` 입력으로 주는 directory-path 시나리오로 고정합니다. 이 시나리오는 비영 exit code, 빈 stdout, `error:` prefix와 open/read 실패 의미가 포함된 stderr를 증명해야 합니다. 권한 기반 fixture는 추가 evidence로만 허용하며, platform guard 또는 skipped case는 acceptance를 대체할 수 없습니다.
- UI, DOM, screenshot, rendered route state, UX design review는 이 CLI-only task의 completion evidence가 아닙니다.

## Required Verification Commands
- `scripts/verify`
- `cargo test --test db_check`

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
