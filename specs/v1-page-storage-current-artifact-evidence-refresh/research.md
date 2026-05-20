# Research

## Question
What is the minimum implementation evidence delta needed to refresh `gate-v1-disk-page-storage` against the current artifact contract without broadening the database implementation?

## Findings
- `src/storage.rs` already exposes `PageStore`, fixed `PAGE_SIZE = 4096`, file/data page magic constants, page-count validation, page-level read/write helpers, restart read behavior, and deterministic storage errors.
- `tests/page_storage.rs` already covers append/read, reopen durability, opaque record encoding, truncated file/page, invalid magic, unsupported version, oversized record, and corrupt record length.
- `docs/file_format.md` already documents the 4096-byte file/page format, header page, data page layout, and record encoding.
- `docs/v1_acceptance.md` already has a `gate-v1-disk-page-storage` row, but it uses older generic requirement IDs and does not map the current artifact requirement IDs listed in this package.
- `scripts/verify` is the baseline required verification command and currently runs formatting, clippy, all tests, and `db --help`.
- `scripts/verify_page_storage_acceptance` is not present in the tracked repo, so implementation should add it as candidate-specific evidence if allowed by the execution phase.

## Decisions
| Decision | Rationale |
|---|---|
| Treat this as evidence refresh, not storage redesign. | The approved spec says past SUCCESS evidence exists and the gap is stale current-artifact digest evidence. |
| Prefer black-box/file-inspection tests in `tests/page_storage.rs`. | Contract asks for focused tests or equivalent black-box evidence tied to page layout, restart durability, and FAIL-6 rejection. |
| Add current artifact requirement IDs directly in test names, assertion messages, or nearby comments. | The missing gate items are current artifact IDs; generic legacy rows are insufficient for projection closure. |
| Add a narrow `scripts/verify_page_storage_acceptance`. | The task names this script as an intended touch and it gives the scheduler a focused command besides baseline `scripts/verify`. |
| Update durable docs only for traceability, not behavior semantics. | The user-facing compatibility contract does not need new storage behavior; it needs current artifact evidence mapping and command references. |

## Rejected Options
| Option | Reason |
|---|---|
| Modify `src/storage.rs` preemptively. | No current finding proves implementation behavior is missing; implementation phase should only touch source if the new evidence tests fail for a real gap. |
| Edit `ssot/` or `policies/`. | Contract protects these areas and no escalation is present. |
| Replace existing page-storage tests wholesale. | Existing tests are useful regression coverage; add focused current-artifact evidence instead. |
| Use syscall tracing to prove no whole-file rewrite. | It would add platform/tooling fragility. Bounded file-inspection plus source-level page-write evidence is sufficient for this contract. |

## Open Implementation Risks
- Black-box file bytes cannot prove kernel-level write ranges by themselves. Implementation should pair bounded file-inspection tests with source/run-report evidence that `PageStore::append_record` writes through page-level helpers rather than `fs::write` or whole-file serialization.
- Existing WAL behavior may create sidecar writes during append. The page-storage acceptance script should focus on the page file contract while allowing the WAL sidecar to exist.

