# Primary index current-artifact evidence refresh

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-20-19-52-09
- Task ID: task-2026-05-20-19-52-09-v1-primary-index-current-artifact-evidence-refresh
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: Primary index current-artifact evidence refresh
- Artifact: v1-primary-index-current-artifact-evidence-refresh

## 목표
- Root Progress Projection은 `gate-v1-indexes`를 open으로 보고하며 `REQ-7-implement-integer-primary-key-as-9c698e08`가 current artifact 기준으로 아직 satisfied row가 아닙니다. 과거 primary index 구현과 테스트 증거는 있지만 artifact contract digest/current-SHA 요구에 맞춘 명시적 증거가 부족해 V1 query correctness 완료 판정이 막혀 있습니다.

## 지금 해야 하는 이유
- `gate-v1-disk-page-storage`는 projected_complete이므로 제외해야 하고, CLI/SQL slice는 conflicting evidence 및 open human request가 있어 자동 후보로 부적합합니다. primary index는 current plan에서 high priority이고, 기존 검증 명령과 merged evidence가 있어 가장 작은 비충돌 query-correctness evidence refresh 후보입니다.

## 기대 산출물 변화
- 관리 대상 repo에 current artifact requirement ID 기준 primary key index 증거를 갱신합니다. 예상 산출은 current SHA에서 `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify` 통과 증거와 `REQ-7-implement-integer-primary-key-as-9c698e08`에 매핑된 final evidence/review입니다.

