# Technical Design: v1-transaction-wal-recovery

## Current Baseline
The repo currently stores opaque page records in `PageStore` and layers SQL catalog/row logical records above that in `src/sql.rs`. `db exec` opens a path, reads all records, reconstructs the in-memory database, then appends catalog or row records for successful mutations. There is no WAL sidecar or public transaction command.

## Proposed Components

### WAL Path Helper
Derive a sidecar path from the database path as `<db-path>.wal`. The implementation should use one helper consistently in code, tests, and docs.

### WAL Frame
The frame layout is frozen for implementation and deterministic fixtures:

| Offset | Size | Field | Value |
|---|---:|---|---|
| 0 | 8 | Magic | `PDBWAL1\0` |
| 8 | 2 | Version | little-endian `u16`, value `1` |
| 10 | 8 | Frame id | little-endian `u64`, monotonically increasing per WAL file |
| 18 | 8 | Record count before apply | little-endian `u64` count of page-store records before this append |
| 26 | 1 | State | `0x01` committed, `0x02` rollback |
| 27 | 1 | Payload kind | `0x01` page-store record append |
| 28 | 4 | Payload length | little-endian `u32` |
| 32 | 4 | Checksum | little-endian `u32` wrapping byte sum |
| 36 | variable | Payload | page-store record payload bytes |

Checksum calculation: sum bytes 0..31 and all payload bytes with wrapping `u32` arithmetic, treating bytes 32..35 as zero. This is not a cryptographic integrity mechanism; it is a deterministic complete-frame validation check for V1 tests.

Incomplete trailing detection: if replay cannot read the full 36-byte header, the declared payload length, or a matching checksum for the final frame, replay ignores that trailing frame and stops. If a malformed complete frame is followed by more bytes, replay reports deterministic recovery/storage corruption instead of guessing.

### Replay Engine
Replay owns file-order interpretation:
- apply committed complete append payloads exactly once;
- skip rollback frames;
- stop before incomplete trailing bytes and leave the incomplete payload unapplied;
- surface malformed non-trailing frames as deterministic storage/recovery errors.

### SQL Integration
`execute_create_table` and `execute_insert` are the mutation sites. Implementation should avoid changing parsing or output behavior. A helper can replace direct `PageStore::append_record` calls with a WAL-aware append path that records intent, commits the WAL frame, applies the page-store append, and updates in-memory state only after durable append success.

### Idempotence Model
Replay must not duplicate rows on repeated opens. The chosen strategy is page-store record-count checkpointing:

- Before writing a committed append frame, capture the current page-store record count.
- Store that count in the frame's `record_count_before` field.
- During replay, if current page-store record count equals `record_count_before`, append the payload to the page store.
- If current count is greater than `record_count_before`, skip the frame as already applied.
- If current count is less than `record_count_before`, fail with deterministic recovery/storage corruption because WAL and page file order disagree.
- WAL frames are retained after apply for evidence; no truncation, side checkpoint file, or applied-marker page record is required for this task.

`tests/wal_recovery.rs` must include a repeated reopen assertion proving retained WAL frames do not duplicate `1|ada`.

## Scenario Design

### Scenario A: CLI Visible Commit Replay
Use `Command::new(env!("CARGO_BIN_EXE_db"))` as existing CLI tests do. Run create/insert in one process, then select in a new process against the same temp path. Assert exact exit code, stdout, and stderr from the spec.

### Scenario B: Storage Fixture For Rollback/Incomplete
Because V1 has no public rollback SQL or transaction command, construct a deterministic WAL fixture directly and verify it through CLI:

1. Run `db exec <path> "CREATE TABLE users (id INT, name TEXT);"` so page-store record count is `1` and only the catalog is durable.
2. Write `<path>.wal` with frame id `1`, `record_count_before = 1`, state `0x01`, payload kind `0x01`, and payload bytes equal to the SQL row logical record for table `users` values `[(INT, "1"), (TEXT, "ada")]`.
3. Append an incomplete trailing frame for frame id `2`, `record_count_before = 2`, state `0x01`, payload kind `0x01`, payload bytes intended to be the SQL row logical record for `9|ghost`, then truncate the final payload bytes or checksum bytes so replay detects it as incomplete.
4. Run `db exec <path> "SELECT * FROM users;"`.
5. Assert exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n`.
6. Run the same select again and assert the same output to prove retained committed frame id `1` is skipped as already applied.

The fixture helper in `tests/wal_recovery.rs` should write the SQL row logical bytes directly using local test helpers that mirror existing `tests/sql_exec.rs` fixture encoders. The test name or comment must state that the fixture is storage-authored because the public CLI intentionally has no rollback or incomplete-transaction command.

## Documentation Design
Add a WAL compatibility note to `docs/file_format.md` containing:
- WAL filename/location;
- WAL frame layout/framing;
- replay order;
- handling of committed, rollback, and incomplete entries;
- behavior when opening old database files with no WAL.

`docs/cli_contract.md` should remain unchanged unless stdout, stderr, exit codes, or supported commands change. The expected implementation should not change it.

## Failure Handling
- User-facing CLI errors should remain non-panicking and use existing exit mappings.
- Incomplete rollback fixture behavior should be deterministic.
- Required verification failure blocks completion.
