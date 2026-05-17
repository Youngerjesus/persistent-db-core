# 트랜잭션 WAL 복구 최소 증거 추가

## 정규화된 후보
- Rank: 1
- Feature slug: v1-transaction-wal-recovery
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
현재 `persistent-db-core`는 durable storage, SQL 실행, index 증거는 갖췄지만, committed mutation만 재시작 후 살아남고 rollback 또는 partial mutation은 사라진다는 복구 계약 증거가 없습니다.

## 지금 해야 하는 이유
Root Progress Projection에서 CLI, storage, SQL, indexes gate는 `projected_complete`이고, 남은 recovery 계열 중 crash matrix는 WAL 복구 의미가 먼저 고정되어야 검증 가능합니다. 따라서 WAL replay의 최소 commit/rollback slice가 다음 의존성 병목입니다.

## 기대 산출물 변화
Rust CLI `db exec` 경로 또는 내부 storage/execution 계층에 최소 transaction WAL 기록, replay, rollback 처리와 deterministic restart tests를 추가하고, WAL 파일/복구 의미를 문서화합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/
- docs/file_format.md
- docs/cli_contract.md
- route:db-exec
- flow:wal-replay-recovery

## Risk flags
- 없음

## 근거
- Root Progress Projection: `gate-v1-transactions-wal-recovery` 상태가 `open`이고 missing requirement가 `req-v1-wal-recovery-proof`입니다.
- Current Plan: `gap-v1-transaction-wal-recovery`는 high priority이며 next candidate hint가 commit/rollback WAL replay입니다.
- Current Artifact: `gate-v1-transactions-wal-recovery`는 WAL replay tests, commit/rollback tests, recovery transcript, file-state evidence를 요구합니다.
- Managed repo progress: SQL schema/execute path와 primary-key indexed lookup/ordered scan proof가 이미 존재해 recovery layer를 검증할 실행 baseline이 있습니다.
- Queue Snapshot: 현재 active 또는 reserved task가 없습니다.
