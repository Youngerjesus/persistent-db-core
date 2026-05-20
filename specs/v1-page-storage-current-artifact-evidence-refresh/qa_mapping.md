# QA Mapping: v1-page-storage-current-artifact-evidence-refresh

Status: `qa_prep_red_contract_ready`

## Scope

This QA mapping prepares the current-artifact evidence refresh for `gate-v1-disk-page-storage`. It intentionally does not implement storage behavior, durable documentation updates, or the focused verification script. The red scaffold in `tests/page_storage.rs` marks the exact current-artifact evidence that the implementation phase must replace with concrete assertions.

## Provenance Contract

Evidence-heavy classification: `yes`. Acceptance depends on product evidence identity and regenerated verifier evidence that connects current artifact requirement IDs to focused tests, docs, and command output.

Evidence root: `target/page_storage_acceptance/` for any implementation-generated focused evidence, plus scheduler run report summaries under the active task run directory.

Required artifact list:
- `tests/page_storage.rs` with concrete current-artifact evidence tests for all four requirement IDs.
- `docs/file_format.md` with current-artifact traceability for page size, page header/data page inspection, restart durability, memory-only rejection, and bounded page-level append evidence.
- `docs/v1_acceptance.md` with `gate-v1-disk-page-storage` rows or notes mapping the four current requirement IDs to tests/docs/scripts.
- `scripts/verify_page_storage_acceptance` running `cargo test --test page_storage` from repo root.
- Command output summaries for `cargo test --test page_storage`, `scripts/verify_page_storage_acceptance`, and `scripts/verify`.

Scenario ids / evidence ids:
- `PS-REQ-6-LAYOUT`: `REQ-6-store-data-in-a-disk-ad3ffc4e`.
- `PS-REQ-6-RESTART`: `REQ-6-data-must-survive-process-restart-0471a233`.
- `PS-FAIL-6-MEMORY-DUMP`: `FAIL-6-reject-memory-only-dump-at-fd82a296`.
- `PS-FAIL-6-FULL-REWRITE`: `FAIL-6-reject-whole-database-file-rewrite-bebf73bb`.

Product evidence identity source or invocation mechanism: the product-facing requirement IDs above, `gate-v1-disk-page-storage`, and the verifier invocation `scripts/verify_page_storage_acceptance`. Do not use scheduler/control-plane run ids as exact product acceptance values in tests, generated product evidence, or Task-Scoped Green.

Clean generation rule: canonical current evidence for each fresh repair or verification pass must be deleted, replaced, or regenerated from the current verifier/product invocation. Historical artifacts may remain only as audit evidence and must not be reused as current proof.

No artifact reuse rule: prior SUCCESS evidence from `v1-page-storage-record-format` can guide implementation, but current proof must come from the current worktree, current tests/docs/scripts, and current command output.

Writer/validator separation expectation: implementation writes tests/docs/scripts and run summaries; the later verifier/reviewer independently runs the preferred commands and checks this mapping against the artifacts.

Redaction target list: no secrets are expected. If command logs include absolute local temp paths or machine-specific scheduler paths, keep them out of durable product docs and summarize them only in run reports.

## Scenario Expansion Lens

Baseline scenarios:
- New page file is created with a fixed 4096-byte header page and deterministic first data page layout after append.
- Reopening the same database path preserves record bytes and order.
- Appended record bytes are inspectable from the page file while `PageStore` remains live.
- An append that fits the active page mutates only expected active-page regions while stable header/page bytes and page count stay unchanged.

Invalid input / corrupt state:
- Truncated files, truncated pages, invalid file/data page magic, unsupported version, oversized records, and corrupt record lengths remain covered by existing `tests/page_storage.rs` negative tests.
- Current-artifact tests should not weaken these existing deterministic errors.

Empty or partial state:
- Layout evidence should include the transition from empty/new file to header plus first data page after append.
- Restart evidence should use deterministic temp paths and lexical drop/reopen boundaries.

Duplicate / already done:
- Reopening after records already exist must not duplicate records.
- WAL sidecar replay, if present, must stay idempotent and not change the page-file evidence expectations.

Dependency failure / timeout:
- Preferred commands must fail on cargo/test/script errors and missing executable script; no silent skip is allowed.

Permission / trust boundary:
- Tests must use temp-path isolated files only and must not depend on network services, external daemons, or shared machine state.

