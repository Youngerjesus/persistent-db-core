# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, disk-backed secondary-index equality/range proof, mutation-maintained secondary-index UPDATE/DELETE proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, `db check` invariant validation for existing database files, SQLite-backed differential/property evidence for the supported SQL subset, and public `db bench` Section 14 100k benchmark acceptance evidence. The next smallest implementation handoff should target any remaining non-Section 14 V1 source-required obligations on top of the SQL execution, recovery, check, differential, benchmark, and index baselines.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | verification_ready | Primary-key tables rebuild an in-memory B-tree index from durable row records, support exact lookup, scan in primary-key order, and preserve row-only table compatibility. |
| gap-v1-secondary-index-range-scan | verification_ready | `CREATE INDEX` creates durable secondary `INT` indexes with indexed equality/range query paths, deterministic ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant evidence. |
| gap-v1-secondary-index-mutation-consistency | verification_ready | Primary-key-targeted UPDATE/DELETE maintain table rows, primary indexes, and secondary indexes across restart, retained WAL replay, WAL-only mutation replay, positive `db check`, and deterministic stale/dangling/missing secondary-index negative fixtures. |
| gap-v1-transaction-wal-recovery | verification_ready | Current-SHA WAL sidecar replay proof covers committed mutation survival, rolled-back/uncommitted frame absence, incomplete-tail exclusion, and retained sidecar state after reopen. |
| gap-v1-deterministic-crash-matrix | verification_ready | Deterministic crash matrix covers pre-WAL append, partial WAL frame, uncommitted frame, committed replay idempotence, interrupted recovery retry, and corrupt tail cleanup evidence. |
| gap-v1-differential-property-tests | verification_ready | SQLite-backed deterministic differential/property tests cover supported SQL subset generation, duplicate-key errors, missing lookups, ordered scans, seed replay, and failure artifact reporting. |
| gap-v1-db-check-invariants | verification_ready | `db check <path>` validates existing page records, SQL catalog/row invariants, primary-key rebuildability, WAL sidecar ordering, missing paths, and directory-path open/read errors. |
| gap-v1-bench-docs-acceptance | verification_ready | Public `db bench` and `scripts/verify_bench_acceptance` generate and validate Section 14 100000-row JSON evidence with index-vs-scan, recovery proportionality, hard-fail, and documentation traceability coverage. |

## Recent Entries

- 2026-05-19: Added public `db bench` Section 14 acceptance evidence for the fixed 100000-row workload, index-vs-scan lower bounds, WAL recovery proportionality, hard-fail verifier policy, CLI contract docs, performance report, V1 acceptance mapping, and bug diary traceability.
- 2026-05-19: Added mutation-maintained secondary-index proof for primary-key-targeted `UPDATE`/`DELETE`, including restart/reopen query evidence, retained and WAL-only replay coverage, positive `db check`, and deterministic stale/dangling/missing secondary-index negative fixtures.
- 2026-05-19: Added disk-backed secondary-index support for `CREATE INDEX`, indexed equality and inclusive `BETWEEN` range scans, deterministic key/tie-break ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant validation.
- 2026-05-18: Added repo-local benchmark acceptance evidence with `scripts/verify_bench_acceptance`, documented lower-bound policy in `docs/benchmarks.md`, and mapped V1 launch gates in `docs/v1_acceptance.md` with explicit missing-evidence blockers.
- 2026-05-18: Added SQLite-backed deterministic differential/property evidence for the supported SQL subset, including seed replay, duplicate-key and missing-lookup coverage, ordered scan comparison, local failure artifact reporting, and `scripts/verify_differential_property`.
- 2026-05-18: Added `db check <path>` invariant validation with exact success/failure CLI contracts, deterministic corrupted fixtures for storage, catalog/record, primary-index, and WAL replay consistency failures, plus focused `cargo test --test db_check` coverage.
- 2026-05-18: Added deterministic WAL crash matrix evidence for six recovery boundaries, including partial/corrupt tails, uncommitted frame exclusion, committed replay idempotence, and interrupted recovery retry with `scripts/verify_crash_matrix`.
- 2026-05-18: Reverified WAL recovery at current SHA with focused WAL tests, baseline `scripts/verify`, CLI reopen smoke, retained WAL sidecar byte evidence, and explicit mapping to `req-v1-wal-recovery-proof`.
- 2026-05-18: Added minimal transaction WAL recovery evidence: committed `db exec` mutations survive reopen, incomplete trailing WAL entries are excluded, retained WAL replay is idempotent, and the WAL sidecar format is documented.
- 2026-05-17: Added primary-key indexed query evidence for `db exec`: single `INT PRIMARY KEY` declarations, duplicate-key rejection, exact lookup, primary-key ordered scans, reopen/rebuild coverage, row-only compatibility, and primary-index persistence docs.
