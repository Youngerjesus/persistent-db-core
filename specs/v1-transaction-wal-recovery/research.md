# Research: v1-transaction-wal-recovery

## Decisions

### WAL Scope
Use a minimal sidecar WAL for SQL mutations on the existing `db exec` path. The WAL should prove restart replay semantics for committed mutations and deterministic exclusion of rollback or incomplete entries without introducing public transaction SQL.

Rationale: the approved scope asks for minimum evidence, not a full transaction feature. Existing CLI mutations are statement-level and already deterministic, so the WAL can model each successful catalog or row append as a committed WAL transaction.

### WAL Location
Use a deterministic sidecar path derived from the database path: `<database-file>.wal`.

Rationale: this avoids changing the existing page file header or page framing and keeps existing database files openable. The exact helper should be documented in `docs/file_format.md` once implemented.

### WAL Record Framing
Use std-only binary framing with a fixed 36-byte header followed by payload bytes:

| Offset | Size | Field |
|---|---:|---|
| 0 | 8 | magic `PDBWAL1\0` |
| 8 | 2 | version `1`, little-endian `u16` |
| 10 | 8 | frame id, little-endian `u64` |
| 18 | 8 | page-store record count before apply, little-endian `u64` |
| 26 | 1 | state: `0x01` committed, `0x02` rollback |
| 27 | 1 | payload kind: `0x01` page-store record append |
| 28 | 4 | payload length, little-endian `u32` |
| 32 | 4 | checksum, little-endian `u32`, wrapping byte sum of header bytes 0..31 and payload bytes with checksum bytes treated as zero |
| 36 | variable | payload bytes |

Replay treats a short header, short payload, or checksum mismatch at end-of-file as an incomplete trailing frame and ignores it. A malformed complete non-trailing frame is a deterministic recovery/storage error. The Scenario B fixture will use the same byte layout directly.

Rationale: tests need deterministic fixture construction. A length-delimited frame lets replay stop before incomplete payloads and keeps corruption behavior explicit without adding dependencies.

### Replay Semantics
On open or before SQL execution, replay WAL frames in file order:
- committed complete frames are applied to the durable page store if not already represented;
- rollback/abort frames are ignored;
- incomplete trailing frames are ignored and must not expose their payload as a SQL row.

Rationale: Scenario A requires committed replay after a new process opens the same path. Scenario B requires rollback or incomplete `9|ghost` not to appear.

### Idempotence
Use the `page-store record count before apply` field as the idempotence checkpoint. During replay:
- if current page-store record count equals `record_count_before`, apply the committed append payload;
- if current page-store record count is greater than `record_count_before`, treat the frame as already applied and skip it;
- if current page-store record count is less than `record_count_before`, report deterministic recovery/storage corruption;
- rollback frames are always skipped and do not change the checkpoint;
- incomplete trailing frames are ignored and do not change the checkpoint.

Rationale: replay may run more than once and V1 is single-process. Existing page records already give an ordered durable count, so no extra applied-marker records or checkpoint files are needed. WAL frames may be retained after successful apply; repeated opens remain deterministic because already-applied frames are skipped by record count.

### Public CLI
Do not add public transaction, rollback, or recovery commands.

Rationale: the approved spec explicitly treats public transaction SQL as out of scope. `docs/cli_contract.md` should remain unchanged unless the implementation changes stdout, stderr, exit codes, or command grammar.

## Risks
- Directly appending page records after WAL replay can duplicate committed rows unless idempotence is designed and tested.
- Storage tests may need public helper APIs; prefer black-box CLI tests for Scenario A and minimal public/storage APIs only when necessary for Scenario B fixture construction.
- File-format documentation must be precise enough to cover WAL filename/location, framing, replay order, committed/rollback/incomplete handling, and old database compatibility.
