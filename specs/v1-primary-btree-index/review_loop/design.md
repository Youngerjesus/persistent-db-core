# Primary B-tree 인덱스 기반 행 조회와 정렬 스캔

## 정규화된 후보
- Rank: 1
- Feature slug: v1-primary-btree-index
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
현재 `db exec`는 durable storage 위에서 기본 SQL schema/insert/select를 수행하지만 primary key 기반 lookup과 ordered traversal evidence가 없습니다. V1 query correctness는 단순 full scan 예시만으로는 부족하며, persisted row를 대상으로 한 primary index proof가 필요합니다.

## 지금 해야 하는 이유
Root Progress Projection에서 CLI, disk page storage, SQL schema/exec gate는 `projected_complete`이고, `gate-v1-indexes`는 `req-v1-primary-index-proof`와 `req-v1-secondary-index-proof`가 open입니다. current plan도 `gap-v1-primary-btree-index`를 high priority로 두며 primary key insert/find/scan tests with deterministic ordering을 다음 slice로 제시합니다.

## 기대 산출물 변화
managed repo에 persisted primary B-tree index primitive와 SQL/storage integration proof를 추가하고, restart 후 primary lookup과 ordered scan이 deterministic하게 유지됨을 tests와 docs로 증명합니다.

## 의도한 변경 대상
- src/index.rs
- src/lib.rs
- src/main.rs
- src/storage.rs
- tests/primary_index.rs
- tests/sql_exec.rs
- docs/file_format.md
- docs/cli_contract.md
- route:db-exec-primary-index

## Risk flags
- persisted_index_compatibility_note_required

## 근거
- Root Progress Projection: `artifact_status=open`이며 open requirement에 `req-v1-primary-index-proof`와 `req-v1-secondary-index-proof`가 포함되어 있습니다.
- Root Progress Projection: `gate-v1-cli-smoke`, `gate-v1-disk-page-storage`, `gate-v1-sql-schema-exec`는 `projected_complete`로 완료 slice에서 제외됩니다.
- Current Plan: `gap-v1-primary-btree-index`는 high priority이고 linked metric은 `metric-v1-query-correctness`, linked artifact gate는 `gate-v1-indexes`입니다.
- Current Plan Gap Details: primary key index는 insert, find, ordered scan, deterministic persistence, query path integration evidence를 요구합니다.
- Managed repo progress: minimal SQL schema/execute path가 완료되었고 다음 작은 handoff는 indexing, recovery, validation gap 중 하나라고 기록되어 있습니다.
- Queue Snapshot: `[]`로 active/reserved duplicate task가 없습니다.
- Active Managed Repo Snapshot: git status가 clean이고 최근 history는 2026-05-17 minimal SQL milestone 완료를 기록합니다.
