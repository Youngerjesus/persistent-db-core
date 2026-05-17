# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, and the minimal SQL schema/execute path for `CREATE TABLE`, `INSERT INTO ... VALUES`, and `SELECT * FROM ...`. The next smallest implementation handoff should target indexing, recovery, or validation gaps on top of the SQL execution baseline.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | missing_evidence | No primary B-tree index yet. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | missing_evidence | No transaction or WAL recovery path yet. |
| gap-v1-deterministic-crash-matrix | missing_evidence | No deterministic crash matrix yet. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | missing_evidence | No `db check` invariant command yet. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |

## Recent Entries

- 2026-05-17: Implemented the minimal SQL schema/execute path for `db exec <path> <sql>`, including parser/executor, SQL logical records over `PageStore`, exact CLI error contracts, restart and mid-command failure coverage, and SQL/file-format docs.
