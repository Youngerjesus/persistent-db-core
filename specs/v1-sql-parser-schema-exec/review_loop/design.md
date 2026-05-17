# 최소 SQL schema/execute 경로 구현

## 정규화된 후보
- Rank: 1
- Feature slug: v1-sql-parser-schema-exec
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
V1은 CLI와 durable page storage 기반은 갖췄지만, 아직 사용자가 SQL로 schema를 만들고 row를 넣고 조회하는 observable query behavior가 없습니다. 이 상태에서는 index, WAL, crash, differential/property test 같은 후속 gate가 붙을 실행 의미론이 없습니다.

## 지금 해야 하는 이유
Progress Projection에서 CLI와 disk page storage gate는 `projected_complete`이고, `gate-v1-sql-schema-exec`는 `req-v1-sql-exec-examples`가 open입니다. current objective의 sequencing도 storage 다음 SQL/schema execution을 요구하므로 오늘의 가장 작은 downstream unblocker입니다.

## 기대 산출물 변화
Managed repo에 최소 SQL parser/schema catalog/executor와 CLI smoke path를 추가하고, deterministic tests와 docs로 `CREATE TABLE`, `INSERT`, `SELECT`의 입력, 출력 row, ordering, unsupported SQL error behavior를 고정합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- src/sql.rs
- tests/sql_exec.rs
- tests/cli_contract.rs
- docs/cli_contract.md
- docs/sql_subset.md
- route:db-sql-exec
- flow:create-table-insert-select

## Risk flags
- storage_format_compatibility_must_be_preserved
- unsupported_sql_error_contract_required
- no_protected_area_change

## 근거
- Current Artifact: `gate-v1-sql-schema-exec`는 open이고 `req-v1-sql-exec-examples`가 missing requirement입니다.
- Current Plan: `gap-v1-sql-parser-schema-exec`의 next candidate hint는 최소 `CREATE TABLE`, `INSERT`, `SELECT` path를 storage 위에 구현하는 것입니다.
- Root Progress Projection: `gate-v1-cli-smoke`와 `gate-v1-disk-page-storage`는 `projected_complete`라 오늘 후보에서 제외됩니다.
- Root Progress Projection: `gate-v1-sql-schema-exec`는 `status=open`, `missing_requirement_ids=[req-v1-sql-exec-examples]`입니다.
- Queue Snapshot: active 또는 reserved task가 없어 동일 feature 중복 실행 근거가 없습니다.
- Managed Repo Snapshot: repo git status가 clean이고, V1 Rust CLI database boundary가 active managed repo로 고정되어 있습니다.
