# 보조 인덱스 mutation 일관성 검증

## 정규화된 후보
- Rank: 1
- Feature slug: v1-secondary-index-mutation-consistency
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
현재 secondary index는 `CREATE INDEX`, equality/range scan, reopen/WAL replay, `db check` 검증까지 진척됐지만, artifact 계약의 `REQ-7-insert-update-and-delete-must-997871f9`는 mutation 이후 table row와 primary/secondary index가 함께 유지되는 증거를 요구합니다.

## 지금 해야 하는 이유
managed repo progress가 다음 최소 handoff로 mutation-maintained secondary-index behavior를 지목하고 있고, queue가 비어 있으며, 직전 secondary-index milestone 직후에 같은 코드 표면에서 가장 작게 닫을 수 있는 남은 index gate slice입니다.

## 기대 산출물 변화
secondary indexed column 또는 indexed row에 대한 UPDATE/DELETE 시나리오를 추가하고, 재시작 후 equality/range query와 `db check`가 stale index entry, missing indexed row, dangling pointer를 잡는 deterministic test evidence를 남깁니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/secondary_index.rs
- tests/sql_exec.rs
- tests/db_check.rs
- docs/cli_contract.md

## Risk flags
- index_consistency_regression_risk
- requires_no_storage_format_change_or_explicit_compat_note
- requires_restart_and_wal_replay_coverage
- requires_db_check_negative_fixture

## 근거
- `work_queue/progress.md` Current State는 다음 최소 handoff가 mutation-maintained secondary-index behavior 또는 더 좁은 acceptance blocker라고 기록합니다.
- `docs/history_archives/history.md`의 2026-05-19 항목은 secondary index가 CREATE INDEX, equality/range scan, reopen/WAL replay, db check invariant validation까지 추가됐다고 기록하지만 UPDATE/DELETE mutation 유지 증거는 명시하지 않습니다.
- `ssot/current-artifact.md`는 `REQ-7-insert-update-and-delete-must-997871f9`와 `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`를 `gate-v1-indexes`의 source-required evidence로 요구합니다.
- Root Progress Projection은 `gate-v1-indexes`를 `open`으로 두고 해당 requirement rows를 missing으로 계산합니다.
- Queue Snapshot은 빈 배열이며 active/reserved task 중 같은 feature_slug 후보와 충돌하는 항목이 없습니다.
