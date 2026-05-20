# Transaction WAL recovery current-artifact evidence refresh

## 정규화된 후보
- Rank: 1
- Feature slug: v1-transaction-wal-current-artifact-evidence-refresh
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
현재 WAL recovery 구현과 과거 task evidence는 존재하지만 Root Progress Projection은 `gate-v1-transactions-wal-recovery`의 source-bound requirement row를 open으로 보고 있습니다. V1 완료 판단은 scheduler SUCCESS가 아니라 current artifact requirement ID별 evidence에 묶여야 하므로, transaction/WAL recovery 증거를 현재 artifact 계약에 맞춰 재검증해야 합니다.

## 지금 해야 하는 이유
CLI와 SQL 후보는 conflicting_evidence 또는 human request로 자동 후보가 막혀 있고, index gate는 projected_complete라 제외됩니다. WAL recovery gate는 V1 신뢰성의 핵심이며 기존 `cargo test --test wal_recovery`, `scripts/verify`, WAL sidecar evidence, final review refs가 있어 작은 current-artifact evidence refresh로 검증 가능성이 높습니다.

## 기대 산출물 변화
`REQ-8-*`와 `REQ-9-*` requirement ID별로 committed-only replay, rollback/uncommitted invisibility, idempotent recovery, checkpoint/log truncation safety를 현재 repo SHA에서 재검증하고, canonical spec package와 managed repo evidence 문서가 current-artifact matcher가 읽을 수 있는 requirement ID, command, artifact refs를 명시하게 됩니다.

## 의도한 변경 대상
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/spec.md
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/contracts.md
- specs/v1-transaction-wal-current-artifact-evidence-refresh/
- tests/wal_recovery.rs
- scripts/verify
- docs/cli_contract.md
- docs/file_format.md
- docs/v1_acceptance.md

## Risk flags
- data_loss_risk_review_required
- checkpoint_truncation_evidence_may_be_insufficient

## 근거
- Root Progress Projection: `gate-v1-transactions-wal-recovery` status=open, missing requirement IDs include `REQ-8-begin-commit-and-rollback-provide-44e7901f`, `REQ-8-committed-writes-survive-crash-and-35caf667`, `REQ-9-provide-wal-or-equivalent-write-80297892`, `REQ-9-recovery-must-be-idempotent-and-300531dc`, `REQ-9-checkpoint-or-log-truncation-must-d633d286`.
- Root Progress Projection: `gap-v1-transaction-wal-recovery` has prior evidence refs for `cargo test --test wal_recovery`, `./scripts/verify`, retained WAL sidecar smoke, `specs/v1-transaction-wal-recovery/final_review.md`, and `specs/v1-wal-recovery-current-sha-proof` but blocker says artifact contract digest does not match current artifact.
- Managed repo history on 2026-05-18 records minimal WAL recovery and current-SHA WAL recovery re-verification milestones.
- Queue Snapshot is empty, so no active or reserved duplicate task blocks this slice.
- Policy requires high confidence and no conflicting_evidence; this candidate avoids CLI/SQL conflicting slices and does not touch protected CAO areas.