## 의도한 변경 대상
- tests/primary_index.rs
- tests/sql_exec.rs
- scripts/verify_primary_index_acceptance
- docs/v1_acceptance.md
- specs/v1-primary-index-current-artifact-evidence-refresh/**

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: tests/primary_index.rs, tests/sql_exec.rs, docs/v1_acceptance.md, scripts/verify, specs/v1-primary-btree-index/final_review.md, src/index.rs, src/sql.rs, docs/sql_subset.md, docs/file_format.md, docs/cli_contract.md

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: artifact_status는 `open`이고 `gate-v1-disk-page-storage`만 `projected_complete`입니다.
- Root Progress Projection: `gate-v1-indexes`는 `open`이며 `REQ-7-implement-integer-primary-key-as-9c698e08`가 missing requirement로 남아 있습니다.
- Root Progress Projection: `gap-v1-primary-btree-index`는 `stale_needs_recheck`이고 blocker는 artifact contract digest/current artifact mismatch입니다.
- Current Plan: `gap-v1-primary-btree-index`는 priority `high`, linked metric `metric-v1-query-correctness`, linked artifact gate `gate-v1-indexes`입니다.
- 과거 evidence refs에는 `cargo test --test primary_index` pass, `cargo test --test sql_exec primary_key` pass, `./scripts/verify` pass, `specs/v1-primary-btree-index/final_review.md` PASS, PR #3 merge evidence가 있습니다.
- Queue Snapshot은 빈 배열이므로 active/reserved duplicate task가 없습니다.
- Human Requests Inbox에는 SQL schema current-artifact acceptance blocking approval이 열려 있어 SQL acceptance 후보는 보수적으로 제외했습니다.
- src/index.rs
- src/sql.rs
- tests/primary_index.rs
- tests/sql_exec.rs
- docs/sql_subset.md
- docs/file_format.md
- docs/cli_contract.md
- docs/v1_acceptance.md
- specs/v1-primary-btree-index/final_review.md
- specs/v1-primary-btree-index/impl_review.md
- specs/v1-primary-btree-index/code_review.md
- PrimaryIndex
- PrimaryIndex::insert
- PrimaryIndex::get
- PrimaryIndex::ordered_positions
- Database::from_records_with_check_label
- execute_insert
- execute_select
- execute_select_where
- append_loaded_row_after_validation
- validate_primary_key_available
- validate_catalog_record_invariants
- encode_catalog_record
- decode_record
- task-2026-05-17-22-43-31-v1-primary-btree-index: SUCCESS
- task-2026-05-19-01-26-09-v1-secondary-index-range-scan: SUCCESS
- task-2026-05-19-03-12-23-v1-secondary-index-mutation-consistency: SUCCESS
- task-2026-05-20-17-17-19-v1-page-storage-current-artifact-evidence-refresh: SUCCESS
- specs/v1-primary-btree-index/spec.md
- specs/v1-primary-btree-index/contracts.md

## 범위
- In scope: selected candidate only.
- Out of scope: unrelated breadth features.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `artifact_requirement_ids`는 `REQ-7-implement-integer-primary-key-as-9c698e08`만 명시하고 `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` 완료를 주장하지 않습니다.
- current managed repo SHA에서 `cargo test --test primary_index`가 통과하고 다음 `PrimaryIndex` observable behavior를 검증합니다.
  - `PrimaryIndex::insert`는 `2 -> 0`, `1 -> 1` 삽입 후 `get(2) == Some(0)`, `get(1) == Some(1)`, `get(3) == None`을 보장합니다.
  - duplicate key 삽입 `insert(2, 99)`는 오류를 반환하고 기존 `get(2) == Some(0)` 값을 덮어쓰지 않습니다.
  - `ordered_positions()`는 `30 -> 0`, `-5 -> 1`, `10 -> 2` 입력에서 key ascending 순서에 해당하는 `[1, 2, 0]`을 반환하고, empty index에서는 `[]`를 반환합니다.
  - persisted SQL rows에서 reopen/rebuild된 primary index는 `SELECT * FROM users WHERE id = 2;`에 대해 `id|name\n2|bea\n`를 출력하고, `SELECT * FROM users;`에 대해 `id|name\n1|ada\n2|bea\n3|cal\n` 순서를 출력합니다.
- current managed repo SHA에서 `cargo test --test sql_exec primary_key`가 통과하고 CLI SQL 경로의 primary-key lookup/ordering evidence를 다음 입력과 기대값으로 남깁니다.
  - 입력 SQL: `CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal'); SELECT * FROM users; SELECT * FROM users WHERE id = 2; SELECT * FROM users WHERE id = 9;`
  - 기대 exit code는 `0`, stderr는 비어 있고, stdout은 `id|name\n1|ada\n2|bea\n3|cal\nid|name\n2|bea\nid|name\n`입니다.
  - 같은 database path를 새 `db exec` process에서 다시 열어 `SELECT * FROM users;`와 `SELECT * FROM users WHERE id = 2;`를 실행해도 위 ordering과 exact lookup 결과가 동일해야 합니다.
  - duplicate primary key 입력 `INSERT INTO users VALUES (2, 'dupe');`는 exit code `2`, stdout은 비어 있고, stderr `error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n`를 반환해야 하며 기존 row를 변경하지 않아야 합니다.
  - persisted duplicate primary-key row 재개방 fixture는 유효한 SQL storage catalog record와 같은 table의 유효한 SQL row record 두 개를 사용해야 하며, 두 row는 동일한 primary key `2`와 서로 다른 payload `bea`, `dupe`를 가져야 합니다. 이 fixture는 malformed record tag, unknown record tag, 깨진 prefix, 손상된 length field로 duplicate invariant 검증을 대체할 수 없습니다.
  - 위 persisted duplicate primary-key row fixture를 새 `db exec` process에서 reopen/rebuild하는 경로는 exit code `1`, stdout은 비어 있고, stderr `error: invalid SQL storage record: duplicate primary key for table users: 2\nhint: primary key values must be unique in persisted SQL storage.\n`로 실패해야 합니다.
- baseline `scripts/verify`가 current managed repo SHA에서 통과해야 합니다.
- final evidence는 `gate-v1-indexes`와 `REQ-7-implement-integer-primary-key-as-9c698e08`를 명시적으로 연결하고, 위 세 명령의 결과와 current SHA를 같은 review surface에서 확인 가능하게 해야 합니다.

## 검증 계획
- 필수 명령:
  - `cargo test --test primary_index`
  - `cargo test --test sql_exec primary_key`
  - `scripts/verify`
- 필수 evidence path:
  - `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`는 `REQ-7-implement-integer-primary-key-as-9c698e08`, `gate-v1-indexes`, 위 acceptance scenario, required command를 scenario별로 매핑해야 합니다.
  - `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`는 current managed repo SHA, 세 required command의 exit code와 pass/fail 결과, `REQ-7-implement-integer-primary-key-as-9c698e08`에 대한 final review mapping을 포함해야 합니다.
  - `docs/v1_acceptance.md`를 갱신하는 경우 해당 row는 `gate-v1-indexes`, `REQ-7-implement-integer-primary-key-as-9c698e08`, current managed repo SHA, final evidence path를 함께 기록해야 하며 다른 requirement 완료를 주장하지 않아야 합니다.
  - scheduler run result는 final evidence path와 command 결과를 인용해야 하며, 단독으로 artifact gate completion을 대체하지 않습니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
