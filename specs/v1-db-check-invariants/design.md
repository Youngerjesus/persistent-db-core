# Technical Design: `db check`

## Component Boundary
```text
src/main.rs
  parses CLI args
  renders stable stdout/stderr and exit codes
  calls check::check(path)

src/check.rs
  orchestrates read-only invariant validation
  maps storage/sql/WAL conditions to labeled CheckError variants

src/storage.rs
  owns page-file and WAL byte-format knowledge
  keeps existing PageStore mutating open/replay path for db exec
  exposes read-only check helpers for existing files

src/sql.rs
  owns SQL logical record decoding and table/index reconstruction semantics
  exposes narrow checker validation summary

src/index.rs
  remains the in-memory primary-index implementation
```

Dependencies remain one-way from CLI/check orchestration into storage/sql/index. `storage.rs` must not import `check.rs` or `sql.rs`.

## Proposed API Shape
Names may be adjusted during implementation to match local style, but the behavioral boundary should remain:

```rust
pub mod check;

pub fn check(path: impl AsRef<Path>) -> Result<(), CheckError>;

pub enum CheckError {
    OpenRead { path: PathBuf, operation: &'static str },
    StorageReadability(StorageError),
    CatalogRecordInvariant,
    PrimaryIndex(String),
    WalReplayConsistency(String),
}
```

Storage helper candidates:

```rust
pub fn read_existing_records_without_wal_replay(path: &Path) -> Result<Vec<Vec<u8>>, StorageError>;
pub fn durable_record_count(path: &Path) -> Result<u64, StorageError>;
pub fn validate_wal_sidecar_for_check(path: &Path, durable_record_count: u64) -> Result<(), WalCheckError>;
```

SQL helper candidates:

```rust
pub struct CheckSqlSummary {
    // enough data for check module or produced after internal primary index validation
}

pub enum SqlCheckError {
    CatalogRecordInvariant,
    PrimaryIndexConsistency,
}

pub fn validate_records_for_check(records: Vec<Vec<u8>>) -> Result<CheckSqlSummary, SqlCheckError>;
```

## Validation Flow
1. `check::check(path)` verifies the path refers to an existing readable file-like object. Missing path and directory path become user-facing open/read errors; no empty DB is created.
2. Storage validation reads the V1 page file:
   - file header and page sizes
   - declared page count equals actual page count
   - every data page has valid magic/version/header size
   - `used` and record stream lengths match record count exactly
   - payload bytes are returned in durable append order
3. SQL logical validation decodes the returned records:
   - SQL prefix and logical record tag are recognized
   - catalog identifiers are valid
   - table and column declarations are coherent
   - row references point to existing tables
   - row values match column count and type
4. Primary index validation rebuilds the in-memory primary index from durable rows:
   - one row position per unique primary key
   - duplicate primary keys fail under the `primary index` label
   - lookup/ordered-position map covers exactly the durable row set for each primary-key table
5. WAL validation reads `<database-path>.wal` if present:
   - absent sidecar passes
   - incomplete trailing frame is ignored for check consistency, matching documented replay behavior
   - complete committed page-append frame with `record count before > durable_record_count` fails as `wal replay consistency`
   - complete frame format errors remain check failures under the WAL/storage invariant label, without mutation

## Fixture Design
- Valid fixture: create DB through `db exec`, then run `db check`.
- Storage fixture: create DB, then corrupt first record length or truncate bytes so storage read fails deterministically.
- Catalog/record fixture: create SQL page file containing an invalid SQL logical record such as a row for a missing table or catalog with invalid schema bytes.
- Primary-index fixture: create a primary-key table and duplicate a durable row key through direct fixture bytes so rebuild fails with `primary index`.
- WAL fixture: create DB through `db exec`; write `<path>.wal` with a complete committed `0x01` page-append frame whose `record count before` is greater than the current durable record count. The test must state this ahead-of-store condition in code/comments and assert `wal replay consistency` in stderr.
- Directory-path fixture: pass the temp directory itself to `db check`; do not rely on permission bits or skipped platform cases.

## Documentation Design
- `docs/cli_contract.md` must move `check <path>` into supported commands and document exact success output plus failure prefix.
- The reserved future command list must no longer include `check <path>`.
- `docs/file_format.md` should retain the WAL sidecar table as the source for the WAL fixture and may add a short `db check` compatibility note.

## Verification Evidence
The implementation phase must attach:
- `cargo test --test db_check`
- `scripts/verify`
- changed files summary
- note that visual/UX evidence is not applicable because canonical spec/contract reject it.
