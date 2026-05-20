# Requirement Evidence

Current repo SHA: `bed51c0d35f392458840870401f304a157a3b005`

## REQ-8-begin-commit-and-rollback-provide-44e7901f

command: cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change
expected_behavior: rollback 또는 uncommitted WAL frame은 reopen/recovery 후 public read 결과에 나타나지 않습니다.
observed_result: exit_code 0; focused test passed with 1 passed, 0 failed, and 4 filtered out. Supporting suite evidence also shows `cargo test --test wal_recovery` passed all 5 tests.
artifact_refs: specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md
blocker_status: none

## REQ-8-committed-writes-survive-crash-and-35caf667

command: cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli
command: cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"
command: cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"
expected_behavior: committed write는 crash/reopen 또는 process 재시작 후에도 replay되어 조회 가능합니다.
observed_result: exit_code 0 for the focused WAL test; the build-coupled separate-process smoke create/insert exited 0 with empty stdout/stderr, reopen/select exited 0 with stdout `id|name\n1|ada\n2|bea\n`, and `$DB_PATH.wal` existed with 202 bytes after both mutation and reopen.
artifact_refs: specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md; specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md
blocker_status: none

## REQ-9-provide-wal-or-equivalent-write-80297892

command: cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli
command: scripts/verify
expected_behavior: durable write path가 WAL 또는 동등한 write-ahead evidence를 만들고 recovery가 그 evidence를 소비합니다.
observed_result: `scripts/verify` exited 0 and included the full WAL recovery suite; the focused committed replay test exited 0; WAL smoke recorded `$DB_PATH.wal` as the persisted write-ahead evidence with positive byte length before and after reopen.
artifact_refs: specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/current-repo-sha.txt; specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md; specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/wal-sidecar-smoke.md
blocker_status: none

## REQ-9-recovery-must-be-idempotent-and-300531dc

command: cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli
command: cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable
expected_behavior: 같은 store를 반복 open/recover해도 이미 적용된 committed frame은 중복 적용되지 않고 incomplete tail cleanup 이후 committed frame은 재현 가능합니다.
observed_result: both focused tests exited 0 with 1 passed, 0 failed, and 4 filtered out. Distinct support from crash matrix `CM-004` showed committed WAL replay idempotent across first and second reopen, and `CM-005` showed interrupted recovery re-entry applied every committed frame exactly once.
artifact_refs: specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md; specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md
blocker_status: none

## REQ-9-checkpoint-or-log-truncation-must-d633d286

command: scripts/verify_crash_matrix
command: cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically
expected_behavior: checkpoint 또는 log truncation 중단 상황은 committed data loss 없이 deterministic recovery 또는 deterministic failure로 처리됩니다.
observed_result: `scripts/verify_crash_matrix` exited 0 and generated `target/crash_matrix/crash_matrix_report.md` with `CM-001` through `CM-006` PASS case evidence; the ahead-of-page-store focused WAL test exited 0 and proves deterministic failure when WAL order is ahead of durable page-store state.
artifact_refs: specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/command-log.md; specs/v1-transaction-wal-current-artifact-evidence-refresh/evidence/crash-matrix-log.md; specs/v1-transaction-wal-current-artifact-evidence-refresh/final_review.md
blocker_status: none
