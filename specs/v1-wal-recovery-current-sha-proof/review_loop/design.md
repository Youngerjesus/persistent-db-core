# 현재 SHA 기준 WAL 복구 증거 재검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-wal-recovery-current-sha-proof
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
WAL recovery 기능은 최근 구현된 것으로 보이지만, 현재 artifact 판단에서는 prior manifest SHA `754958b37fd01f796b9d4f7522a2062b6e65abc5`가 현재 repo SHA `33b480cac6cf9d505a86eda4c149a4471454f11d`와 맞지 않아 `gate-v1-transactions-wal-recovery`가 닫히지 않습니다. 현재 SHA에서 committed mutation 생존과 uncommitted 또는 incomplete WAL 항목 배제가 다시 증명되지 않으면 V1 회복성 gate와 후속 crash matrix가 모두 불안정합니다.

## 지금 해야 하는 이유
CLI, page storage, SQL, index gate는 Root Progress Projection에서 `projected_complete`이므로 오늘 후보에서 제외됩니다. 남은 high-priority recovery slice 중 WAL proof가 crash matrix보다 선행 조건이고, queue가 비어 있어 현재 SHA 검증을 가장 작은 독립 작업으로 handoff할 수 있습니다.

## 기대 산출물 변화
관리 레포의 `tests/wal_recovery.rs`, 관련 CLI smoke, WAL sidecar/recovery 문서 또는 evidence transcript를 현재 SHA 기준으로 보강해 `req-v1-wal-recovery-proof`와 직접 매핑되는 deterministic recovery proof를 남깁니다.

## 의도한 변경 대상
- tests/wal_recovery.rs
- docs/file_format.md
- docs/cli_contract.md
- src/main.rs
- src/lib.rs
- route:/db-exec-wal-recovery
- flow:/wal-replay-current-sha-proof

## Risk flags
- stale_evidence_sha_mismatch
- storage_recovery_semantics_boundary
- requires_current_sha_verification
- avoid_storage_format_change

## 근거
- Root Progress Projection: artifact_status는 `open`이고 `gate-v1-transactions-wal-recovery`는 `missing satisfied requirement rows` 때문에 `open`입니다.
- Root Progress Projection: `req-v1-wal-recovery-proof`는 WAL replay가 committed changes survival과 uncommitted absence를 증명해야 하지만 현재 `open`입니다.
- Root Progress Projection: `gap-v1-transaction-wal-recovery`는 prior manifest repo SHA `754958b37fd01f796b9d4f7522a2062b6e65abc5`와 현재 repo SHA `33b480cac6cf9d505a86eda4c149a4471454f11d` 불일치로 `stale_needs_recheck`입니다.
- Current Plan: `gap-v1-transaction-wal-recovery`는 high priority이며 `metric-v1-recovery-correctness`와 `gate-v1-transactions-wal-recovery`에 연결됩니다.
- Current Artifact: `req-v1-wal-recovery-proof`는 recover 후 committed changes survival과 uncommitted changes absence를 요구합니다.
- Active Managed Repo Snapshot: queue는 비어 있고 git_status는 clean이며, 2026-05-18 history/progress는 minimal WAL recovery milestone이 추가되었다고 보고합니다.
- Root Progress Projection: CLI, disk storage, SQL, index gates는 `projected_complete`라 오늘 후보 범위에서 제외됩니다.
