# Transaction WAL recovery current-artifact evidence refresh

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-20-23-32-28
- Task ID: task-2026-05-20-23-32-28-v1-transaction-wal-current-artifact-evidence-refresh
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: Transaction WAL recovery current-artifact evidence refresh
- Artifact: v1-transaction-wal-current-artifact-evidence-refresh

## 목표
- 현재 WAL recovery 구현과 과거 task evidence는 존재하지만 Root Progress Projection은 `gate-v1-transactions-wal-recovery`의 source-bound requirement row를 open으로 보고 있습니다. V1 완료 판단은 scheduler SUCCESS가 아니라 current artifact requirement ID별 evidence에 묶여야 하므로, transaction/WAL recovery 증거를 현재 artifact 계약에 맞춰 재검증해야 합니다.

## 지금 해야 하는 이유
- CLI와 SQL 후보는 conflicting_evidence 또는 human request로 자동 후보가 막혀 있고, index gate는 projected_complete라 제외됩니다. WAL recovery gate는 V1 신뢰성의 핵심이며 기존 `cargo test --test wal_recovery`, `scripts/verify`, WAL sidecar evidence, final review refs가 있어 작은 current-artifact evidence refresh로 검증 가능성이 높습니다.

## 기대 산출물 변화
- `REQ-8-*`와 `REQ-9-*` requirement ID별로 committed-only replay, rollback/uncommitted invisibility, idempotent recovery, checkpoint/log truncation safety를 현재 repo SHA에서 재검증하고, canonical spec package와 managed repo evidence 문서가 current-artifact matcher가 읽을 수 있는 requirement ID, command, artifact refs를 명시하게 됩니다.

## 의도한 변경 대상
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/spec.md
- project_manager/specs/v1-transaction-wal-current-artifact-evidence-refresh/contracts.md
- specs/v1-transaction-wal-current-artifact-evidence-refresh/
- tests/wal_recovery.rs
- scripts/verify
- docs/cli_contract.md
- docs/file_format.md
- docs/v1_acceptance.md

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: bed51c0d35f392458840870401f304a157a3b005
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: tests/wal_recovery.rs, scripts/verify, docs/cli_contract.md, docs/file_format.md, docs/v1_acceptance.md, specs/v1-transaction-wal-recovery/final_review.md, specs/v1-wal-recovery-current-sha-proof/analysis_report.md, specs/v1-wal-recovery-current-sha-proof/code_review.md, specs/v1-wal-recovery-current-sha-proof/contracts.md, specs/v1-wal-recovery-current-sha-proof/design.md, specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.exit, specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.stderr

## Risk flags
- data_loss_risk_review_required
- checkpoint_truncation_evidence_may_be_insufficient

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `gate-v1-transactions-wal-recovery` status=open, missing requirement IDs include `REQ-8-begin-commit-and-rollback-provide-44e7901f`, `REQ-8-committed-writes-survive-crash-and-35caf667`, `REQ-9-provide-wal-or-equivalent-write-80297892`, `REQ-9-recovery-must-be-idempotent-and-300531dc`, `REQ-9-checkpoint-or-log-truncation-must-d633d286`.
- Root Progress Projection: `gap-v1-transaction-wal-recovery` has prior evidence refs for `cargo test --test wal_recovery`, `./scripts/verify`, retained WAL sidecar smoke, `specs/v1-transaction-wal-recovery/final_review.md`, and `specs/v1-wal-recovery-current-sha-proof` but blocker says artifact contract digest does not match current artifact.
- Managed repo history on 2026-05-18 records minimal WAL recovery and current-SHA WAL recovery re-verification milestones.
- Queue Snapshot is empty, so no active or reserved duplicate task blocks this slice.
- Policy requires high confidence and no conflicting_evidence; this candidate avoids CLI/SQL conflicting slices and does not touch protected CAO areas.
- src/storage.rs
- tests/wal_recovery.rs
- tests/crash_matrix.rs
- docs/file_format.md
- docs/cli_contract.md
- docs/v1_acceptance.md
- scripts/verify
- scripts/verify_crash_matrix
- specs/v1-transaction-wal-recovery/spec.md
- specs/v1-transaction-wal-recovery/contracts.md
- specs/v1-transaction-wal-recovery/final_review.md
- specs/v1-wal-recovery-current-sha-proof/spec.md
- specs/v1-wal-recovery-current-sha-proof/contracts.md
- specs/v1-wal-recovery-current-sha-proof/final_report.md
- specs/v1-wal-recovery-current-sha-proof/final_review.md
- PageStore::open
- PageStore::append_record
- replay_wal
- append_wal_frame
- next_wal_frame_id
- wal_checksum
- committed_wal_replay_survives_reopen_via_cli
- rolled_back_wal_frame_is_not_replayed_as_uncommitted_change
- incomplete_wal_entry_is_not_replayed_without_public_rollback_cli
- committed_frame_after_incomplete_tail_cleanup_remains_replayable
- committed_wal_frame_ahead_of_page_store_fails_deterministically
- task-2026-05-17-23-45-17-v1-transaction-wal-recovery: SUCCESS
- task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof: SUCCESS

