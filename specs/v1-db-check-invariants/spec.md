# `db check` invariant validation 추가

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-18-03-29-23
- Task ID: task-2026-05-18-03-29-23-v1-db-check-invariants
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: `db check` invariant validation 추가
- Artifact: v1-db-check-invariants

## 목표
- V1 데이터베이스는 저장소, primary index, WAL, crash matrix 증거를 갖췄지만 사용자가 기존 파일의 일관성을 확인하는 `db check` 명령과 corruption 실패 증거가 아직 없습니다.

## 지금 해야 하는 이유
- 완료된 storage, SQL, index, WAL, crash evidence 위에서 invariant checker를 추가하는 작업은 acceptance evidence metric에 직접 연결되고, benchmark/docs보다 먼저 실제 검증 표면을 확장합니다.

## 기대 산출물 변화
- `db check <path>` 명령이 정상 데이터베이스를 성공으로 통과시키고 손상된 fixture를 명확한 비영 exit code와 오류 메시지로 거부하며, CLI contract와 테스트가 이를 문서화합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/cli_contract.rs
- tests/db_check.rs
- docs/cli_contract.md
- docs/file_format.md
- route:db-check-cli
- flow:storage-index-wal-invariant-validation

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 881905933361ae5957a43c350efb1b6005d759f0
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, src/storage.rs, tests/cli_contract.rs, docs/cli_contract.md, docs/file_format.md, src/sql.rs, src/index.rs, tests/page_storage.rs, tests/primary_index.rs

## Risk flags
- data_loss_review_required
- persisted_format_compatibility_sensitive

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: artifact_status=open이며 open_requirement_ids에 `req-v1-db-check-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-db-check-invariants`는 status=open, missing_requirement_ids=`req-v1-db-check-proof`입니다.
- Current Plan: `gap-v1-db-check-invariants`는 `metric-v1-acceptance-evidence`와 `gate-v1-db-check-invariants`에 연결되며 `db check` command와 valid/corrupted fixture cases를 요구합니다.
- Current Artifact: `req-v1-db-check-proof`는 `db check`가 valid fixture를 수락하고 corrupted fixture를 명확한 exit code로 거부해야 한다고 정의합니다.
- Managed repo progress: storage, SQL execution, primary index, WAL recovery, crash matrix는 이미 verification_ready 또는 완료 evidence가 있어 invariant checker의 기반이 존재합니다.
- Queue Snapshot: 활성 또는 예약 task가 없어 동일 feature 중복이 보이지 않습니다.
- src/main.rs
- src/lib.rs
- src/storage.rs
- src/sql.rs
- src/index.rs
- tests/cli_contract.rs
- tests/page_storage.rs
- tests/primary_index.rs
- tests/wal_recovery.rs
- tests/crash_matrix.rs
- tests/fixtures/crash_matrix/README.md
- docs/cli_contract.md
- docs/file_format.md
- docs/v1_spec.md
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- HELP
- main
- exit_with_sql_error
- sql::execute
- PageStore
- PageStore::open
- PageStore::read_records
- replay_wal
- PrimaryIndex
- Database::from_records
- validate_catalog_record_invariants
- reserved_future_command_remains_unsupported
- corrupt_record_length_returns_error
- autopilot/project_manager/tasks/tasks.json: task-2026-05-15-16-06-54-v1-bootstrap-cli-contract SUCCESS, gap-v1-bootstrap-cli-contract
- autopilot/project_manager/tasks/tasks.json: task-2026-05-16-13-58-47-v1-page-storage-record-format SUCCESS, gap-v1-page-storage-record-format
- autopilot/project_manager/tasks/tasks.json: task-2026-05-17-19-38-21-v1-sql-parser-schema-exec SUCCESS, gap-v1-sql-parser-schema-exec
- autopilot/project_manager/tasks/tasks.json: task-2026-05-17-22-43-31-v1-primary-btree-index SUCCESS, gap-v1-primary-btree-index
- autopilot/project_manager/tasks/tasks.json: task-2026-05-17-23-45-17-v1-transaction-wal-recovery SUCCESS, gap-v1-transaction-wal-recovery
- autopilot/project_manager/tasks/tasks.json: task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof SUCCESS, gap-v1-transaction-wal-recovery
- autopilot/project_manager/tasks/tasks.json: task-2026-05-18-02-23-10-v1-deterministic-crash-matrix SUCCESS, gap-v1-deterministic-crash-matrix
- autopilot/project_manager/tasks/tasks.json: gap-v1-db-check-invariants 항목 없음
- autopilot/project_manager/specs/v1-bootstrap-cli-contract
- autopilot/project_manager/specs/v1-page-storage-record-format
- autopilot/project_manager/specs/v1-sql-parser-schema-exec
- autopilot/project_manager/specs/v1-primary-btree-index
- autopilot/project_manager/specs/v1-transaction-wal-recovery
- autopilot/project_manager/specs/v1-wal-recovery-current-sha-proof
- autopilot/project_manager/specs/v1-deterministic-crash-matrix
- autopilot/project_manager/specs: v1-db-check-invariants 디렉터리 없음

## 범위
- 포함 범위: `db check <path>` CLI 명령, 정상 database file 검증, deterministic corruption fixture 검증, 존재하지 않는 경로와 열 수 없는 경로의 사용자 오류 처리, CLI contract 문서와 focused test 갱신.
- 제외 범위: network server, multi-process concurrency, distributed storage, query optimization, background repair, automatic data mutation, UI 또는 browser surface, visual regression evidence.

