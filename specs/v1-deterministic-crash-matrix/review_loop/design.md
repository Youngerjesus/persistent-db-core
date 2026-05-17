# 결정적 crash matrix로 WAL 회복 경계 검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-deterministic-crash-matrix
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
현재 `persistent-db-core`는 WAL replay 증거는 갖췄지만, write/WAL/commit/recovery 중단 지점별 deterministic crash matrix가 없어 V1 회복 정확성 gate를 완료할 수 없습니다.

## 지금 해야 하는 이유
Root Progress Projection에서 WAL recovery gate는 `projected_complete`이고 `gate-v1-crash-testing`만 open으로 남아 있습니다. Current Plan도 WAL 이후 crash matrix를 high priority 다음 후보로 정의하며, Queue Snapshot이 비어 있어 중복 handoff 위험이 없습니다.

## 기대 산출물 변화
managed repo에 deterministic crash injection matrix와 검증 가능한 test/runner evidence를 추가해, crash point별 post-recovery 상태를 재현 가능하게 증명합니다.

## 의도한 변경 대상
- tests/crash_matrix.rs
- tests/fixtures/crash_matrix/
- src/
- scripts/verify_crash_matrix
- docs/

## Risk flags
- 없음

## 근거
- Root Progress Projection: `artifact_status=open`; `gate-v1-crash-testing` status=`open`, missing_requirement_ids=[`req-v1-crash-matrix-output`].
- Current Artifact: `req-v1-crash-matrix-output`은 write, commit, recovery boundaries를 덮는 deterministic crash matrix output을 요구합니다.
- Current Plan: `gap-v1-deterministic-crash-matrix`는 high priority이며 WAL 이후 crash injection around write/WAL/replay boundaries를 next candidate로 제시합니다.
- Root Progress Projection: `gate-v1-transactions-wal-recovery`는 status=`projected_complete`, satisfied_requirements=[`req-v1-wal-recovery-proof`]로 crash matrix 선행 조건이 충족됐습니다.
- Managed repo progress: 현재 상태는 SQL, primary index, WAL replay evidence가 있고 다음 작은 handoff 후보로 deterministic crash matrix 또는 validation gaps를 제시합니다.
- Queue Snapshot: []이며 Active Managed Repo Snapshot의 git_status는 clean입니다.
