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
