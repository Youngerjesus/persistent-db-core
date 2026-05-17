# 결정적 crash matrix로 WAL 회복 경계 검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-18-02-23-10
- Task ID: task-2026-05-18-02-23-10-v1-deterministic-crash-matrix
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 결정적 crash matrix로 WAL 회복 경계 검증
- Artifact: v1-deterministic-crash-matrix

## 목표
- 현재 `persistent-db-core`는 WAL replay 증거는 갖췄지만, write/WAL/commit/recovery 중단 지점별 deterministic crash matrix가 없어 V1 회복 정확성 gate를 완료할 수 없습니다.

## 지금 해야 하는 이유
- Root Progress Projection에서 WAL recovery gate는 `projected_complete`이고 `gate-v1-crash-testing`만 open으로 남아 있습니다. Current Plan도 WAL 이후 crash matrix를 high priority 다음 후보로 정의하며, Queue Snapshot이 비어 있어 중복 handoff 위험이 없습니다.

## 기대 산출물 변화
- managed repo에 deterministic crash injection matrix와 검증 가능한 test/runner evidence를 추가해, crash point별 post-recovery 상태를 재현 가능하게 증명합니다.

## 의도한 변경 대상
- tests/crash_matrix.rs
- tests/fixtures/crash_matrix/
- src/
- scripts/verify_crash_matrix
- docs/

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 358854464059ba41ed2f232cfc6ab17a9ce51dac
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/index.rs, src/lib.rs, src/main.rs, src/sql.rs, src/storage.rs, docs/cli_contract.md, docs/file_format.md, docs/history_archives/history.md, docs/sql_subset.md, docs/v1_spec.md, AGENTS.md

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `artifact_status=open`; `gate-v1-crash-testing` status=`open`, missing_requirement_ids=[`req-v1-crash-matrix-output`].
- Current Artifact: `req-v1-crash-matrix-output`은 write, commit, recovery boundaries를 덮는 deterministic crash matrix output을 요구합니다.
- Current Plan: `gap-v1-deterministic-crash-matrix`는 high priority이며 WAL 이후 crash injection around write/WAL/replay boundaries를 next candidate로 제시합니다.
- Root Progress Projection: `gate-v1-transactions-wal-recovery`는 status=`projected_complete`, satisfied_requirements=[`req-v1-wal-recovery-proof`]로 crash matrix 선행 조건이 충족됐습니다.
- Managed repo progress: 현재 상태는 SQL, primary index, WAL replay evidence가 있고 다음 작은 handoff 후보로 deterministic crash matrix 또는 validation gaps를 제시합니다.
- Queue Snapshot: []이며 Active Managed Repo Snapshot의 git_status는 clean입니다.
- persistent-db-core_worktree/main/src/storage.rs
- persistent-db-core_worktree/main/tests/wal_recovery.rs
- persistent-db-core_worktree/main/docs/file_format.md
- persistent-db-core_worktree/main/docs/v1_spec.md
- persistent-db-core_worktree/main/work_queue/progress.md
- persistent-db-core_worktree/main/scripts/verify
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- PageStore::open
- PageStore::append_record
- replay_wal
- append_wal_frame
- committed_wal_replay_survives_reopen_via_cli
- rolled_back_wal_frame_is_not_replayed_as_uncommitted_change
- incomplete_wal_entry_is_not_replayed_without_public_rollback_cli
- committed_frame_after_incomplete_tail_cleanup_remains_replayable
- task-2026-05-17-23-45-17-v1-transaction-wal-recovery
- task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof
- project_manager/specs/v1-transaction-wal-recovery/spec.md
- project_manager/specs/v1-transaction-wal-recovery/contracts.md
- project_manager/specs/v1-wal-recovery-current-sha-proof/spec.md
- project_manager/specs/v1-wal-recovery-current-sha-proof/contracts.md

## 범위
- In scope: WAL write, commit marker, incomplete/corrupt tail, recovery replay 경계를 덮는 deterministic crash matrix test/runner, 필요한 최소 crash injection hook, 검증 스크립트, fixture, file format/WAL compatibility note.
- Out of scope: networked database server behavior, multi-process concurrency, distributed storage, query optimizer 변경, 기존 CLI contract와 무관한 출력 형식 변경.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Deterministic Crash Matrix
- Worker는 아래 최소 행을 `tests/crash_matrix.rs` 또는 동등한 integration test에서 모두 검증해야 합니다.
- 모든 case는 fixture/seed를 코드 또는 `tests/fixtures/crash_matrix/` 아래 명명된 fixture로 고정하고, 실패 메시지에 `case_id`와 crash point를 포함해야 합니다.
- `db` reopen command는 task worktree에서 생성한 임시 DB 경로를 사용해야 하며, `SELECT` output 비교는 row 순서까지 deterministic해야 합니다.
- 아래 관찰된 함수명과 파일명은 시작점이며 구현 mandate가 아닙니다. 최신 worktree의 실제 구조에 맞춰 같은 행위 계약을 만족해야 합니다.

