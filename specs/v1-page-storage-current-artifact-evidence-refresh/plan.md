# Implementation Plan

## Objective
Refresh current-artifact evidence for `gate-v1-disk-page-storage` by mapping existing page-storage behavior to the exact requirement IDs in this spec package and by adding focused verification hooks.

## Boundaries
- In scope:
  - `tests/page_storage.rs`
  - `docs/file_format.md`
  - `docs/v1_acceptance.md`
  - `scripts/verify_page_storage_acceptance`
- Conditional scope:
  - `src/storage.rs` only if focused evidence tests expose a real behavior gap.
- Out of scope:
  - SQL behavior, indexes, WAL redesign, benchmark changes, CLI command expansion, network services, `ssot/`, and `policies/`.

## Requirement Mapping
| Requirement ID | Required evidence | Planned artifact |
|---|---|---|
| `REQ-6-store-data-in-a-disk-ad3ffc4e` | 4096-byte page-backed file, header/page inspection, durable on-disk bytes after append | Focused test in `tests/page_storage.rs`; doc row in `docs/v1_acceptance.md`; file-format note in `docs/file_format.md`. |
| `REQ-6-data-must-survive-process-restart-0471a233` | Deterministic reopen test reading records from the same path | Focused reopen test or existing reopen test renamed/annotated for current ID. |
| `FAIL-6-reject-memory-only-dump-at-fd82a296` | Evidence that append is observable in page file before process/drop-only finalization | File-inspection test that reads the page file while `PageStore` remains live after append. |
| `FAIL-6-reject-whole-database-file-rewrite-bebf73bb` | Bounded mutation/page-level evidence rejecting every-write full-file rewrite design | Test that snapshots unaffected pages/headers across append and verifies only expected active-page/header regions change, plus source/run-report evidence for page-level `write_page`. |

## Implementation Steps
1. Reconfirm current HEAD, dirty state, and relevant file presence before editing.
2. Add current-artifact focused tests to `tests/page_storage.rs`.
3. Add or update `scripts/verify_page_storage_acceptance` to run `cargo test --test page_storage` from repo root and fail on missing tools/errors.
4. Update `docs/file_format.md` with a concise evidence mapping note for current artifact requirement IDs and page-level write expectations.
5. Update `docs/v1_acceptance.md` so `gate-v1-disk-page-storage` rows cite the current artifact IDs, focused tests, and `scripts/verify_page_storage_acceptance`.
6. Run focused and baseline verification:
   - `cargo test --test page_storage`
   - `scripts/verify_page_storage_acceptance`
   - `scripts/verify`
7. Record command output summaries and artifact delta in the run report/final execution evidence required by the scheduler.

## Verification Strategy
- Focused verification proves the current artifact evidence mapping:
  - `cargo test --test page_storage`
  - `scripts/verify_page_storage_acceptance`
- Baseline verification proves repo-level compatibility:
  - `scripts/verify`
- Manual review evidence must explicitly mention:
  - current requirement IDs in docs/tests,
  - page-file byte inspection,
  - restart durability,
  - memory-only dump rejection,
  - bounded page-level mutation or source-level page-write evidence.

## Stop Conditions
- Stop and report a blocker if implementation requires changing `spec.md`, `contracts.md`, `ssot/`, or `policies/`.
- Stop and report a blocker if the existing storage behavior contradicts the approved contract in a way that cannot be fixed within the intended touches.
- Escalate if a second recovery attempt becomes necessary.

