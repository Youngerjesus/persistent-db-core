# QA Mapping: V1 Page Storage Record Format

## QA Prep Context
- Task ID: `task-2026-05-16-13-58-47-v1-page-storage-record-format`
- Feature slug: `v1-page-storage-record-format`
- Artifact gate: `gate-v1-disk-page-storage`
- Requirement IDs: `req-v1-page-storage-restart`, `req-v1-record-format-doc`
- Current run ID: `qa_prep_retry_1_resume_20260516_142144_828543_086d4356`
- Task source: `specs/v1-page-storage-record-format/tasks.md`, which declares one canonical task: `task-2026-05-16-13-58-47-v1-page-storage-record-format`.
- Required skill note: `.codex/skills/qa-prep/SKILL.md` was requested for this phase, but no repo-local or session skill by that name is available. This artifact follows the requested workflow order directly: Scenario expansion lens -> QA generation lens -> Testing-review lens.
- Evidence-heavy classification: not evidence-heavy under the supplied definition. Acceptance depends on deterministic Rust tests, command output, and checked-in documentation rather than screenshots, launch evidence bundles, exported reports, evaluator output, redaction proof, or artifact-producing harnesses.

## Scenario Expansion Lens
| Scenario ID | Path | Coverage Pressure | Expected Contract |
| --- | --- | --- | --- |
| `PST-SC-001` | Open a missing database path | Happy path, initial creation | Creates a deterministic fixed-size page file; no user-facing storage CLI is added. |
| `PST-SC-002` | Append and read `b"alpha"`, `b"beta"` | Basic append/read | Read returns the same byte payloads in append order. |
| `PST-SC-003` | Drop/reopen same database path | Restart/re-entry | Reopened store reads the same record sequence and byte values. |
| `PST-SC-004` | Append `b""`, ASCII, and `[0x00, 0xff, 0x10]` | Record encoding boundary | Empty and binary payloads round-trip exactly; append order is preserved. |
| `PST-SC-005` | Inspect bytes after append | Format determinism | File length is page aligned; header/page magic, version, page size, and little-endian record length are stable. |
| `PST-SC-006` | File shorter than one header page | Partial state/truncation | Returns `StorageError::TruncatedFile`; no panic or silent success. |
| `PST-SC-007` | Header declares or implies an incomplete data page | Partial page/truncation | Returns `StorageError::TruncatedPage`; no best-effort read. |
| `PST-SC-008` | Invalid file magic/header | Invalid input/corruption | Returns `StorageError::InvalidMagic` deterministically. |
| `PST-SC-008B` | Invalid data page magic/header | Invalid input/corruption | Returns `StorageError::InvalidMagic` deterministically after a valid header plus corrupt data page exists. |
| `PST-SC-009` | Supported magic with unsupported version field | Version boundary | Returns `StorageError::UnsupportedVersion`; evidence is separate from invalid magic/header. |
| `PST-SC-010` | Single record larger than one data page capacity | Boundary/overflow | `append_record` returns `StorageError::RecordTooLarge`; no overflow pages are introduced. |
| `PST-SC-011` | Stored record length exceeds page used bytes/capacity | Corrupt record length | Returns `StorageError::CorruptRecordLength`; no panic or silent truncation. |
| `PST-SC-012` | Existing CLI help route after storage code lands | Regression/scope guard | `cargo run --bin db -- --help` still succeeds; reserved storage command is not exposed. |
| `PST-SC-013` | Permission or filesystem failure | Dependency failure/trust boundary | Scope-review-only for QA prep. The approved spec does not require this as task-scoped green; implementation should still avoid network service, daemon, remote dependency, or multi-process behavior. |
| `PST-SC-014` | Repeated test and command runs | Retry/re-entry | Tests and smoke commands are deterministic; current-run evidence must be regenerated, not reused from earlier runs. |

## QA Generation Lens
| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `task-2026-05-16-13-58-47-v1-page-storage-record-format` | `qa-prep-retry-ready-red-scaffold` | Integration tests for public Rust storage API; deterministic byte-format assertions; failure-mode assertions by stable `StorageError` variant; documentation review for `docs/file_format.md`; full regression via `cargo test`; CLI preservation smoke via `cargo run --bin db -- --help`; scope review confirming no new storage CLI command and no protected-area edits. | `tests/page_storage.rs`; existing `tests/cli_contract.rs`; expected implementation docs `docs/file_format.md`. | `cargo fmt --check`; `cargo test --test page_storage`; `cargo test`; `cargo run --bin db -- --help`. | Green only when `tests/page_storage.rs` passes all append/read/reopen, encoding, byte-format, and required failure-mode tests for truncated file, truncated page, invalid file magic/header, invalid data page magic/header, unsupported version, page overflow record, and corrupt record length; `cargo test` passes including existing CLI contract tests; `docs/file_format.md` documents page size, page numbering, header or page layout, slot or record length layout, endian, record encoding, validation errors, and compatibility note; `db --help` succeeds without exposing a storage-specific CLI command; implementation changes remain scoped to the approved storage primitive, tests, docs, and minimal crate exposure if required. | Current scaffold is intentionally red because `src/storage.rs` and library exposure do not exist yet. Implementation must add a minimal `src/lib.rs` with `pub mod storage;` before behavioral failures become visible. `PST-SC-013` is scope-review-only and not a task-scoped green requirement. |

