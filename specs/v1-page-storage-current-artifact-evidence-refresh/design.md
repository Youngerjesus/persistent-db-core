# Technical Design

## Design Summary
The implementation should make current-artifact evidence explicit while preserving the existing small storage architecture. The preferred shape is additive: focused tests and traceability docs around the existing `PageStore` page-file behavior.

## Evidence Test Design

### 4096-byte Page Layout And Header Inspection
- Create a temporary database path.
- Open `PageStore` and append deterministic payloads.
- Read the page file bytes directly with `std::fs::read`.
- Assert:
  - file length is a multiple of 4096,
  - header magic is `PDBV1\0\0\0`,
  - header page count matches expected page count,
  - first data page starts at byte offset 4096,
  - data page magic is `PDPG`,
  - record length prefix and payload bytes are present at the documented offset.
- Map this test to `REQ-6-store-data-in-a-disk-ad3ffc4e`.

### Restart Durability
- Append records in one lexical scope.
- Drop the first `PageStore` by leaving the scope.
- Reopen the same path with `PageStore::open`.
- Assert record order and bytes match exactly.
- Map this test to `REQ-6-data-must-survive-process-restart-0471a233`.

### Memory-Only Dump Rejection
- Keep a `PageStore` instance live.
- Append a record.
- Before dropping/reopening the store, read the page file directly.
- Assert the record length and payload are already present in the page file.
- This rejects a design where records exist only in memory and are dumped at the end.
- Map this test to `FAIL-6-reject-memory-only-dump-at-fd82a296`.

### Whole-Database Rewrite Rejection
- Create records spanning at least one stable data page and one active append page, or otherwise snapshot stable header/page bytes before an append that fits in the active page.
- Append one more record.
- Read the file again and assert:
  - file length is unchanged when the record fits the active page,
  - stable page bytes are unchanged,
  - page count is unchanged,
  - only the active page's used offset, record count, and appended record region change.
- Pair this bounded mutation evidence with source/run-report review that append writes through `write_page` rather than whole-file serialization.
- Map this test to `FAIL-6-reject-whole-database-file-rewrite-bebf73bb`.

## Script Design
Add `scripts/verify_page_storage_acceptance` as a repo-root portable Bash script:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo test --test page_storage
```

The script may include short echo labels if desired, but should remain deterministic and fail on any command failure.

## Documentation Design
- `docs/file_format.md` should gain a concise section connecting the documented 4096-byte page file and page-level append behavior to the current artifact IDs.
- `docs/v1_acceptance.md` should update or add `gate-v1-disk-page-storage` rows for all four current IDs and cite both focused and baseline commands.
- Do not claim scheduler closure until command evidence exists in the implementation/final report.

## Compatibility
- No on-disk format change is intended.
- Existing WAL sidecar behavior remains allowed; this task proves the page file remains the durable page-backed storage artifact.
- Existing CLI output must remain stable.

