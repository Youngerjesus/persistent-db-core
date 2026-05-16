# 고정 크기 페이지 저장소와 레코드 재시작 검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-16-13-58-47
- Task ID: task-2026-05-16-13-58-47-v1-page-storage-record-format
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 고정 크기 페이지 저장소와 레코드 재시작 검증
- Artifact: v1-page-storage-record-format

## 목표
- 현재 persistent-db-core는 CLI skeleton과 CLI 계약 근거는 있으나, records가 disk page에 저장되고 process restart 뒤에도 동일하게 읽힌다는 durable storage 증거가 없습니다. SQL, index, WAL, recovery 작업은 모두 이 저장소 primitive 없이는 검증 가능한 제품 가치로 이어지기 어렵습니다.

## 지금 해야 하는 이유
- Current objective는 CLI 다음 순서로 durable page storage를 요구하고, current plan의 high priority gap 중 CLI 이후 첫 번째 기반 gap입니다. Queue Snapshot은 비어 있고, Root Progress Projection은 gate-v1-disk-page-storage의 두 evidence requirement가 모두 open이라고 보고합니다.

## 기대 산출물 변화
- managed repo에 deterministic page file 생성, fixed record encoding, append/read/reopen 동작, restart test, file/page/record format compatibility note가 추가됩니다.

## 의도한 변경 대상
- `src/storage.rs`: fixed-size page file 생성, record append/read, reopen read를 담당하는 storage primitive.
- `tests/page_storage.rs`: append/read/reopen, record encoding, file format failure mode를 검증하는 deterministic integration tests.
- `docs/file_format.md`: V1 page file, page header 또는 slot layout, record encoding, compatibility note.
- `src/main.rs`: 새 storage 전용 user-facing CLI command를 추가하지 않습니다. 기존 `db --help` CLI contract가 유지되는지만 smoke check로 확인합니다.

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 178aa445c286aee9929ed7e0b8a14bd7e3d6b2e0
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, work_queue/progress.md, AGENTS.md, docs/history_archives/history.md, .codex/skills/spec-creator/SKILL.md, .codex/skills/spec-reviewer/SKILL.md, Cargo.toml, docs/cli_contract.md, docs/v1_spec.md, specs/v1-bootstrap-cli-contract/code_review.md, specs/v1-bootstrap-cli-contract/contracts.md

## Risk flags
- format_compatibility_sensitive
- requires_existing_cli_contract_preservation
- pre_launch_no_existing_user_data_assumed

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- ssot/current-objective.md: metric-v1-durable-storage는 records가 deterministic on-disk page storage를 통해 restart 뒤에도 살아남아야 한다고 정의합니다.
- ssot/current-plan.md: gap-v1-page-storage-record-format은 high priority이며 page file creation, fixed record encoding, restart read verification을 next candidate hint로 둡니다.
- ssot/current-artifact.md: gate-v1-disk-page-storage는 req-v1-page-storage-restart와 req-v1-record-format-doc을 완료 조건으로 둡니다.
- Root Progress Projection: gate-v1-disk-page-storage status=open, missing_requirement_ids=[req-v1-page-storage-restart, req-v1-record-format-doc].
- Active Managed Repo Snapshot: work_queue/progress.md는 page storage와 record format implementation evidence가 아직 없다고 기록합니다.
- Queue Snapshot: []로 현재 중복 active/reserved task가 없습니다.
- Gap Evidence Cache: verified task는 v1-bootstrap-cli-contract 1개뿐이고 page storage verified evidence는 없습니다.
- autopilot/ssot/current-objective.md
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- autopilot/project_manager/specs/v1-page-storage-record-format/review_loop/design.md
- persistent-db-core_worktree/main/AGENTS.md
- persistent-db-core_worktree/main/Cargo.toml
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/tests/cli_contract.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/v1_spec.md
- persistent-db-core_worktree/main/work_queue/progress.md
- metric-v1-durable-storage
- gap-v1-page-storage-record-format
- gate-v1-disk-page-storage
- req-v1-page-storage-restart
- req-v1-record-format-doc
- HELP
- main
- [[bin]] name = "db"
- reserved_future_command_remains_unsupported
- autopilot/project_manager/tasks/tasks.json#task-2026-05-15-16-06-54-v1-bootstrap-cli-contract:SUCCESS
- autopilot/project_manager/tasks/tasks.json: no v1-page-storage-record-format task
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md: absent
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md: absent

## 범위
- 포함 범위: V1 storage primitive, deterministic page file format, record append/read/reopen tests, 최소 file format failure-mode tests, file format 문서화.
- 제외 범위: SQL parser, catalog, B-tree index, WAL, crash recovery, network service, multi-process concurrency, 새 storage 전용 user-facing CLI command.

