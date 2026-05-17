# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, and `db check` invariant validation for existing database files. The next smallest implementation handoff should target secondary indexes, differential/property testing, or benchmark/acceptance docs on top of the SQL execution, recovery, and check baselines.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | verification_ready | Primary-key tables rebuild an in-memory B-tree index from durable row records, support exact lookup, scan in primary-key order, and preserve row-only table compatibility. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | verification_ready | Current-SHA WAL sidecar replay proof covers committed mutation survival, rolled-back/uncommitted frame absence, incomplete-tail exclusion, and retained sidecar state after reopen. |
| gap-v1-deterministic-crash-matrix | verification_ready | Deterministic crash matrix covers pre-WAL append, partial WAL frame, uncommitted frame, committed replay idempotence, interrupted recovery retry, and corrupt tail cleanup evidence. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | verification_ready | `db check <path>` validates existing page records, SQL catalog/row invariants, primary-key rebuildability, WAL sidecar ordering, missing paths, and directory-path open/read errors. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |

## Recent Entries

- 2026-05-18: Added `db check <path>` invariant validation with exact success/failure CLI contracts, deterministic corrupted fixtures for storage, catalog/record, primary-index, and WAL replay consistency failures, plus focused `cargo test --test db_check` coverage.
- 2026-05-18: Added deterministic WAL crash matrix evidence for six recovery boundaries, including partial/corrupt tails, uncommitted frame exclusion, committed replay idempotence, and interrupted recovery retry with `scripts/verify_crash_matrix`.
- 2026-05-18: Reverified WAL recovery at current SHA with focused WAL tests, baseline `scripts/verify`, CLI reopen smoke, retained WAL sidecar byte evidence, and explicit mapping to `req-v1-wal-recovery-proof`.
- 2026-05-18: Added minimal transaction WAL recovery evidence: committed `db exec` mutations survive reopen, incomplete trailing WAL entries are excluded, retained WAL replay is idempotent, and the WAL sidecar format is documented.
- 2026-05-17: Added primary-key indexed query evidence for `db exec`: single `INT PRIMARY KEY` declarations, duplicate-key rejection, exact lookup, primary-key ordered scans, reopen/rebuild coverage, row-only compatibility, and primary-index persistence docs.
- 2026-05-17: Implemented the minimal SQL schema/execute path for `db exec <path> <sql>`, including parser/executor, SQL logical records over `PageStore`, exact CLI error contracts, restart and mid-command failure coverage, and SQL/file-format docs.
