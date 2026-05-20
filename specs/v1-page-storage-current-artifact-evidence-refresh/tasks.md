# Tasks

## Phase 1: Pre-edit Confirmation
- [ ] T1 Confirm current HEAD and dirty state with `git rev-parse HEAD` and `git status --short`.
- [ ] T2 Confirm `tests/page_storage.rs`, `docs/file_format.md`, `docs/v1_acceptance.md`, `scripts/verify`, and `src/storage.rs` still exist.
- [ ] T3 Read the latest review/report files if present before repair or retry work.

## Phase 2: Focused Tests
- [ ] T4 Add a focused page-file layout/header inspection test in `tests/page_storage.rs` mapped to `REQ-6-store-data-in-a-disk-ad3ffc4e`.
  - Assert 4096-byte alignment, file magic, format version, page count, data page magic, record length, and payload bytes.
  - Keep assertions deterministic and temp-path isolated.
- [ ] T5 Add or annotate a deterministic reopen durability test mapped to `REQ-6-data-must-survive-process-restart-0471a233`.
  - Reopen the same path after dropping the first store.
  - Assert byte-for-byte record preservation and append order.
- [ ] T6 Add a file-inspection test mapped to `FAIL-6-reject-memory-only-dump-at-fd82a296`.
  - Read the page file after append while `PageStore` is still live.
  - Assert durable record bytes exist before any end-of-process dump opportunity.
- [ ] T7 Add bounded mutation/page-level evidence mapped to `FAIL-6-reject-whole-database-file-rewrite-bebf73bb`.
  - Snapshot stable page/header bytes before append.
  - Append a record that fits the active page.
  - Assert stable bytes and page count remain unchanged while the expected active-page region changes.

## Phase 3: Focused Verification Script
- [ ] T8 Add `scripts/verify_page_storage_acceptance`.
  - Use repo-root resolution matching `scripts/verify`.
  - Run `cargo test --test page_storage`.
  - Ensure executable bit is set.

## Phase 4: Traceability Docs
- [ ] T9 Update `docs/file_format.md` with current artifact requirement ID traceability for page size, header/page inspection, restart durability, memory-only rejection, and bounded page-level append evidence.
- [ ] T10 Update `docs/v1_acceptance.md` so `gate-v1-disk-page-storage` maps the four current requirement IDs to tests/docs/scripts.

## Phase 5: Verification And Report Evidence
- [ ] T11 Run `cargo test --test page_storage`.
- [ ] T12 Run `scripts/verify_page_storage_acceptance`.
- [ ] T13 Run `scripts/verify`.
- [ ] T14 Record the artifact delta, command summaries, and requirement-by-requirement evidence in the execution run report.

## Notes For Implementer
- Do not edit `spec.md` or `contracts.md`.
- Do not edit `ssot/` or `policies/` without explicit escalation.
- Do not broaden the task into SQL, index, WAL redesign, or CLI command work.
- If tests reveal a real storage behavior gap, keep any `src/storage.rs` change minimal and tied to the failing current-artifact evidence.

