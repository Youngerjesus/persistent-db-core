# Persistent DB Core Progress

## Current State

`persistent-db-core` is bootstrapped as a Rust CLI skeleton. The next smallest implementation handoff should target `gap-v1-bootstrap-cli-contract`.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | missing_evidence | No SQL parser, schema catalog, or executor yet. |
| gap-v1-primary-btree-index | missing_evidence | No primary B-tree index yet. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | missing_evidence | No transaction or WAL recovery path yet. |
| gap-v1-deterministic-crash-matrix | missing_evidence | No deterministic crash matrix yet. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | missing_evidence | No `db check` invariant command yet. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |
