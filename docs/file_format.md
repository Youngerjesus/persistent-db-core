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
The byte after the prefix is the SQL logical record kind: `C` for catalog, `R`
for row, `E` for secondary-index backfill entry, `X` for committed
secondary-index metadata, and `I` for one atomic post-index row plus its
embedded secondary-index entries. `U` updates one existing row slot and `D`
tombstones one existing row slot. Catalog records include table name and ordered
column metadata. Row records include table name and ordered typed values.
Arbitrary records without the SQL prefix are valid page-storage payloads, but
they are not valid SQL database records and are rejected by `db exec` with the
documented invalid SQL storage record error.

Catalog records may include an optional primary-key extension after the ordered
column metadata: byte tag `P` followed by a little-endian `u16` zero-based
column index. The referenced column must be `INT`. Catalog records without this
extension are valid row-only SQL catalogs and load as tables without a primary
key.

Primary indexes are not persisted as separate page records, sidecar files, or
background metadata. `db exec` rebuilds the in-memory primary index from durable
row records when the database is opened. A primary-key table with duplicate
persisted key values is treated as corrupt SQL logical data and fails with the
documented duplicate-primary-key invalid-storage error:

```text
error: invalid SQL storage record: duplicate primary key for table users: 2
hint: primary key values must be unique in persisted SQL storage.
```

Other corrupt SQL logical-record data, such as unknown record tags, continues
to use the generic unknown-record-tag invalid-storage error. Because no separate
index metadata is stored, missing index metadata is not a V1 failure mode.

Secondary indexes are persisted as append-only SQL logical records above the
same page framing. Existing no-index databases containing only `C` and `R`
records remain compatible: they reopen normally, and a later `CREATE INDEX`
backfills existing rows.

`CREATE INDEX` writes all backfill `E` records first, then writes the final `X`
metadata record as the commit marker. The `build_id` in `E` and `X` is the
durable SQL logical-record count before that `CREATE INDEX` appends anything.
An `E` record without a matching committed `X(build_id, index_name)` is an
orphan interrupted build entry; `db exec` and `db check` ignore it. Retrying the
same index name after an interrupted build writes a fresh build id and fresh
`E` records before the final `X`.

Committed secondary-index metadata record:

```text
PDBSQL1\0
X
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 indexed_column little-endian
u8 tie_break_mode: P for primary-key value, R for row insertion order
```

Backfill secondary-index entry record:

```text
PDBSQL1\0
E
u64 build_id little-endian
u16 index_name_len little-endian
index_name UTF-8 bytes
i64 indexed_key little-endian
i64 tie_break little-endian
u64 row_position little-endian
```

After a table has committed secondary indexes, `INSERT` writes exactly one `I`
record instead of `R` plus standalone `E` records. The single record contains
the row values and exactly one embedded entry for every committed secondary
index on that table.

```text
PDBSQL1\0
I
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 value_count little-endian
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count little-endian
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

`UPDATE` writes exactly one `U` record. The row slot remains stable, which keeps
secondary-index row positions meaningful across mutation replay. The record
stores the target row position, replacement row values, and the complete set of
embedded secondary-index entries expected for the replacement row.

```text
PDBSQL1\0
U
u64 row_position little-endian
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 value_count little-endian
repeat value_count:
  u8 type tag: I for INT, T for TEXT
  u32 value_len little-endian
  value UTF-8 bytes
u16 embedded_entry_count little-endian
repeat embedded_entry_count:
  u64 index_build_id little-endian
  u16 index_name_len little-endian
  index_name UTF-8 bytes
  i64 indexed_key little-endian
  i64 tie_break little-endian
  u64 row_position little-endian
```

`DELETE` writes exactly one `D` record and tombstones an existing row slot.

```text
PDBSQL1\0
D
u16 table_name_len little-endian
table_name UTF-8 bytes
u64 row_position little-endian
```

Index names and table/column names are stored with their input spelling.
Runtime attachment and duplicate-index checks compare names ASCII
case-insensitively.

## Validation Errors

Opening or reading a file validates the header and every declared data page. Short files return a truncated-file error. Non-page-aligned files or missing declared pages return a truncated-page error. Invalid file or data page magic returns an invalid-magic error. Unsupported format versions return an unsupported-version error. Record lengths that exceed the page used bytes or page capacity return a corrupt-record-length error. Oversized appends return a record-too-large error.

## Compatibility Note

V1 is pre-launch and does not guarantee backward compatibility for existing user data. After this page and record format is introduced, format changes must not be made implicitly: the documentation and deterministic tests must be updated together with any intentional format change. SQL logical-record evolution must preserve the lower-level page framing unless a future task explicitly changes the page format contract. The primary-key catalog extension is optional so existing row-only SQL database files remain readable as non-primary-key tables. Secondary-index records are additive, and existing no-index databases remain readable and can be backfilled by a later `CREATE INDEX`. Mutation records are additive SQL logical records above unchanged page framing; existing row-only and existing secondary-index databases reopen unchanged.

## Current Artifact Traceability

The `gate-v1-disk-page-storage` current-artifact evidence is tied to the fixed
4096-byte page format above and to the focused verifier
`scripts/verify_page_storage_acceptance`, which runs
`cargo test --test page_storage`.

| Requirement id | File-format evidence |
| --- | --- |
| `REQ-6-store-data-in-a-disk-ad3ffc4e` | `tests/page_storage.rs::qa_scaffold_req_6_store_data_in_disk_current_artifact_layout_evidence` inspects a real page file for 4096-byte alignment, `PDBV1\0\0\0`, version `1`, page count, `PDPG`, record length, and payload bytes. |
| `REQ-6-data-must-survive-process-restart-0471a233` | `tests/page_storage.rs::qa_scaffold_req_6_restart_durability_current_artifact_evidence` drops and reopens `PageStore` on the same path and verifies byte-for-byte record order without duplicate replay. |
| `FAIL-6-reject-memory-only-dump-at-fd82a296` | `tests/page_storage.rs::qa_scaffold_fail_6_reject_memory_only_dump_current_artifact_evidence` reads appended record bytes from the page file while `PageStore` is still live, before process-end or drop-only finalization could occur. |
| `FAIL-6-reject-whole-database-file-rewrite-bebf73bb` | `tests/page_storage.rs::qa_scaffold_fail_6_reject_whole_file_rewrite_current_artifact_evidence` snapshots the header and active data page, appends within the same data page, and verifies stable header/page regions and page count while only the expected active-page record region changes. The same test uses `PageStore::append_record_with_write_audit_for_test` to assert the page-write path records exactly one data-page write range at offset `4096` with length `4096`; this implementation-level audit is paired with byte inspection and source review of the page-write helpers rather than syscall tracing. |

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

## `db check` Format Validation

`db check <path>` validates the existing page file and optional
`<database-path>.wal` sidecar without creating, replaying, truncating,
checkpointing, or repairing either file. The check covers page-record
readability, SQL catalog/row consistency, primary-key rebuildability from
durable row records, committed secondary-index metadata/content consistency,
and WAL replay consistency for the documented sidecar format.

Because primary indexes are rebuilt in memory from row records and are not
persisted as a separate file or record family, `db check` reports duplicate
durable primary-key values as a `primary index` invariant failure. Missing
primary-index metadata is not a V1 file-format failure mode.

`db check` reports committed secondary-index invariant failures with label
`secondary index`. This includes metadata that references a missing table or
non-`INT` column, duplicate index names, missing committed entries, wrong keys,
wrong tie-breaks, invalid row positions, duplicate `(key, tie_break)` entries,
malformed `I` or `U` records with missing, extra, wrong-index, wrong-key,
wrong-tie-break, or wrong-row-position embedded entries, stale entries left
after update, dangling entries for deleted or nonexistent row slots, and missing
entries for visible indexed rows. Orphan `E` records without matching committed
`X` metadata are ignored.
