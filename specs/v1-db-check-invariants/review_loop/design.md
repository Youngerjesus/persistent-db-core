# `db check` invariant validation 추가

## 정규화된 후보
- Rank: 1
- Feature slug: v1-db-check-invariants
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
V1 데이터베이스는 저장소, primary index, WAL, crash matrix 증거를 갖췄지만 사용자가 기존 파일의 일관성을 확인하는 `db check` 명령과 corruption 실패 증거가 아직 없습니다.

## 지금 해야 하는 이유
완료된 storage, SQL, index, WAL, crash evidence 위에서 invariant checker를 추가하는 작업은 acceptance evidence metric에 직접 연결되고, benchmark/docs보다 먼저 실제 검증 표면을 확장합니다.

## 기대 산출물 변화
`db check <path>` 명령이 정상 데이터베이스를 성공으로 통과시키고 손상된 fixture를 명확한 비영 exit code와 오류 메시지로 거부하며, CLI contract와 테스트가 이를 문서화합니다.

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

## Risk flags
- data_loss_review_required
- persisted_format_compatibility_sensitive

## 근거
- Root Progress Projection: artifact_status=open이며 open_requirement_ids에 `req-v1-db-check-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-db-check-invariants`는 status=open, missing_requirement_ids=`req-v1-db-check-proof`입니다.
- Current Plan: `gap-v1-db-check-invariants`는 `metric-v1-acceptance-evidence`와 `gate-v1-db-check-invariants`에 연결되며 `db check` command와 valid/corrupted fixture cases를 요구합니다.
- Current Artifact: `req-v1-db-check-proof`는 `db check`가 valid fixture를 수락하고 corrupted fixture를 명확한 exit code로 거부해야 한다고 정의합니다.
- Managed repo progress: storage, SQL execution, primary index, WAL recovery, crash matrix는 이미 verification_ready 또는 완료 evidence가 있어 invariant checker의 기반이 존재합니다.
- Queue Snapshot: 활성 또는 예약 task가 없어 동일 feature 중복이 보이지 않습니다.
