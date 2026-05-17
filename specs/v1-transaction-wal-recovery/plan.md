# Implementation Plan: v1-transaction-wal-recovery

## Goal
Add minimum WAL recovery evidence for the existing Rust CLI database: committed SQL mutations survive reopen through WAL replay, while rollback or incomplete WAL mutations do not become visible rows.

## Non-Goals
- Public transaction SQL such as `BEGIN`, `COMMIT`, or `ROLLBACK`.
- Multi-process concurrency, crash matrix expansion, fsync policy hardening, background recovery service, or network behavior.
- Secondary indexes or unrelated SQL grammar changes.
- Changes to protected `ssot/` or `policies/`.

## Affected Contract Surface
- CLI behavior: `db exec <path> <sql>` must preserve the existing stdout/stderr/exit-code contract.
- Persisted-data compatibility: existing page files without WAL must still open and read through current SQL behavior.
- Storage/file format: new WAL sidecar location, frame layout, replay semantics, and compatibility notes must be documented.
- Tests: new `tests/wal_recovery.rs` must cover Scenario A and Scenario B.
- Final evidence: required command output and WAL file-state summary must be captured by implementation phase reporting.

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| `src/storage.rs` | Add or expose minimal WAL support near page-store persistence if this remains the owning durable boundary | keep page format compatible; std only |
| `src/sql.rs` | Route successful SQL catalog/row mutations through WAL-aware append/replay path if SQL owns logical payload construction | do not change public SQL grammar |
| `src/lib.rs` | Export a new module only if implementation introduces one, such as `wal` or `recovery` | keep public surface narrow |
| `src/main.rs` | Prefer no change | CLI output and exit mapping should remain stable |
| `tests/wal_recovery.rs` | Add CLI Scenario A and deterministic Scenario B fixture | assert exit code, stdout, stderr exactly |
| `docs/file_format.md` | Add WAL compatibility note | must include location, framing, replay order, committed/rollback/incomplete behavior, old-file behavior |
| `docs/cli_contract.md` | Change only if CLI behavior changes | expected no change; final report must state why |

## Recovery Flow
1. `db exec` opens the database path.
2. WAL replay runs before reading SQL records for execution.
3. Replay scans sidecar WAL frames in order.
4. Each complete frame has fixed framing: magic `PDBWAL1\0`, version `1`, `u64` frame id, `u64` page-store record count before apply, state byte, payload-kind byte, `u32` payload length, `u32` wrapping byte-sum checksum, then payload bytes.
5. Complete committed append frames are applied only when the current page-store record count equals the frame's `record_count_before`; frames whose count is already behind the current page-store count are skipped as already applied.
6. Rollback frames and incomplete trailing frames are ignored.
7. A complete frame whose `record_count_before` is ahead of the current page-store count is treated as deterministic recovery/storage corruption.
8. SQL then reads page-store records and executes the requested statements.
9. Successful mutation statements write a committed append frame before appending the page-store record. WAL frames may be retained; idempotence is provided by `record_count_before`, not by truncating WAL.

## Test Strategy
1. Add `tests/wal_recovery.rs` first.
2. Scenario A uses separate `db exec` child processes:
   - create/insert command exits `0`, stdout `""`, stderr `""`;
   - reopen select command exits `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`.
3. Scenario B uses a deterministic fixture because the public CLI has no rollback command:
   - first create the table through CLI so the page store contains only the catalog record;
   - write `<db-path>.wal` with a complete committed append frame for SQL row `1|ada` using `record_count_before = 1`;
   - append an incomplete trailing frame for SQL row `9|ghost` using the same header/payload layout but with missing trailing bytes;
   - run a new `db exec <path> "SELECT * FROM users;"`, which replays the committed row and ignores the incomplete ghost row;
   - assert exit code `0`, stderr `""`, and stdout exactly `id|name\n1|ada\n`;
   - test name or comment explains why the fixture is storage-level rather than CLI-level.
4. Add file-state assertions: database path exists, WAL sidecar path exists with retained frames, repeated reopen/select still returns one row without duplicate replay, and no ghost row appears.
5. Update `docs/file_format.md` with the exact WAL framing above.

## Required Verification
- `cargo test`
- `cargo test --test wal_recovery`
- `./scripts/verify`
- Smoke:
  - `cargo run --bin db -- exec <temp-db> "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`
  - `cargo run --bin db -- exec <temp-db> "SELECT * FROM users;"`

## Acceptance Mapping
| Acceptance Item | Planned Evidence |
|---|---|
| `tests/wal_recovery.rs` exists | file added in implementation |
| Scenario A committed rows replay after reopen | `cargo test --test wal_recovery` CLI test with exact stdout/stderr/exit assertions |
| Scenario B rollback/incomplete row absent | deterministic WAL fixture test in `tests/wal_recovery.rs` |
| `cargo test` passes | implementation report command output |
| `./scripts/verify` passes | implementation report command output |
| `docs/file_format.md` covers WAL compatibility | manual review plus doc diff in implementation report |
| `docs/cli_contract.md` updated only if public behavior changes | final report states unchanged if no CLI contract delta |
| Smoke commands produce expected output | implementation report includes redacted temp path and output summary |

## Stop Conditions
- Implementation requires changing canonical `spec.md` or `contracts.md`.
- A public rollback/transaction command becomes necessary to satisfy Scenario B.
- Existing database files cannot remain openable without a spec change.
- Verifier rejects and a second recovery attempt would be required; contract says to escalate.