Retry / re-entry:
- A later repair pass must regenerate current evidence and replace red scaffolds with concrete tests rather than marking them ignored.

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | `prepared_observed` | Git/worktree preflight | n/a | `git rev-parse HEAD`; `git status --short` | HEAD recorded and dirty state understood before implementation. | QA prep observed HEAD `02632eed38ac83e4091f23dca8f2419efc076d3f`; `specs/v1-page-storage-current-artifact-evidence-refresh/` is untracked phase input/output. |
| T2 | `prepared_observed` | File existence preflight | n/a | `ls -l tests/page_storage.rs docs/file_format.md docs/v1_acceptance.md scripts/verify src/storage.rs` | All required starting files exist before editing. | Confirmed present in QA prep. |
| T3 | `prepared_observed` | Latest review/report discovery | n/a | `find specs/v1-page-storage-current-artifact-evidence-refresh -maxdepth 2 ...` | Latest review/report files are read if present. | No latest review/report files were present for this feature during QA prep. |
| T4 | `red_scaffolded` | Unit/integration file-byte inspection; current requirement traceability | `tests/page_storage.rs` | `cargo test --test page_storage qa_scaffold_req_6_store_data_in_disk_current_artifact_layout_evidence -- --exact --nocapture` | Concrete test replaces scaffold and asserts 4096-byte alignment, `PDBV1\0\0\0`, version, page count, `PDPG`, first record length, and payload bytes for `REQ-6-store-data-in-a-disk-ad3ffc4e`. | Scenario `PS-REQ-6-LAYOUT`; current scaffold intentionally panics. |
| T5 | `red_scaffolded` | Reopen durability integration test | `tests/page_storage.rs` | `cargo test --test page_storage qa_scaffold_req_6_restart_durability_current_artifact_evidence -- --exact --nocapture` | Concrete test replaces scaffold and proves byte-for-byte record preservation and append order after drop/reopen for `REQ-6-data-must-survive-process-restart-0471a233`. | Scenario `PS-REQ-6-RESTART`; also ensure no replay duplication. |
| T6 | `red_scaffolded` | Live-store file inspection | `tests/page_storage.rs` | `cargo test --test page_storage qa_scaffold_fail_6_reject_memory_only_dump_current_artifact_evidence -- --exact --nocapture` | Concrete test replaces scaffold and reads the page file before `PageStore` drop, proving appended bytes exist before process-end dump for `FAIL-6-reject-memory-only-dump-at-fd82a296`. | Scenario `PS-FAIL-6-MEMORY-DUMP`. |
| T7 | `red_scaffolded` | Bounded mutation/page-level evidence | `tests/page_storage.rs` | `cargo test --test page_storage qa_scaffold_fail_6_reject_whole_file_rewrite_current_artifact_evidence -- --exact --nocapture` | Concrete test replaces scaffold and proves active-page append leaves stable header/page bytes and page count unchanged while expected active-page regions change for `FAIL-6-reject-whole-database-file-rewrite-bebf73bb`. | Scenario `PS-FAIL-6-FULL-REWRITE`; pair with source/run-report evidence for page-level helper use. |
| T8 | `not_started` | Focused verifier script | n/a | `scripts/verify_page_storage_acceptance` | Executable script exists, resolves repo root like `scripts/verify`, and runs `cargo test --test page_storage`. | Red evidence expected now because script is not yet implemented. |
| T9 | `not_started` | Durable docs traceability | n/a | Manual review plus `rg 'REQ-6|FAIL-6|gate-v1-disk-page-storage' docs/file_format.md docs/v1_acceptance.md` | `docs/file_format.md` maps current requirement IDs to page format, page-level append, restart, memory-only rejection, and bounded mutation evidence. | Implementation phase should update docs; QA prep does not alter durable docs. |
| T10 | `not_started` | Acceptance guide traceability | n/a | Manual review plus `rg 'REQ-6|FAIL-6|verify_page_storage_acceptance' docs/v1_acceptance.md` | `docs/v1_acceptance.md` maps `gate-v1-disk-page-storage` to all four current IDs, tests, docs, and focused/baseline commands. | Implementation phase should update docs; QA prep does not alter durable docs. |
| T11 | `red_evidence_required` | Focused cargo test | `tests/page_storage.rs` | `cargo test --test page_storage` | All page-storage tests pass after scaffolds are replaced with concrete assertions. | Red evidence captured in QA prep: fails on the four scaffold panic tests. |
| T12 | `red_evidence_required` | Focused verifier script | `scripts/verify_page_storage_acceptance` | `scripts/verify_page_storage_acceptance` | Script exists and passes from repo root. | Red evidence captured in QA prep: command currently fails because script is missing. |
| T13 | `red_evidence_required` | Baseline repo verification | `scripts/verify` | `scripts/verify` | Baseline verification passes after current-artifact tests/docs/script are implemented. | Expected red while scaffold panic tests remain. |
| T14 | `not_started` | Run report evidence | run result/report | Review run report and terminal PM result | Artifact delta, command summaries, and requirement-by-requirement evidence are recorded with no scheduler ids copied into product acceptance values. | Implementation phase owns final closure report; QA prep result records red contract readiness only. |

## Testing-Review Lens

- All Task IDs T1 through T14 have mapping rows.
- Preferred commands are concrete and runnable, except T8/T12 intentionally identify the missing script as red evidence.
- Task-Scoped Green entries name exact behavior and requirement IDs.
- Negative and boundary coverage is represented by existing corruption/oversize tests plus new scaffold expectations for partial state, retry/reopen, live-store inspection, and bounded mutation.
- Scaffold tests are deterministic, isolated to `tests/page_storage.rs`, and intentionally fail until implementation replaces `panic!` bodies with assertions.

## Red Evidence

QA prep red commands:
- `cargo test --test page_storage` failed as expected with 10 passing tests and 4 failing `qa_scaffold_*current_artifact*` tests.
- `scripts/verify_page_storage_acceptance` failed as expected with exit code 127 because the focused verifier script does not exist yet.
- `scripts/verify` failed as expected after formatting, clippy, and earlier integration tests reached `tests/page_storage.rs`, where the same 4 scaffold tests failed. It should pass only after implementation replaces the scaffold panic bodies with concrete assertions and adds the focused script/docs.
