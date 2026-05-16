# 고정 크기 페이지 저장소와 레코드 재시작 검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-page-storage-record-format
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
현재 persistent-db-core는 CLI skeleton과 CLI 계약 근거는 있으나, records가 disk page에 저장되고 process restart 뒤에도 동일하게 읽힌다는 durable storage 증거가 없습니다. SQL, index, WAL, recovery 작업은 모두 이 저장소 primitive 없이는 검증 가능한 제품 가치로 이어지기 어렵습니다.

## 지금 해야 하는 이유
Current objective는 CLI 다음 순서로 durable page storage를 요구하고, current plan의 high priority gap 중 CLI 이후 첫 번째 기반 gap입니다. Queue Snapshot은 비어 있고, Root Progress Projection은 gate-v1-disk-page-storage의 두 evidence requirement가 모두 open이라고 보고합니다.

## 기대 산출물 변화
managed repo에 deterministic page file 생성, fixed record encoding, append/read/reopen 동작, restart test, file/page/record format compatibility note가 추가됩니다.

## 의도한 변경 대상
- src/main.rs
- src/storage.rs
- tests/page_storage.rs
- docs/file_format.md
- route:db-page-storage-open-append-read
- flow:restart-read-verification

## Risk flags
- format_compatibility_sensitive
- requires_existing_cli_contract_preservation
- pre_launch_no_existing_user_data_assumed

## 근거
- ssot/current-objective.md: metric-v1-durable-storage는 records가 deterministic on-disk page storage를 통해 restart 뒤에도 살아남아야 한다고 정의합니다.
- ssot/current-plan.md: gap-v1-page-storage-record-format은 high priority이며 page file creation, fixed record encoding, restart read verification을 next candidate hint로 둡니다.
- ssot/current-artifact.md: gate-v1-disk-page-storage는 req-v1-page-storage-restart와 req-v1-record-format-doc을 완료 조건으로 둡니다.
- Root Progress Projection: gate-v1-disk-page-storage status=open, missing_requirement_ids=[req-v1-page-storage-restart, req-v1-record-format-doc].
- Active Managed Repo Snapshot: work_queue/progress.md는 page storage와 record format implementation evidence가 아직 없다고 기록합니다.
- Queue Snapshot: []로 현재 중복 active/reserved task가 없습니다.
- Gap Evidence Cache: verified task는 v1-bootstrap-cli-contract 1개뿐이고 page storage verified evidence는 없습니다.