## CLI 및 관찰 경로
- 이 task는 page storage primitive를 제품 내부 기반 기능으로 추가하며, `db` binary에 storage 전용 user-facing CLI command를 새로 노출하지 않습니다.
- 사용자 관찰 가능한 CLI 계약은 기존 `cargo run --bin db -- --help`가 성공하고 기존 help/reserved command 동작을 깨지 않는 것으로 제한합니다.
- append/read/reopen 동작은 `tests/page_storage.rs`의 public Rust storage API 기반 deterministic integration tests로 검증합니다.
- test 입력 record는 opaque byte payload로 다루며, 최소 `b"alpha"`, `b"beta"`, `b""`, `[0x00, 0xff, 0x10]` 같은 서로 다른 길이와 binary byte를 포함해야 합니다.
- read 결과는 append 순서와 byte 값이 정확히 보존되어야 하며, reopen 뒤에도 같은 database path에서 동일한 record sequence를 반환해야 합니다.
- truncated file/page, invalid magic/header, unsupported format version, page capacity를 초과하는 record, corrupt record length는 각각 panic이나 silent success 없이 deterministic error를 반환해야 합니다. Error type 또는 메시지는 각 failure-mode별로 테스트가 안정적으로 assert할 수 있어야 합니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- 고정 크기 page file을 생성하고 opaque byte record를 deterministic binary encoding으로 append/read할 수 있어야 합니다.
- 같은 database path를 닫고 다시 열었을 때 이전에 쓴 records를 동일한 순서와 byte 값으로 읽는 automated restart test가 있어야 합니다.
- record encoding test는 empty payload, 일반 ASCII payload, binary byte payload를 포함하고, read result가 append order를 보존함을 검증해야 합니다.
- file/page/record format 문서가 page size, file header 또는 page header, slot layout 또는 record length layout, endian, record encoding, compatibility note를 명시해야 합니다.
- compatibility note는 V1 pre-launch에서는 기존 user data backward compatibility를 보장하지 않지만, 이 spec 이후 format 변경은 문서와 테스트 갱신 없이 암묵적으로 이루어지면 안 된다고 명시해야 합니다.
- 최소 failure-mode tests는 truncated file/page, invalid magic/header, unsupported format version, page overflow record, corrupt record length를 각각 별도 deterministic error로 처리함을 검증해야 합니다.
- `cargo test`가 통과해야 합니다.
- `cargo test --test page_storage`가 append/read/reopen, encoding, failure-mode coverage를 실행하고 통과해야 합니다.
- `cargo run --bin db -- --help`가 기존 CLI contract를 깨지 않고 성공해야 합니다.

## 검증 계획
- 필수 command:
  - `cargo test`
  - `cargo test --test page_storage`
  - `cargo run --bin db -- --help`
- Acceptance evidence mapping은 다음과 같습니다.
  - page file 생성과 append/read: `cargo test --test page_storage`의 append/read test output과 `tests/page_storage.rs`.
  - restart read verification: `cargo test --test page_storage`의 reopen/restart test output과 `tests/page_storage.rs`.
  - record encoding: `cargo test --test page_storage`의 encoding coverage output과 `tests/page_storage.rs`.
  - truncated file/page failure-mode: `cargo test --test page_storage`의 truncated file/page test output과 deterministic error assertions.
  - invalid magic/header failure-mode: `cargo test --test page_storage`의 invalid magic/header test output과 deterministic error assertions.
  - unsupported format version failure-mode: `cargo test --test page_storage`의 unsupported format version test output과 deterministic error assertions.
  - page overflow record failure-mode: `cargo test --test page_storage`의 overflow record test output과 deterministic error assertions.
  - corrupt record length failure-mode: `cargo test --test page_storage`의 corrupt record length test output과 deterministic error assertions.
  - file format documentation: `docs/file_format.md` 존재 및 page size, header/layout, endian, record encoding, compatibility note 섹션 확인.
  - CLI contract preservation: `cargo run --bin db -- --help` command output과 기존 CLI contract tests.
  - 최종 연결 증거: scheduler run report의 command 결과, 변경 파일 목록, acceptance criterion별 proof entry.

## Visual Verification Evidence
- 적용 제외입니다. 이 task는 reference bundle, DOM route, viewport, screenshot, UX review가 없는 Rust CLI storage/file-format 작업입니다.
- 검증 근거는 deterministic command output, integration test output, `docs/file_format.md`, scheduler run report로 제한합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: file format이 이후 SQL, index, WAL 작업의 기반이 되므로 format ambiguity와 silent corruption을 방치하면 후속 gap 검증이 불안정해집니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
