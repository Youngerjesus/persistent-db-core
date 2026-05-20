# Persistent DB Core History

## 2026-05-15

- Created `persistent-db-core` as a V1 managed repo for CAO Autopilot.
- Initial product boundary is a Rust CLI binary named `db`.
- No V1 implementation gaps have verified completion evidence yet.

## 2026-05-17

- Added the minimal SQL schema/execute milestone: `db exec <path> <sql>` now supports the documented `CREATE TABLE`, `INSERT`, and `SELECT *` path with deterministic persistence and error-contract coverage.
- Added the primary-key index milestone: `db exec` now supports single `INT PRIMARY KEY` tables with duplicate-key rejection, exact lookup, key-ordered scans, and reopen-safe in-memory index rebuild from durable row records.

## 2026-05-18

- Added the minimal WAL recovery milestone: committed `db exec` mutations are replay-safe across reopen, incomplete trailing WAL entries are excluded, and the retained sidecar format is documented.
- Reverified the WAL recovery milestone at the current task SHA with separate committed, rolled-back/uncommitted, incomplete-tail, CLI smoke, and retained sidecar evidence.
- Added deterministic WAL crash matrix coverage for pre-append, partial-frame, uncommitted, committed replay, interrupted recovery, and corrupt-tail boundaries.
- Added the `db check` invariant milestone: existing database files can now be validated for page readability, SQL catalog/row consistency, primary-index rebuildability, WAL replay consistency, and stable open/read error behavior.
- Added SQLite-backed differential/property coverage for the supported SQL subset with deterministic seed replay, duplicate-key and missing-lookup checks, ordered scan comparison, and task-specific verification.
- Added the V1 benchmark and acceptance documentation milestone: `scripts/verify_bench_acceptance` records deterministic lower-bound evidence, and the V1 acceptance guide maps launch gates to evidence and explicit blockers.

## 2026-05-19

- Added the secondary-index milestone: `CREATE INDEX` now persists `INT` secondary indexes, uses indexed equality and inclusive bounded range paths with deterministic ordering, survives reopen/WAL replay, and is covered by `db check` invariant validation.
- Added the secondary-index mutation milestone: primary-key-targeted `UPDATE` and `DELETE` now keep table rows, primary indexes, and secondary indexes consistent across restart, retained WAL replay, WAL-only replay, and `db check` invariant validation.
- Added the Section 14 benchmark acceptance milestone: public `db bench` now generates 100000-row evidence with index-vs-scan lower bounds, WAL recovery proportionality, hard-fail validation, and requirement-traceable performance documentation.

## 2026-05-20

- Refreshed disk page-storage current-artifact evidence for `gate-v1-disk-page-storage`, tying 4096-byte page layout, restart durability, live-file append visibility, and bounded same-page write behavior to focused and baseline verification.
- Refreshed primary-index current-artifact evidence for `gate-v1-indexes`, tying integer primary-key lookup, key-ordered scan, duplicate-key rejection, valid duplicate persisted-row failure, and V1 acceptance traceability to focused and baseline verification.

## 2026-05-21

- Refreshed transaction WAL recovery current-artifact evidence for `gate-v1-transactions-wal-recovery`, tying `REQ-8-*` and `REQ-9-*` to focused WAL tests, baseline verification, crash matrix evidence, WAL sidecar smoke proof, and final review traceability.
