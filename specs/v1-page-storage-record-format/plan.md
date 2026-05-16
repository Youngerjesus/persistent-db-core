# Plan: V1 Page Storage Record Format

Status: ready_for_execution

## Phase Boundary

This plan closes the plan-execution responsibility only. It does not edit production code, tests, runtime configuration, SSOT, policies, or final evidence artifacts.

Requested skill note: `sdd-autopilot` is not available in this session's skill list. The plan follows the repository `AGENTS.md`, the approved `spec.md`, and `contracts.md`.

## Latest Worktree Check

- Repo root: `persistent-db-core_worktree/task-2026-05-16-13-58-47-v1-page-storage-record-format`
- Verified HEAD: `178aa445c286aee9929ed7e0b8a14bd7e3d6b2e0`
- Current dirty state observed for this phase: untracked `specs/v1-page-storage-record-format/` package only.
- Existing code shape: binary-only Rust crate with `src/main.rs`, `tests/cli_contract.rs`, and no existing storage module.
- Protected areas remain out of scope: `ssot/`, `policies/`.

## Implementation Boundary

Implement only the durable page storage primitive needed for this gap:

- Add `src/storage.rs` with a deterministic fixed-size page file abstraction.
- Add `tests/page_storage.rs` with append/read/reopen, record encoding, and failure-mode coverage.
- Add `docs/file_format.md` with the V1 on-disk format and compatibility note.
- Preserve the existing `db --help` CLI contract; do not add a storage-specific user-facing CLI command.

Because the current crate is binary-only, execution should expose `storage` to integration tests by either:

- adding a minimal `src/lib.rs` with `pub mod storage;`, or
- using a test-local module include for `src/storage.rs`.

The preferred path is `src/lib.rs` because the contract calls for a public Rust storage API and it keeps integration tests aligned with downstream internal callers. This is a minimal implementation-enabling touch, not a new product feature.

## Storage API Shape

Keep the public API narrow and deterministic:

- `PageStore::open(path: impl AsRef<Path>) -> Result<PageStore, StorageError>`
- `PageStore::append_record(&mut self, payload: &[u8]) -> Result<(), StorageError>`
- `PageStore::read_records(&mut self) -> Result<Vec<Vec<u8>>, StorageError>`
- `StorageError` variants with stable discriminants or stable messages for tests.

Required deterministic error categories:

- `TruncatedFile`
- `TruncatedPage`
- `InvalidMagic`
- `UnsupportedVersion`
- `RecordTooLarge`
- `CorruptRecordLength`
- plus `Io` wrapping normal filesystem failures.

Tests should assert the variant, not a fragile full display string, unless the implementation chooses stable error codes in messages.

## File Format Plan

Use an explicit, little-endian, fixed-page layout and document every byte range in `docs/file_format.md`.

Recommended V1 constants:

- Page size: `4096` bytes.
- Format version: `1`.
- File header page: page `0`.
- Data pages: page `1..`.
- Opaque record payload length prefix: `u32` little-endian.

Recommended header page layout:

- bytes `0..8`: file magic, for example `PDBV1\0\0\0`.
- bytes `8..10`: format version `u16` little-endian.
- bytes `10..12`: page size `u16` little-endian.
- bytes `12..16`: header size `u32` little-endian.
- bytes `16..24`: page count `u64` little-endian, including page `0`.
- bytes `24..32`: record count `u64` little-endian.
- bytes `32..4096`: reserved zero bytes.

Recommended data page layout:

- bytes `0..4`: page magic, for example `PDPG`.
- bytes `4..6`: format version `u16` little-endian.
- bytes `6..8`: header size `u16` little-endian.
- bytes `8..10`: used bytes `u16` little-endian, including the page header.
- bytes `10..12`: record count `u16` little-endian.
- bytes `12..16`: reserved zero bytes.
- bytes `16..used_bytes`: repeated records, each as `u32` little-endian payload length followed by exact payload bytes.
- bytes `used_bytes..4096`: reserved zero bytes.

Append behavior:

- Create a new database file with a zero-filled header page if the path does not exist.
- Append records to the last data page when `4 + payload.len()` fits.
- Allocate one new zero-filled data page when the last page lacks capacity.
- Reject any single record that cannot fit in one data page after the data page header.
- Do not introduce overflow pages, WAL, indexes, SQL, or transaction semantics in this task.

Read behavior:

- Validate file length and every page boundary before returning records.
- Read data pages in page-number order and records in page-local append order.
- Preserve empty payloads and arbitrary binary bytes exactly.
- Return deterministic corruption errors instead of panicking or silently truncating.

