# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: Transaction WAL recovery current-artifact evidence refresh에 대한 managed repo evidence summary와 필요한 최소 문서/테스트 delta.
- 생성 대상 테스트 또는 verification output: `scripts/verify`, `cargo test --test wal_recovery`, `scripts/verify_crash_matrix`의 현재 repo SHA 기준 command output.
- 생성 대상 리포트 업데이트: `specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, 필요 시 human-required blocker.
- 생성 대상 evidence 파일: `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/current-repo-sha.txt`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md`.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, CLI command output, persisted WAL/file evidence, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `scripts/verify`가 현재 repo SHA에서 통과합니다.
- `cargo test --test wal_recovery`가 현재 repo SHA에서 통과하고 committed replay, rollback/uncommitted exclusion, incomplete-tail exclusion, repeated recovery/idempotence를 requirement ID별로 매핑합니다.
- `scripts/verify_crash_matrix`가 현재 repo SHA에서 통과하고 checkpoint/log truncation crash-interruption safety를 직접 증명합니다.
- `db exec` 또는 기존 WAL verification helper가 생성한 WAL sidecar/reopen smoke evidence가 `REQ-8-committed-writes-survive-crash-and-35caf667`와 `REQ-9-provide-wal-or-equivalent-write-80297892`에 명시적으로 연결됩니다.
- final review 또는 evidence summary가 `REQ-8-*`와 `REQ-9-*` requirement IDs, verification commands, evidence artifact paths, current repo SHA를 포함합니다.
- scheduler `SUCCESS`, queue delta, run report, task status event는 보조 운영 로그일 뿐 artifact gate completion evidence가 아닙니다.

## Requirement Evidence Contract
| Requirement ID | Required command | Required evidence artifact | Completion condition | Blocker condition |
| --- | --- | --- | --- | --- |
| `REQ-8-begin-commit-and-rollback-provide-44e7901f` | `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | rollback 또는 uncommitted WAL frame이 public read 결과에서 제외됨을 현재 SHA에서 증명합니다. | test 부재, 실패, 또는 requirement ID와 evidence path 미매핑입니다. |
| `REQ-8-committed-writes-survive-crash-and-35caf667` | `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli` | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | committed write가 reopen/recovery 후 조회 가능함을 증명합니다. | WAL sidecar/reopen smoke 또는 command output이 현재 SHA와 연결되지 않습니다. |
| `REQ-9-provide-wal-or-equivalent-write-80297892` | `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli` 및 `scripts/verify` | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/current-repo-sha.txt` | WAL 또는 동등한 write-ahead persisted evidence가 recovery path와 연결됨을 증명합니다. | persisted WAL/equivalent evidence identity가 기록되지 않습니다. |
| `REQ-9-recovery-must-be-idempotent-and-300531dc` | `cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` 및 `cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable` | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md` | repeated recovery가 중복 적용 없이 idempotent하고 incomplete tail cleanup 후 committed frame이 유지됨을 증명합니다. | idempotence evidence가 incomplete-tail evidence와 분리되어 requirement ID에 매핑되지 않습니다. |
| `REQ-9-checkpoint-or-log-truncation-must-d633d286` | `scripts/verify_crash_matrix` 및 `cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically` | `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/requirement-evidence.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md`, `specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md` | checkpoint/log truncation interruption이 committed data loss 없이 deterministic recovery 또는 deterministic failure로 처리됨을 증명합니다. | `scripts/verify_crash_matrix` 부재, 실패, 미실행, 또는 해당 safety 직접 증거 부재입니다. |

## Non-Visual Evidence Contract
- 이 package는 visual-risk task가 아닙니다.
- DOM capture, rendered route state, screenshot, UX design review는 not-applicable입니다.
- Deterministic evidence는 CLI command output, persisted file/WAL sidecar evidence, crash/reopen verification artifact, current repo SHA 기록으로 한정합니다.

## 완료 정의
- 구현 또는 evidence refresh delta가 존재하거나 blocker가 명시되어야 합니다.
- Acceptance criteria가 requirement ID별로 충족되어야 합니다.
- Verification proof가 managed repo evidence artifact로 첨부되어야 합니다.
- Artifact delta가 final review 또는 evidence summary에 반영되어야 합니다.
- `scripts/verify_crash_matrix`가 checkpoint/log truncation safety를 증명하지 못하면 완료가 아니라 human-required blocker입니다.
