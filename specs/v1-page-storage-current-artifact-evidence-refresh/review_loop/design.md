# 디스크 페이지 스토리지 current-artifact 증거 재검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-page-storage-current-artifact-evidence-refresh
- Target boundary: managed_repo
- Selection type: instrumentation_gap
- Confidence: high

## 문제 정의
디스크 페이지 스토리지 구현과 과거 SUCCESS 증거는 존재하지만, Root Progress Projection은 artifact contract digest 불일치 때문에 REQ-6 및 FAIL-6 요구사항을 current-artifact 기준으로 아직 open/stale 상태로 본다.

## 지금 해야 하는 이유
CLI와 SQL 후보는 conflicting_evidence 또는 human input blocker가 있고, page storage는 V1의 하위 의존성이라 stale 증거를 먼저 정리해야 이후 SQL, index, WAL, acceptance evidence 판단의 기반이 안정된다. Queue Snapshot도 비어 있어 중복 실행 위험이 낮다.

## 기대 산출물 변화
managed repo에 4096-byte page-backed file, restart durability, memory-only dump 금지, every-write full-file rewrite 금지를 current-artifact requirement ID와 연결하는 focused test/evidence/doc delta를 추가하고 scripts/verify로 재검증한다.

## 의도한 변경 대상
- tests/page_storage.rs
- docs/file_format.md
- docs/v1_acceptance.md
- scripts/verify_page_storage_acceptance

## Risk flags
- 없음

## 근거
- Root Progress Projection: gate-v1-disk-page-storage는 open이며 REQ-6-store-data-in-a-disk-ad3ffc4e, REQ-6-data-must-survive-process-restart-0471a233, FAIL-6-reject-memory-only-dump-at-fd82a296, FAIL-6-reject-whole-database-file-rewrite-bebf73bb가 missing_requirement_ids로 남아 있다.
- Root Progress Projection: gap-v1-page-storage-record-format은 stale_needs_recheck이며 blocker는 artifact contract digest does not match current artifact이다.
- Gap Evidence Cache: task-2026-05-16-13-58-47-v1-page-storage-record-format는 SUCCESS 상태이며 src/storage.rs, tests/page_storage.rs, docs/file_format.md 증거가 연결되어 있다.
- Active Managed Repo Snapshot: git_status는 clean이고 Queue Snapshot은 []이다.
- Current Plan: gap-v1-page-storage-record-format은 metric-v1-durable-storage와 gate-v1-disk-page-storage에 직접 연결되어 있다.