## Invariant Matrix
- 정상 database file: storage record를 끝까지 parse/read할 수 있고, catalog record와 data record invariant가 현재 durable format과 일치해야 하며, primary index로 재구성한 key set이 record set과 일치해야 합니다. 성공 시 exit code는 0이고 stdout은 정확히 `ok: db check passed\n`을 출력하며 stderr는 비어 있어야 합니다.
- Storage record parse/readability 실패: 잘린 record, 잘못된 record length, decode 불가능한 persisted bytes 같은 deterministic fixture를 `db check`가 감지해야 합니다. 실패 시 exit code는 비영 값이고 stdout은 비어 있어야 하며 stderr는 `error: db check failed:` prefix를 포함해야 합니다.
- Catalog/record invariant 실패: catalog가 참조하는 table 또는 schema와 persisted record payload가 모순되는 fixture를 감지해야 합니다. 실패 시 exit code는 비영 값이고 stdout은 비어 있어야 하며 stderr는 `error: db check failed:` prefix와 invariant 종류를 포함해야 합니다.
- Primary index consistency 실패: primary index가 durable record set과 누락, 중복, 다른 key mapping 중 하나로 불일치하는 fixture를 감지해야 합니다. 실패 시 exit code는 비영 값이고 stdout은 비어 있어야 하며 stderr는 `error: db check failed:` prefix와 `primary index` 또는 동등한 문서화된 invariant label을 포함해야 합니다.
- WAL replay consistency: 현재 V1 format의 WAL sidecar는 `<database-path>.wal`이고 frame layout은 `docs/file_format.md`의 `Write-Ahead Log Sidecar` 표를 기준으로 합니다. `tests/db_check.rs`는 test-local fixture 생성 코드로 database file을 만든 뒤, 현재 durable record count보다 큰 `record count before` 값을 가진 complete committed `0x01` page-append frame을 `<database-path>.wal`에 작성해야 합니다. 이 fixture는 replay 입력이 durable state보다 앞선 상태를 나타내며, durable state 비교 대상은 page-store record count와 SQL row set입니다. `db check <path>`는 이 fixture를 비영 exit code, 빈 stdout, `error: db check failed:` prefix, `wal replay consistency` invariant label이 포함된 stderr로 거부해야 합니다. unknown payload kind, future WAL version, checkpointed WAL format처럼 현재 V1 sidecar 문서 밖의 WAL shape은 out-of-scope이지만, WAL sidecar 부재 또는 storage-only corruption만으로 이 invariant evidence를 대체할 수 없습니다.
- 존재하지 않는 경로: `db check <path>`는 user-facing error로 실패해야 합니다. 실패 시 exit code는 비영 값이고 stdout은 비어 있어야 하며 stderr는 `error:` prefix와 경로를 열 수 없다는 의미의 안정적인 문구를 포함해야 합니다.
- 열 수 없는 파일 경로: `cargo test --test db_check`는 temp directory 자체를 `db check <path>`의 `<path>`로 전달하는 directory-path 시나리오를 필수 테스트로 포함해야 합니다. 이 시나리오는 권한 bit에 의존하지 않는 unreadable-path acceptance evidence이며, `db check`는 panic 없이 비영 exit code, 빈 stdout, `error:` prefix와 open/read 실패 의미가 포함된 stderr로 실패해야 합니다. 권한 기반 fixture는 추가할 수 있지만 platform guard 또는 skipped case는 directory-path evidence를 대체할 수 없습니다.
- 제외 invariant: crash 시점 탐색, benchmark, online repair, SQL semantic validation, multi-process lock validation, UI state validation은 이 task의 acceptance가 아닙니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `db check <path>`가 문서화된 CLI surface에 추가되고 정상 데이터베이스에서 exit code 0으로 성공합니다.
- deterministic fixture가 storage record parse/readability 실패, catalog/record invariant 실패, primary index consistency 실패, WAL replay consistency 실패 중 in-scope 시나리오를 각각 검증하며, `db check`는 각 실패를 비영 exit code와 안정적인 stderr prefix로 거부합니다. WAL replay consistency fixture는 `<database-path>.wal`에 complete committed page-append frame을 작성하고, 해당 frame의 `record count before`가 현재 durable record count보다 큰 ahead-of-store 상태임을 테스트 코드에서 명시해야 합니다.
- 자동화 테스트가 valid fixture, corrupted fixture, 존재하지 않는 경로, temp directory를 입력으로 주는 열 수 없는 파일 경로를 포함해 exit code와 stdout/stderr 계약을 검증합니다.
- `docs/cli_contract.md` 또는 관련 durable docs가 `db check` 사용법, 성공/실패 출력, compatibility note를 갱신합니다.
- 검증 명령은 `scripts/verify` baseline과 `cargo test --test db_check` focused check를 포함합니다.

## 검증 계획
- 필수 명령:
  - `scripts/verify`
  - `cargo test --test db_check`
- 기대 증거: scheduler run report, `scripts/verify` 결과, `cargo test --test db_check` 결과, `tests/db_check.rs`, `docs/cli_contract.md`, 필요한 경우 `docs/file_format.md`.

## CLI Verification Evidence
- 이 task에는 UI 또는 browser surface가 없습니다. DOM capture, rendered route state, screenshot artifact, UX design review는 completion evidence가 아닙니다.
- Deterministic evidence는 CLI command output, focused test output, persisted fixture 또는 fixture 생성 코드, durable docs diff, scheduler run report로 구성합니다.
- stale evidence를 피하기 위해 worker는 현재 task worktree에서 `scripts/verify`와 `cargo test --test db_check`를 새로 실행하고 결과를 report에 연결해야 합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
