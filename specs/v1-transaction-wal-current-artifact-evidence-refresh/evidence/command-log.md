# Command Log

Current repo SHA: `bed51c0d35f392458840870401f304a157a3b005`

## Baseline

command: scripts/verify
exit_code: 0
observed_output_summary:
- `cargo fmt --check` completed.
- `cargo clippy --all-targets -- -D warnings` completed.
- `cargo test` completed with all listed unit, integration, and doc tests passing.
- `cargo run --bin db -- --help` completed and printed the documented help text.
- Included `tests/wal_recovery.rs`: 5 passed.
- Included `tests/crash_matrix.rs`: 7 passed.

## WAL Recovery Suite

command: cargo test --test wal_recovery
exit_code: 0
observed_output_summary:
- `running 5 tests`
- `committed_wal_replay_survives_reopen_via_cli ... ok`
- `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change ... ok`
- `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli ... ok`
- `committed_frame_after_incomplete_tail_cleanup_remains_replayable ... ok`
- `committed_wal_frame_ahead_of_page_store_fails_deterministically ... ok`
- `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

command: cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli
exit_code: 0
observed_output_summary:
- `running 1 test`
- `committed_wal_replay_survives_reopen_via_cli ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

command: cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change
exit_code: 0
observed_output_summary:
- `running 1 test`
- `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

command: cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli
exit_code: 0
observed_output_summary:
- `running 1 test`
- `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

command: cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable
exit_code: 0
observed_output_summary:
- `running 1 test`
- `committed_frame_after_incomplete_tail_cleanup_remains_replayable ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

command: cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically
exit_code: 0
observed_output_summary:
- `running 1 test`
- `committed_wal_frame_ahead_of_page_store_fails_deterministically ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`

## Crash Matrix

command: scripts/verify_crash_matrix
exit_code: 0
observed_output_summary:
- `cargo test --test crash_matrix` completed.
- `running 7 tests`
- `crash_matrix_contract_lists_all_cases_and_evidence_ids ... ok`
- `cm_001_pre_wal_append_seed_only_visible ... ok`
- `cm_002_partial_wal_frame_is_ignored ... ok`
- `cm_003_wal_frame_without_commit_marker_is_not_visible ... ok`
- `cm_004_committed_wal_before_data_apply_is_idempotent ... ok`
- `cm_005_recovery_interrupted_after_first_apply_replays_remaining_once ... ok`
- `cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix ... ok`
- `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
