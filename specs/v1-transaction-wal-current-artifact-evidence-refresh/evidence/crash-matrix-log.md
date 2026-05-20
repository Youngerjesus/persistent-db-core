# Crash Matrix Log

Current repo SHA: `bed51c0d35f392458840870401f304a157a3b005`

command: scripts/verify_crash_matrix
exit_code: 0
report_path: target/crash_matrix/crash_matrix_report.md
validator_outcome: PASS

The generated report records `command: cargo test --test crash_matrix` and all
case outputs below.

## Case Summaries

- CM-001 pre-wal-append: expected and actual visible rows were `id|name`, `1|seed`; WAL/file-format assertion result PASS because WAL sidecar was absent or empty and seed data remained unchanged; exit status `Some(0)`.
- CM-002 partial-wal-frame: expected and actual visible rows were `id|name`, `1|seed`; WAL/file-format assertion result PASS because incomplete WAL header or payload tail was ignored/truncated without panic; exit status `Some(0)`.
- CM-003 wal-frame-without-commit-marker: expected and actual visible rows were `id|name`, `1|seed`; WAL/file-format assertion result PASS because the absent commit marker maps to rollback state and is not replayed; exit status `Some(0)`.
- CM-004 committed-wal-before-data-apply: expected and actual visible rows were `id|name`, `1|seed`, `2|committed_wal`; WAL/file-format assertion result PASS because committed WAL replay was idempotent across first and second reopen; first and second reopen exit status `Some(0)`.
- CM-005 recovery-interrupted-after-first-apply: expected and actual visible rows were `id|name`, `1|seed`, `2|recover_a`, `3|recover_b`; WAL/file-format assertion result PASS because recovery re-entry applied every committed frame exactly once; interrupted reopen exit status `Some(101)`, subsequent recovery reopen exit statuses `Some(0)`.
- CM-006 corrupt-tail-after-committed-frame: expected and actual visible rows were `id|name`, `1|seed`, `2|committed_before_tail`; WAL/file-format assertion result PASS because the committed WAL prefix was replayed and incomplete/invalid-length tail was ignored without CLI output change; exit status `Some(0)`.
