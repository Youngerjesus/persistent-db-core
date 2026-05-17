# Research: `db check` Invariant Validation

## Decision 1: Add a Dedicated Checker Module
- Decision: introduce a small public checker surface, likely `src/check.rs` exported from `src/lib.rs`, rather than placing invariant logic in `main.rs`.
- Rationale: CLI formatting belongs in `src/main.rs`; invariant traversal needs access to storage records, SQL logical validation, rebuilt primary index consistency, and WAL consistency. A module keeps testable domain logic separate from process exit handling.
- Constraint: no new dependency is needed.

## Decision 2: Preserve `PageStore::open` Mutation Semantics For `db exec`, But Avoid Mutation For `db check`
- Current reality: `PageStore::open` creates missing files and calls WAL replay, which can append records and truncate incomplete WAL tails.
- Decision: `db check <path>` must not create a missing DB file and should not mutate the DB or WAL while checking. Add read-only validation helpers in `storage.rs` for checker use.
- Rationale: the contract requires missing paths to fail as user-facing errors, and invariant checking should report persisted state rather than repair or replay it as a side effect.
- Implementation implication: expose read-only helpers such as validated record scan, durable record count, WAL sidecar path, and WAL frame validation/classification. Keep existing mutating `PageStore::open` behavior unchanged for `db exec`.

## Decision 3: Represent Check Failures With Stable Invariant Labels
- Decision: checker errors should carry stable labels used by CLI stderr:
  - `storage record readability`
  - `catalog/record invariant`
  - `primary index`
  - `wal replay consistency`
  - `open/read`
- Rationale: tests and docs need stable matching without over-coupling to Rust enum debug output.
- CLI contract: success prints exactly `ok: db check passed\n`; check invariant failures print `error: db check failed: <label>: <detail>\n`; open/read errors outside invariant checking may print `error: <stable open/read wording>\n`.

## Decision 4: Reuse SQL Logical Decode Semantics Through A Public Validation API
- Current reality: `Database::from_records`, `decode_record`, and `validate_catalog_record_invariants` are private in `src/sql.rs`; they already enforce catalog/row invariants and duplicate primary-key detection.
- Decision: expose a narrow SQL validation function or checker-oriented summary that validates records and returns enough metadata to verify primary-key index consistency.
- Rationale: duplicating SQL logical parsing in the checker would risk drift from `db exec`.
- Implementation implication: a function such as `sql::validate_records_for_check(records: &[Vec<u8>]) -> Result<CheckSqlSummary, CheckSqlError>` can remain internal-detail-aware while keeping caller logic simple.

## Decision 5: Primary Index Consistency Means Rebuild Parity, Not Separate Persisted Index Files
- Current reality: primary indexes are in-memory only; docs state no separate persisted index metadata exists.
- Decision: `primary index` invariant should prove rebuilt key set/row-position mapping from durable records is complete and duplicate-free. Duplicate persisted primary keys and invalid mapping construction are failures under the `primary index` label, even though no separate index file can be missing.
- Rationale: this satisfies the contract without inventing out-of-scope persisted index metadata.
- Documentation note: `docs/file_format.md` should clarify that V1 `db check` validates rebuildability/uniqueness, not a separate index sidecar.

## Decision 6: WAL Ahead-Of-Store Must Be Detected Read-Only
- Current reality: `replay_wal` returns `StorageError::CorruptRecordLength` when a committed frame's `record count before` is greater than durable record count. Existing `tests/wal_recovery.rs` covers this through `db exec`.
- Decision: `db check` should parse the documented sidecar and fail with `wal replay consistency` when a complete committed page-append frame is ahead of durable state. It should not rely on storage-only corruption or absent WAL evidence.
- Fixture requirement: `tests/db_check.rs` must create a database file first, then write `<database-path>.wal` with a complete committed `0x01` page-append frame whose `record count before` is greater than current durable record count.

## Decision 7: Verification Is Baseline Plus Focused Test
- Required commands:
  - `scripts/verify`
  - `cargo test --test db_check`
- Evidence to record after implementation: command outputs, deterministic fixture generation in `tests/db_check.rs`, doc diff for CLI/file-format contract.
