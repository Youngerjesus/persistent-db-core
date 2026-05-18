# 보조 인덱스 생성 및 범위 스캔 검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-secondary-index-range-scan
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
V1 artifact의 `gate-v1-indexes`는 primary index evidence만으로 닫을 수 없으며, `CREATE INDEX` 기반 disk-backed secondary index와 range scan proof가 아직 명시 completion target으로 남아 있다.

## 지금 해야 하는 이유
storage, SQL execution, primary index, WAL recovery, crash matrix, `db check`, differential/property, benchmark/docs baseline이 이미 supporting evidence로 존재하고 queue도 비어 있다. 남은 구현 gap 중 보조 인덱스는 current objective의 query correctness를 직접 올리는 가장 작은 coherent slice다.

## 기대 산출물 변화
managed repo에 secondary index metadata/storage, indexed equality/range query path, restart persistence tests, black-box CLI examples, 필요한 문서 업데이트를 추가하되 `gate-v1-indexes`의 sibling requirement completion은 자동 추론하지 않는다.

## 의도한 변경 대상
- src/
- tests/secondary_index.rs
- tests/sql_exec.rs
- docs/file_format.md
- docs/cli_contract.md

## Risk flags
- persisted_format_compatibility_tests_required
- shared_gate_partial_slice_only
- do_not_close_primary_or_mutation_sibling_requirements
- stale_existing_evidence_must_not_be_reused_as_completion

## 근거
- `ssot/current-plan.md`는 `gap-v1-secondary-index-range-scan`을 `metric-v1-query-correctness`, `gate-v1-indexes`, expected target `REQ-7-create-index-must-create-disk-3b71a7dc`에 매핑한다.
- `ssot/current-artifact.md`의 `REQ-7-create-index-must-create-disk-3b71a7dc`는 disk-backed secondary indexes, equality lookup, bounded range scan, ordering, persistence, query path use를 요구한다.
- managed repo `work_queue/progress.md`는 `gap-v1-secondary-index-range-scan`을 `missing_evidence`로 두고, 다음 작은 handoff로 secondary indexes 또는 좁은 acceptance blocker를 제안한다.
- managed repo history는 storage, SQL execution, primary index, WAL recovery, crash matrix, `db check`, differential/property, benchmark/docs baseline이 이미 추가되었음을 기록한다.
- Queue Snapshot은 `[]`이고 Git Status는 `clean`이므로 active duplicate나 dirty worktree blocker가 없다.