## 범위
- In scope: 현재 managed repo SHA에서 WAL recovery requirement evidence를 다시 생성하고, `REQ-8-*`와 `REQ-9-*` requirement row가 exact-match로 닫힐 수 있도록 command, expected behavior, evidence artifact path를 명시합니다.
- In scope: `scripts/verify`, `cargo test --test wal_recovery`, `scripts/verify_crash_matrix` 실행 결과와 WAL sidecar/reopen smoke evidence를 managed repo 내부 evidence summary에 연결합니다.
- Out of scope: unrelated breadth features, scheduler/control-plane completion을 artifact evidence로 대체하는 변경, visual/UI evidence 생성.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.
- scheduler `SUCCESS`, queue delta, run report만으로는 artifact gate completion을 주장할 수 없습니다.

## Candidate Acceptance Criteria
- `scripts/verify`가 현재 repo SHA에서 통과하고 baseline evidence command로 기록됩니다.
- `cargo test --test wal_recovery`가 현재 repo SHA에서 통과하고 committed replay, rollback/uncommitted exclusion, incomplete-tail exclusion, repeated recovery/idempotence를 requirement ID별로 매핑합니다.
- `scripts/verify_crash_matrix`가 현재 repo SHA에서 통과하고 checkpoint/log truncation crash-interruption safety evidence로 기록됩니다.
- `db exec` 또는 기존 WAL verification helper가 생성한 WAL sidecar/reopen smoke evidence가 `REQ-8-committed-writes-survive-crash-and-35caf667`와 `REQ-9-provide-wal-or-equivalent-write-80297892`에 명시적으로 연결됩니다.
- final review 또는 evidence summary가 `REQ-8-*`와 `REQ-9-*` requirement IDs, verification commands, evidence artifact paths, current repo SHA를 포함합니다.
- `scripts/verify_crash_matrix`가 없거나 실패하거나 checkpoint/log truncation safety를 직접 증명하지 못하면, 이 package는 completion이 아니라 package-level blocker로 종료하고 human-required decision을 남깁니다.

## Requirement Evidence Matrix
| Requirement ID | Verification command | Expected behavior | Evidence artifact path | Fallback blocker condition |
| --- | --- | --- | --- | --- |
| `REQ-8-begin-commit-and-rollback-provide-44e7901f` | `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` | rollback 또는 uncommitted WAL frame은 reopen/recovery 후 public read 결과에 나타나지 않습니다. | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md` 및 `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | test가 없거나 실패하거나 rollback/uncommitted exclusion을 직접 검증하지 않으면 blocker입니다. |
| `REQ-8-committed-writes-survive-crash-and-35caf667` | `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli` | committed write는 crash/reopen 또는 process 재시작 후에도 replay되어 조회 가능합니다. | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | committed replay smoke가 현재 SHA에서 생성되지 않거나 WAL sidecar/reopen 경로가 기록되지 않으면 blocker입니다. |
| `REQ-9-provide-wal-or-equivalent-write-80297892` | `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli` 및 `scripts/verify` | durable write path가 WAL 또는 동등한 write-ahead evidence를 만들고 recovery가 그 evidence를 소비합니다. | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/current-repo-sha.txt` | WAL sidecar 또는 동등한 persisted evidence artifact가 확인되지 않으면 blocker입니다. |
| `REQ-9-recovery-must-be-idempotent-and-300531dc` | `cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` 및 `cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable` | 같은 store를 반복 open/recover해도 이미 적용된 committed frame은 중복 적용되지 않고 incomplete tail cleanup 이후 committed frame은 재현 가능합니다. | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md` 및 `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | repeated recovery/idempotence가 별도 evidence로 분리되지 않거나 incomplete tail cleanup과 committed replay가 함께 증명되지 않으면 blocker입니다. |
| `REQ-9-checkpoint-or-log-truncation-must-d633d286` | `scripts/verify_crash_matrix` 및 `cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically` | checkpoint 또는 log truncation 중단 상황은 committed data loss 없이 deterministic recovery 또는 deterministic failure로 처리됩니다. | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md` | `scripts/verify_crash_matrix`가 없거나 실패하거나 checkpoint/log truncation interruption safety를 직접 다루지 않으면 human-required blocker입니다. |

## 검증 계획
- Required commands:
  - `scripts/verify`
  - `cargo test --test wal_recovery`
  - `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli`
  - `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`
  - `cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`
  - `cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable`
  - `cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically`
  - `scripts/verify_crash_matrix`
- Required evidence artifacts:
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/current-repo-sha.txt`
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md`
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md`
  - `specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md`
- 기대 증거는 managed repo 내부 command output, current repo SHA, requirement ID별 evidence mapping, WAL sidecar/reopen evidence, crash/reopen verification artifact입니다.
- scheduler outcome, queue delta, control-plane run report는 보조 운영 로그일 뿐 current artifact evidence를 대체하지 않습니다.

## Non-Visual Verification Evidence
- 이 작업은 Rust CLI DB의 WAL recovery evidence refresh이며 visual task가 아닙니다.
- DOM capture, rendered route state, screenshot, UX design review는 not-applicable입니다.
- 필요한 deterministic evidence는 CLI command output, persisted file/WAL sidecar evidence, crash/reopen verification artifact, current repo SHA 기록으로 한정합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- `scripts/verify_crash_matrix` 또는 동등한 checkpoint/log truncation interruption safety evidence가 없으면 자동 completion을 주장하지 말고 human-required blocker로 escalate합니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
