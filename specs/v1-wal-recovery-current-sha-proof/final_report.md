# WAL Recovery Current-SHA Evidence

### EV-PROVENANCE
implementation_active_run_id: impl_retry_0_resume_20260518_013345_481129_3fa984a5
implementation_result_path: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/autopilot/project_manager/tasks/task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof/runs/impl_retry_0_resume_20260518_013345_481129_3fa984a5/result.md
current-run generation: This report was regenerated during final retry 2 after final verification found that inline identity/status values and linked transcript files disagreed. Prior `impl_exec` evidence remains historical only and is not used for identity/status proof.
evidence_root: specs/v1-wal-recovery-current-sha-proof/evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/

### EV-IDENTITY-HEAD
command: git rev-parse HEAD
exit_code: 0
stdout: "5f55110307d2f57c6a809d48409df06385ef9133"
stderr: ""
transcripts: evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_head.stdout, git_head.stderr, git_head.exit

### EV-IDENTITY-STATUS
command: git status --short
exit_code: 0
stdout:
```
 M specs/v1-wal-recovery-current-sha-proof/final_review.history.md
 M specs/v1-wal-recovery-current-sha-proof/final_review.md
?? specs/v1-wal-recovery-current-sha-proof/evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/
```
stderr: ""
transcripts: evidence/final_retry_2_resume_20260518_021150_286954_dbddfabb/git_status_short.stdout, git_status_short.stderr, git_status_short.exit
note: This status transcript was captured at final retry 2 entry before repairing the latest final-family SSOT and adding this retry evidence. The committed product code delta remains scoped to `tests/wal_recovery.rs`; finish documentation sync updated `work_queue/progress.md` and `docs/history_archives/history.md`; task-scoped spec/evidence artifacts remain under `specs/v1-wal-recovery-current-sha-proof/`.

### EV-TEST-WAL
command: cargo test --test wal_recovery
exit_code: 0
transcripts: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/cargo_test_wal_recovery.stdout, cargo_test_wal_recovery.stderr, cargo_test_wal_recovery.exit
stdout summary:
```
running 5 tests
test committed_wal_frame_ahead_of_page_store_fails_deterministically ... ok
test committed_wal_replay_survives_reopen_via_cli ... ok
test rolled_back_wal_frame_is_not_replayed_as_uncommitted_change ... ok
test incomplete_wal_entry_is_not_replayed_without_public_rollback_cli ... ok
test committed_frame_after_incomplete_tail_cleanup_remains_replayable ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
stderr: cargo test harness status only; no test failure output.

### EV-VERIFY-BASE
command: ./scripts/verify
exit_code: 0
covered checks: cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test; cargo run --bin db -- --help
transcripts: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/scripts_verify.stdout, scripts_verify.stderr, scripts_verify.exit
stdout/stderr summary:
```
cargo fmt --check completed.
cargo clippy --all-targets -- -D warnings completed without warnings.
cargo test completed with the full integration suite, including 5 wal_recovery tests.
cargo run --bin db -- --help exited 0 and printed the documented help text.
```

### EV-SMOKE-CREATE-INSERT
command: cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"
execution env: CARGO_TERM_QUIET=true; DB_PATH=<temp>/wal_smoke.pdb
exit_code: 0
stdout: ""
stderr: ""
transcripts: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/smoke_create_insert.stdout, smoke_create_insert.stderr, smoke_create_insert.exit

### EV-WAL-AFTER-CREATE-INSERT
path: "$DB_PATH.wal"
exists: true
byte_length: 202
transcript: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/wal_after_create_insert.txt

### EV-SMOKE-REOPEN-SELECT
command: cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"
execution env: CARGO_TERM_QUIET=true; DB_PATH=<same temp>/wal_smoke.pdb
exit_code: 0
stderr: ""
stdout: "id|name\n1|ada\n2|bea\n"
transcripts: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/smoke_reopen_select.stdout, smoke_reopen_select.stderr, smoke_reopen_select.exit

### EV-WAL-AFTER-REOPEN-SELECT
path: "$DB_PATH.wal"
exists: true
byte_length: 202
transcript: evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/wal_after_reopen_select.txt

### EV-FIXTURE-RATIONALE
V1 has no public rollback command and no public command that can intentionally leave an incomplete transaction or incomplete WAL frame. Direct WAL fixtures are therefore the only deterministic way to represent V1-observable recovery-boundary bytes that may exist after an interrupted writer. `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` covers a complete uncommitted/rolled-back `9|ghost` frame independently from truncation. `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` covers an incomplete trailing `9|ghost` frame. Both recover through the public `db exec` reopen path and prove the ghost row is absent; the incomplete-tail cleanup path is separately covered by `committed_frame_after_incomplete_tail_cleanup_remains_replayable`.

### EV-ACCEPTANCE-MAPPING
- gap-v1-transaction-wal-recovery: Current task branch SHA `5f55110307d2f57c6a809d48409df06385ef9133` was reverified in final retry 2 with source-backed HEAD/status transcripts, focused WAL tests, baseline verification, CLI reopen smoke, and retained WAL sidecar byte-state evidence. The original stale-proof target SHA was `33b480cac6cf9d505a86eda4c149a4471454f11d`; the committed proof branch contains that implementation state plus the WAL proof artifact delta and final evidence provenance corrections.
- gate-v1-transactions-wal-recovery: Evidence layers now separately cover committed mutation survival, complete rolled-back/uncommitted frame absence, incomplete trailing WAL exclusion, retained-frame idempotence, deterministic ahead-of-store failure, and complete-frame sidecar retention.
- req-v1-wal-recovery-proof: `committed_wal_replay_survives_reopen_via_cli` proves committed changes survive process reopen; `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` proves complete uncommitted/rolled-back changes are absent; `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` proves incomplete trailing WAL ghost absence; `committed_frame_after_incomplete_tail_cleanup_remains_replayable` proves cleanup keeps future replay possible; `committed_wal_frame_ahead_of_page_store_fails_deterministically` proves dependency/order corruption fails deterministically.
