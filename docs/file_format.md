# V1 Page File Format

## Page Size And Numbering

V1 page files use fixed 4096-byte pages. Page `0` is the file header page. Data pages start at page `1` and continue in append order. The file length must always be an exact multiple of 4096 bytes.

## File Header Page

All multi-byte integer fields are little-endian.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 8 | File magic `PDBV1\0\0\0` |
| 8 | 2 | Format version, currently `1` |
| 10 | 2 | Page size as `u16`, currently `4096` |
| 12 | 4 | Page size as `u32`, currently `4096` |
| 16 | 8 | Total page count, including the file header page |
| 24 | 4072 | Reserved, zero-filled in new files |

## Data Page Layout

Each data page stores opaque byte records in append order.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | Data page magic `PDPG` |
| 4 | 2 | Page format version, currently `1` |
| 6 | 2 | Data page header size, currently `16` |
| 8 | 2 | Used byte offset from the start of the page |
| 10 | 2 | Record count in this page |
| 12 | 4 | Reserved, zero-filled in new pages |
| 16 | variable | Record stream |

## Record Encoding

Records are encoded as `u32 little-endian length` followed by exactly that many payload bytes. Payloads are opaque bytes; empty payloads, UTF-8 text, and arbitrary binary bytes are all valid. A single record must fit in one data page after the 16-byte page header and 4-byte length prefix. Overflow pages are not part of V1.

## SQL Logical Records

The SQL executor does not change the page header, data page header, or opaque
record framing. SQL catalog and row data live above `PageStore` as opaque record
payloads documented in `docs/sql_subset.md`.

SQL payloads are UTF-8 compatible and start with the prefix `PDBSQL1\0`.
The byte after the prefix is the SQL logical record kind: `C` for catalog and
`R` for row. Catalog records include table name and ordered column metadata.
Row records include table name and ordered typed values. Arbitrary records
without the SQL prefix are valid page-storage payloads, but they are not valid
SQL database records and are rejected by `db exec` with the documented invalid
SQL storage record error.

Catalog records may include an optional primary-key extension after the ordered
column metadata: byte tag `P` followed by a little-endian `u16` zero-based
column index. The referenced column must be `INT`. Catalog records without this
extension are valid row-only SQL catalogs and load as tables without a primary
key.

Primary indexes are not persisted as separate page records, sidecar files, or
background metadata. `db exec` rebuilds the in-memory primary index from durable
row records when the database is opened. A primary-key table with duplicate
persisted key values is treated as corrupt SQL logical data and fails with the
existing invalid SQL storage record error. Because no separate index metadata is
stored, missing index metadata is not a V1 failure mode.

## Validation Errors

Opening or reading a file validates the header and every declared data page. Short files return a truncated-file error. Non-page-aligned files or missing declared pages return a truncated-page error. Invalid file or data page magic returns an invalid-magic error. Unsupported format versions return an unsupported-version error. Record lengths that exceed the page used bytes or page capacity return a corrupt-record-length error. Oversized appends return a record-too-large error.

## Compatibility Note

V1 is pre-launch and does not guarantee backward compatibility for existing user data. After this page and record format is introduced, format changes must not be made implicitly: the documentation and deterministic tests must be updated together with any intentional format change. SQL logical-record evolution must preserve the lower-level page framing unless a future task explicitly changes the page format contract. The primary-key catalog extension is optional so existing row-only SQL database files remain readable as non-primary-key tables.

## Write-Ahead Log Sidecar

Each database path may have a retained local write-ahead log sidecar at
`<database-path>.wal`, for example `data.pdb.wal` beside `data.pdb`. Existing
database files without this sidecar remain valid and open normally.

The WAL is an append-only stream of frames. All multi-byte integer fields are
little-endian.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 8 | WAL magic `PDBWAL1\0` |
| 8 | 2 | WAL version, currently `1` |
| 10 | 8 | Frame id |
| 18 | 8 | Durable page-store record count before this frame |
| 26 | 1 | State: `0x01` committed, `0x02` rolled back |
| 27 | 1 | Payload kind: `0x01` page append |
| 28 | 4 | Payload length in bytes |
| 32 | 4 | Additive checksum of the full frame with this checksum field treated as zero |
| 36 | variable | Payload bytes |

For payload kind `0x01`, the payload is exactly one page-store record payload:
the same bytes that would appear after the page record's `u32` length prefix.
`PageStore::append_record` writes a committed WAL frame before appending the
payload to the page file.

On open, replay scans frames in append order. A complete committed page-append
frame is applied only when its `record count before` equals the current durable
record count, which makes retained frames idempotent across repeated opens. A
frame whose `record count before` is lower than the current durable count is
treated as already applied. A frame whose `record count before` is higher than
the current durable count is a deterministic storage corruption error because
the WAL and page file order disagree. Rolled-back frames are skipped.

An incomplete trailing frame, including a short header or a short payload, is
ignored, removed from the sidecar during open, and not exposed as a durable
record. This keeps future WAL appends reachable by replay instead of appending
after bytes that replay must ignore. Complete frames with invalid magic,
unsupported version, checksum mismatch, or unknown non-rollback state are
treated as storage corruption and fail open with the existing storage error
surface. Complete WAL frames are retained after replay; V1 does not checkpoint
or truncate complete frames.