| case_id | crash point | setup fixture/seed | injected interruption location | reopen command | expected visible rows | expected WAL/file-format compatibility assertion | required evidence id |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CM-001 | pre-wal-append | `seed_committed_one`: table에 `(1, 'seed')`만 commit된 DB에서 `(2, 'pre_wal')` insert 시도 | 새 row의 WAL frame append 시작 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` 또는 test harness의 동일 CLI path | `[(1, 'seed')]` | WAL sidecar가 없거나 비어 있어도 reopen이 성공하고 file format version과 기존 data file을 변경하지 않습니다. | `crash-matrix-case-CM-001` |
| CM-002 | partial-wal-frame | `seed_committed_one`에서 `(2, 'partial_wal')` insert 시도 | WAL frame header 또는 payload를 일부만 쓴 직후 process interruption을 주입 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed')]` | incomplete WAL tail은 replay되지 않고 reopen이 panic 없이 성공합니다. 허용되는 cleanup/truncation 동작은 `docs/file_format.md` compatibility note에 기록합니다. | `crash-matrix-case-CM-002` |
| CM-003 | wal-frame-without-commit-marker | `seed_committed_one`에서 `(2, 'uncommitted')` insert 시도 | WAL frame은 완전히 썼지만 commit marker 쓰기 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed')]` | commit marker 없는 WAL frame은 replay되지 않으며 기존 `wal_recovery` uncommitted replay regression을 깨지 않습니다. | `crash-matrix-case-CM-003` |
| CM-004 | committed-wal-before-data-apply | `seed_committed_one`에서 `(2, 'committed_wal')` commit 완료 | commit marker flush 이후 data file apply 또는 checkpoint 전 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"`를 두 번 반복 | `[(1, 'seed'), (2, 'committed_wal')]` | committed WAL replay는 첫 reopen과 두 번째 reopen 모두 idempotent하며 중복 row를 만들지 않습니다. | `crash-matrix-case-CM-004` |
| CM-005 | recovery-interrupted-after-first-apply | WAL sidecar에 `(2, 'recover_a')`, `(3, 'recover_b')` committed frame이 있고 data file에는 `seed_committed_one`만 있는 fixture | recovery replay 중 첫 committed frame 적용 후 cleanup/checkpoint 완료 전 | interruption 이후 같은 `SELECT` reopen command를 다시 실행 | `[(1, 'seed'), (2, 'recover_a'), (3, 'recover_b')]` | recovery 자체가 중단되어도 다음 reopen에서 모든 committed frame이 정확히 한 번 보이고 WAL replay는 idempotent합니다. | `crash-matrix-case-CM-005` |
| CM-006 | corrupt-tail-after-committed-frame | `seed_committed_one`과 `(2, 'committed_before_tail')` committed WAL frame 뒤에 deterministic corrupt tail이 붙은 fixture | committed frame 뒤 trailing garbage 또는 invalid length tail을 남긴 상태 | `cargo run --bin db -- <db_path> "SELECT * FROM items ORDER BY id"` | `[(1, 'seed'), (2, 'committed_before_tail')]` | committed prefix는 replay되고 corrupt tail은 user-facing CLI output 변경 없이 안전하게 무시되거나 documented error 조건으로 처리됩니다. error/output 변화가 있으면 `docs/cli_contract.md` 갱신이 필수입니다. | `crash-matrix-case-CM-006` |

## Candidate Acceptance Criteria
- `cargo test --test crash_matrix`와 `./scripts/verify_crash_matrix`가 write, WAL append, commit marker, incomplete-tail, corrupt-tail, recovery replay 경계를 검증합니다.
- 각 matrix case는 고정 seed 또는 명명된 fixture를 사용하고, 실패 시 어떤 crash point가 깨졌는지 식별 가능한 output을 남깁니다.
- commit 완료 전 중단된 row는 reopen 후 보이지 않고, commit 완료 row는 reopen 후 deterministic `SELECT` output으로 확인됩니다.
- WAL sidecar replay는 반복 실행해도 idempotent하며 기존 CLI contract와 저장 포맷 호환성을 깨지 않습니다.
- `docs/file_format.md`에는 crash matrix가 검증하는 WAL sidecar compatibility note가 추가되거나, 기존 문서가 이미 충분하면 최종 리포트에 갱신 불필요 근거가 명시되어야 합니다.
- user-facing CLI error/output 변화가 없어야 합니다. 불가피한 변화가 있으면 `docs/cli_contract.md`와 관련 integration test가 함께 갱신되어야 합니다.
- baseline `./scripts/verify`, `cargo test --test crash_matrix`, `./scripts/verify_crash_matrix`가 통과하고, task contract의 expected evidence path가 존재해야 합니다.

## 검증 계획
- Required commands:
  - `./scripts/verify`
  - `cargo test --test crash_matrix`
  - `./scripts/verify_crash_matrix`
- Expected evidence paths:
  - `tests/crash_matrix.rs`
  - `tests/fixtures/crash_matrix/`
  - `scripts/verify_crash_matrix`
  - `docs/file_format.md`
  - `target/crash_matrix/crash_matrix_report.md`
  - scheduler final report artifact의 verification evidence section
- `target/crash_matrix/crash_matrix_report.md`는 각 `case_id`, required evidence id, reopen command, expected visible rows, actual visible rows, WAL/file-format assertion result, command exit status를 포함해야 합니다.
- `./scripts/verify`는 기존 `wal_recovery` regression을 포함한 baseline suite가 유지됨을 증명해야 합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
