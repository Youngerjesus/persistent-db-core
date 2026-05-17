# 지원 SQL 부분집합 differential/property 테스트 추가

## 정규화된 후보
- Rank: 1
- Feature slug: v1-differential-property-tests
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
V1 artifact에서 SQL 실행, primary index, WAL recovery, crash matrix, invariant check는 검증되었지만, 지원 SQL 부분집합을 생성 시퀀스 기반으로 검증하는 differential/property evidence가 아직 없습니다.

## 지금 해야 하는 이유
남은 open requirement 중 `req-v1-differential-property-proof`는 executable test evidence로 좁게 닫을 수 있고, benchmark/docs acceptance보다 먼저 쿼리 correctness의 신뢰도를 높입니다.

## 기대 산출물 변화
managed repo에 deterministic operation generator, comparison oracle 또는 SQLite-backed differential check, seed/failing-case capture, task-specific verification command를 추가하고 `scripts/verify` 통과를 유지합니다.

## 의도한 변경 대상
- tests/differential_property.rs
- scripts/verify_differential_property
- docs/cli_contract.md
- docs/testing.md
- Cargo.toml

## Risk flags
- 없음

## 근거
- Root Progress Projection: `artifact_status=open`, open requirement에 `req-v1-differential-property-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-differential-property-tests`는 `status=open`, blocker는 `missing satisfied requirement rows`입니다.
- Current Plan: `gap-v1-differential-property-tests`는 `metric-v1-acceptance-evidence`와 `gate-v1-differential-property-tests`에 매핑됩니다.
- Current Artifact: `req-v1-differential-property-proof`는 deterministic seed capture가 있는 SQLite differential/property test evidence를 요구합니다.
- Managed repo progress: `gap-v1-differential-property-tests`는 `missing_evidence`이며 SQLite differential/property harness가 아직 없습니다.
- Queue Snapshot: active 또는 reserved task가 없습니다.