## Test Plan

Create `tests/page_storage.rs` with deterministic temporary directories under `std::env::temp_dir()` using a unique suffix from process id and test name. Clean up best-effort at the end of each test.

Required test cases:

- `append_read_preserves_order_and_bytes`: appends at least `b"alpha"` and `b"beta"` and reads them back in order.
- `reopen_reads_previously_appended_records`: closes/drops store, reopens the same path, and verifies identical record sequence.
- `record_encoding_supports_empty_ascii_and_binary_payloads`: includes `b""`, ASCII payloads, and `[0x00, 0xff, 0x10]`.
- `truncated_file_returns_error`: creates a file shorter than the header page and asserts `TruncatedFile`.
- `truncated_page_returns_error`: creates a valid header that declares or implies an incomplete final page and asserts `TruncatedPage`.
- `invalid_magic_returns_error`: corrupts the file magic or page magic and asserts `InvalidMagic`.
- `unsupported_format_version_returns_error`: sets the version field to an unsupported value and asserts `UnsupportedVersion`; keep this separate from invalid magic.
- `page_overflow_record_returns_error`: tries appending a payload larger than one data page can hold and asserts `RecordTooLarge`.
- `corrupt_record_length_returns_error`: corrupts a stored record length so it exceeds `used_bytes` or page capacity and asserts `CorruptRecordLength`.

The tests should also inspect deterministic bytes where useful, for example file magic, version, page size, and one record length prefix, to make format regressions visible.

## Documentation Plan

Create `docs/file_format.md` with these sections:

- Overview and scope.
- Page size and page numbering.
- Header page layout.
- Data page layout.
- Record encoding.
- Endianness.
- Validation and corruption errors.
- Compatibility note.

The compatibility note must state both:

- V1 is pre-launch, so existing user data backward compatibility is not guaranteed.
- After this spec lands, on-disk format changes must update this document and the corresponding tests; implicit format changes are not allowed.

## CLI Preservation Plan

Do not add storage commands to `src/main.rs`.

Run the existing contract checks after implementation:

- `cargo test`
- `cargo run --bin db -- --help`

If a minimal `src/lib.rs` is added, verify that binary behavior and existing `tests/cli_contract.rs` remain unchanged.

## Verification Evidence Mapping

- Fixed-size page file creation and append/read: `cargo test --test page_storage`, `append_read_preserves_order_and_bytes`, `tests/page_storage.rs`.
- Restart read verification: `cargo test --test page_storage`, `reopen_reads_previously_appended_records`.
- Record encoding: `cargo test --test page_storage`, `record_encoding_supports_empty_ascii_and_binary_payloads`.
- Truncated file/page: dedicated truncated tests asserting `TruncatedFile` and `TruncatedPage`.
- Invalid magic/header: dedicated invalid magic test asserting `InvalidMagic`.
- Unsupported version: dedicated version test asserting `UnsupportedVersion`.
- Page overflow: dedicated oversized append test asserting `RecordTooLarge`.
- Corrupt record length: dedicated length corruption test asserting `CorruptRecordLength`.
- File format documentation: `docs/file_format.md` sections listed above.
- Full regression: `cargo test`.
- CLI contract preservation: `cargo run --bin db -- --help` and existing CLI contract tests inside `cargo test`.

## Visual And UX Evidence

Visual verification is not applicable. This is a Rust CLI storage/file-format task with no DOM route, viewport, screenshot, or UX surface. The proof layers are deterministic test output, command output, `docs/file_format.md`, and the run report evidence mapping.

## Risk Controls

- Treat `docs/file_format.md` and `tests/page_storage.rs` as compatibility gates once introduced.
- Keep all integer encoding little-endian and documented.
- Prefer explicit page and record validation over best-effort reads.
- Avoid adding WAL, recovery, SQL, B-tree, secondary indexes, concurrency, background services, or remote dependencies.
- Escalate rather than expanding scope if a second recovery attempt is required or if implementation needs SSOT/policy changes.

## Execution Checklist

1. Re-check HEAD, dirty state, and relevant file existence before code edits.
2. Add the storage module and minimal library exposure if needed.
3. Add deterministic integration tests first or alongside implementation.
4. Add `docs/file_format.md`.
5. Run `cargo fmt`.
6. Run `cargo test --test page_storage`.
7. Run `cargo test`.
8. Run `cargo run --bin db -- --help`.
9. Update the final run report with changed files, command summaries, and acceptance-criterion evidence links.