## Test Scaffold
- `tests/page_storage.rs` is the red scaffold for `PST-SC-001` through `PST-SC-012`, excluding `PST-SC-013` because dependency failure/trust-boundary is scope-review-only for this approved spec.
- It imports `persistent_db_core::storage::{PageStore, StorageError}` to force a public internal Rust API for downstream V1 work.
- It expects stable error variants: `TruncatedFile`, `TruncatedPage`, `InvalidMagic`, `UnsupportedVersion`, `RecordTooLarge`, and `CorruptRecordLength`.
- It asserts byte-level format anchors for page alignment, file magic `PDBV1\0\0\0`, data page magic `PDPG`, version `1`, and little-endian record length.
- It has separate invalid magic/header tests for file header magic and data page magic/header, keeping invalid-header evidence distinct from unsupported-version evidence.
- It intentionally fails first with `E0433` until implementation adds the allowed minimal `src/lib.rs` library target with `pub mod storage;`.
- It does not add production storage code, docs, a user-facing storage command, network behavior, WAL, SQL, indexes, or recovery logic.

## Provenance Contract
- Evidence root: current run directory `/Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/autopilot/project_manager/tasks/task-2026-05-16-13-58-47-v1-page-storage-record-format/runs/qa_prep_retry_1_resume_20260516_142144_828543_086d4356/`.
- Required artifact list: `result.md`; task source at `specs/v1-page-storage-record-format/tasks.md`; QA mapping at `specs/v1-page-storage-record-format/qa_mapping.md`; QA prep review at `specs/v1-page-storage-record-format/qa_prep_review.md`; red test scaffold at `tests/page_storage.rs`; implementation-phase evidence later must include `cargo test --test page_storage`, `cargo test`, `cargo run --bin db -- --help`, `docs/file_format.md`, changed file list, and acceptance-criterion proof entries in the run report.
- Scenario/evidence IDs: `PST-SC-001` through `PST-SC-014`, including `PST-SC-008B`; `EV-PAGE-STORAGE-RED`; `EV-CARGO-TEST-RED`; `EV-HELP-SMOKE-QA-PREP`.
- Current-run ID source: task metadata `active_run_id=qa_prep_retry_1_resume_20260516_142144_828543_086d4356`.
- Clean generation rule: canonical launch evidence, command evidence, and verification artifacts for a fresh repair or verification pass are deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: implementation and verification phases must not reuse prior run command output, screenshots, exported reports, cached evaluator output, or older database fixture files as proof for this run.
- Writer/validator separation expectation: the implementation worker may write storage code, docs, and tests; verifier must independently rerun the preferred commands and inspect generated artifacts before accepting `gate-v1-disk-page-storage`.
- Redaction target list: no secrets are expected; still redact absolute home-directory-sensitive tokens, environment variable values, credentials, API keys, auth tokens, and any database fixture payloads if future evidence accidentally emits non-test data.

## Testing-Review Lens
- Task ID coverage: `specs/v1-page-storage-record-format/tasks.md` declares one task ID, and that task ID is represented in the QA Generation Lens table and maps to every acceptance criterion.
- Requirement coverage: `req-v1-page-storage-restart` is covered by `PST-SC-001` through `PST-SC-004` plus `EV-PAGE-STORAGE-RED`; `req-v1-record-format-doc` is covered by `PST-SC-005`, the documentation review layer, and implementation-phase `docs/file_format.md` proof.
- Negative/boundary coverage: truncated file, truncated page, invalid file magic/header, invalid data page magic/header, unsupported version, oversized record, corrupt record length, retry/re-entry, and trust-boundary scope controls are represented. Filesystem/dependency failure is explicitly scope-review-only and not task-scoped green.
- Preferred command coverage: all required commands from the approved spec plus `cargo fmt --check` are listed and must be rerun after implementation.
- Current red expectation: `cargo test --test page_storage` and `cargo test` should fail against the current binary-only skeleton because the public `storage` API does not exist. Implementation must first add `src/lib.rs` with `pub mod storage;` and `src/storage.rs`; then the scaffold will expose the behavioral contract failures until storage is complete. `cargo run --bin db -- --help` should remain green as CLI preservation evidence.
