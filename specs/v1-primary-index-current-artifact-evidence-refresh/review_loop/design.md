# Primary index current-artifact evidence refresh

## 정규화된 후보
- Rank: 1
- Feature slug: v1-primary-index-current-artifact-evidence-refresh
- Target boundary: managed_repo
- Selection type: instrumentation_gap
- Confidence: high

## 문제 정의
Root Progress Projection은 `gate-v1-indexes`를 open으로 보고하며 `REQ-7-implement-integer-primary-key-as-9c698e08`가 current artifact 기준으로 아직 satisfied row가 아닙니다. 과거 primary index 구현과 테스트 증거는 있지만 artifact contract digest/current-SHA 요구에 맞춘 명시적 증거가 부족해 V1 query correctness 완료 판정이 막혀 있습니다.

## 지금 해야 하는 이유
`gate-v1-disk-page-storage`는 projected_complete이므로 제외해야 하고, CLI/SQL slice는 conflicting evidence 및 open human request가 있어 자동 후보로 부적합합니다. primary index는 current plan에서 high priority이고, 기존 검증 명령과 merged evidence가 있어 가장 작은 비충돌 query-correctness evidence refresh 후보입니다.

## 기대 산출물 변화
관리 대상 repo에 current artifact requirement ID 기준 primary key index 증거를 갱신합니다. 예상 산출은 current SHA에서 `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify` 통과 증거와 `REQ-7-implement-integer-primary-key-as-9c698e08`에 매핑된 final evidence/review입니다.

## 의도한 변경 대상
- tests/primary_index.rs
- tests/sql_exec.rs
- scripts/verify_primary_index_acceptance
- docs/v1_acceptance.md
- specs/v1-primary-index-current-artifact-evidence-refresh/**

## Risk flags
- 없음

## 근거
- Root Progress Projection: artifact_status는 `open`이고 `gate-v1-disk-page-storage`만 `projected_complete`입니다.
- Root Progress Projection: `gate-v1-indexes`는 `open`이며 `REQ-7-implement-integer-primary-key-as-9c698e08`가 missing requirement로 남아 있습니다.
- Root Progress Projection: `gap-v1-primary-btree-index`는 `stale_needs_recheck`이고 blocker는 artifact contract digest/current artifact mismatch입니다.
- Current Plan: `gap-v1-primary-btree-index`는 priority `high`, linked metric `metric-v1-query-correctness`, linked artifact gate `gate-v1-indexes`입니다.
- 과거 evidence refs에는 `cargo test --test primary_index` pass, `cargo test --test sql_exec primary_key` pass, `./scripts/verify` pass, `specs/v1-primary-btree-index/final_review.md` PASS, PR #3 merge evidence가 있습니다.
- Queue Snapshot은 빈 배열이므로 active/reserved duplicate task가 없습니다.
- Human Requests Inbox에는 SQL schema current-artifact acceptance blocking approval이 열려 있어 SQL acceptance 후보는 보수적으로 제외했습니다.
