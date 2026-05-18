# Final Review History

## 2026-05-19 pre-finish implementation-ready report

# Final Implementation Report: v1-secondary-index-range-scan

Verdict: implementation-side ready for verifier gate after `impl_retry_0`.

## Repair Diagnosis

- F-001 primary-key-column secondary index false green:
  - Finding: `CREATE INDEX idx_users_id ON users(id); SELECT * FROM users WHERE id BETWEEN 1 AND 2;` was accepted at index creation time but later planned/executed as unsupported SQL.
  - Root cause: query path resolution checked the primary-key branch before secondary indexes. Primary-key equality worked, but primary-key range rejected before the explicit secondary index could be considered.
  - Repair: added focused coverage for explicit secondary indexes on primary-key columns, then changed planner/executor resolution to prefer a committed secondary index on the predicate column when present. This repairs the routing boundary and proves both `SecondaryIndexEquality` and `SecondaryIndexRange`.

- F-002 metadata invariant evidence gap:
  - Finding: docs claimed non-`INT` metadata columns and duplicate committed index names fail `db check`, but tests did not name those exact fixture cases.
  - Root cause: implementation had validation branches, but the corruption matrix was narrower than the documented invariant list.
  - Repair: extended `db_check_secondary_index_corruption_matrix_reports_secondary_index` with `non_int_column_metadata` and case-insensitive `duplicate_index_metadata` fixtures.

- F-003 post-commit matching `E` interpretation:
  - Finding: matching `E(build_id,index_name)` records appended after a committed `X` were not explicitly tested and were ignored by the loader.
  - Root cause: metadata attach read pending entries with `get` and final validation rebuilt the in-memory index from rows, so later matching `E` records remained unclassified.
  - Repair: metadata attach now consumes matching pending entries, tracks committed `(index_name, build_id)` pairs, and fails `secondary index` if matching pending entries remain after replay. Added `db_check_reports_secondary_index_for_matching_entry_appended_after_commit`.

- F-004 secondary-index WAL-only replay evidence:
  - Finding: process reopen evidence existed, but secondary-index-specific WAL replay was indirect through generic WAL tests.
  - Root cause: focused secondary-index suite did not directly author committed WAL frames containing `E...X` or `I` records.
  - Repair: added `committed_wal_replay_applies_secondary_backfill_entries_and_metadata` and `committed_wal_replay_applies_atomic_indexed_row_record`.

- F-005 performance/maintainability risk:
  - Finding: reopen/check rebuild paths and multi-index insert have avoidable repeated work.
  - Root cause: V1 implementation favors simple deterministic reconstruction and small-scope correctness over index-count optimization.
  - Repair decision: no code change in this retry. The brake report marks this as verify-risk, not verify-blocking, and the acceptance contract does not set performance thresholds. The risk remains visible for verifier judgment.

## Command Evidence

### `cargo test --test secondary_index -- --nocapture`
- Exit code: 0
- Stdout summary: compiled and ran `tests/secondary_index.rs`; 21 passed, 0 failed.
- Stderr summary: normal cargo progress only; no errors.

### `scripts/verify`
- Exit code: 0
- Stdout summary: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help` completed successfully.
- Stderr summary: normal cargo progress only; no errors.

## Requirement Mapping

`REQ-7-create-index-must-create-disk-3b71a7dc` was mapped to CLI examples, exact semantic errors, index path use evidence, persisted compatibility evidence, `db check` evidence, and docs evidence. Sibling `gate-v1-indexes` requirements were not closed by inference. This report maps only `REQ-7-create-index-must-create-disk-3b71a7dc`.

This is a non-visual CLI/database task; DOM capture, screenshots, rendered route state, and UX design review evidence were not used as acceptance evidence.
