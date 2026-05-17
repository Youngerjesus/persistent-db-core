# Tasks: `db check` Invariant Validation

## Phase 1: Red Tests And Contract Surface

### T1 Add focused `db check` integration tests
- Files: `tests/db_check.rs`
- Details:
  - Add helpers for invoking `db`, temp paths, stdout/stderr decoding, cleanup.
  - Add a valid database test asserting exit `0`, stdout exactly `ok: db check passed\n`, empty stderr.
  - Add storage readability corruption fixture using deterministic page/record corruption.
  - Add catalog/record invariant fixture.
  - Add primary-index consistency fixture with duplicate persisted primary-key data.
  - Add WAL ahead-of-store fixture using a complete committed `0x01` page-append frame at `<database-path>.wal` and `record count before > durable record count`.
  - Add missing path test.
  - Add directory-path test using the temp directory itself as `<path>`.
- Acceptance links: all Invariant Evidence Contract bullets.
- Initial expected state: tests fail because `db check` is unsupported.

### T2 Update CLI contract tests for command surface
- Files: `tests/cli_contract.rs`
- Details:
  - Update required help lines to include `db check <path>` as supported.
  - Remove `check <path>` from reserved future expectations.
  - Keep `open` and `bench` reserved/unsupported.
  - Replace or adjust the reserved command test so it does not assert `check` is unsupported.
- Acceptance links: documented CLI surface and supported command behavior.

## Phase 2: Checker Implementation

### T3 Add checker module and CLI wiring
- Files: `src/check.rs`, `src/lib.rs`, `src/main.rs`
- Details:
  - Export `pub mod check;`.
  - Add `db check <path>` match arm.
  - Print exact success output.
  - Map checker errors to stable stderr and non-zero exit codes.
  - Ensure unsupported/malformed argument handling remains stable for other commands.
- Acceptance links: valid fixture, missing path, directory path, stable stdout/stderr.

### T4 Add read-only storage and WAL validation helpers
- Files: `src/storage.rs`
- Details:
  - Add helpers that validate/read existing page files without creating a file and without replaying/truncating WAL.
  - Reuse existing page validation logic where practical.
  - Expose durable record count for checker/WAL validation.
  - Add read-only WAL sidecar parsing based on documented offsets.
  - Detect committed page-append frames ahead of durable count as `wal replay consistency`.
  - Preserve current `PageStore::open` and `db exec` behavior.
- Acceptance links: storage readability and WAL replay consistency evidence.

### T5 Expose SQL logical validation for checker use
- Files: `src/sql.rs`
- Details:
  - Reuse existing decode/catalog/row validation and primary-index rebuild logic.
  - Add narrow public validation API for checker use.
  - Return distinguishable catalog/record and primary-index consistency failures.
  - Avoid changing SQL execution output or accepted SQL grammar.
- Acceptance links: catalog/record invariant and primary index consistency evidence.

## Phase 3: Durable Docs

### T6 Update CLI contract documentation
- Files: `docs/cli_contract.md`
- Details:
  - Add `db check <path>` to supported commands.
  - Document exact success output.
  - Document failure prefix and exit-code category.
  - Remove `check <path>` from reserved future commands.
  - Note that `db check` does not repair or mutate data.
- Acceptance links: CLI contract documentation.

### T7 Update file-format compatibility note if needed
- Files: `docs/file_format.md`
- Details:
  - Add a concise note that `db check` validates page records, SQL logical record consistency, primary-key rebuildability, and documented WAL sidecar consistency.
  - Clarify that primary indexes remain in-memory and are validated by rebuild/key-set consistency, not by a separate persisted index file.
- Acceptance links: compatibility note and WAL fixture source.

## Phase 4: Verification And Report Evidence

### T8 Run focused verification
- Command: `cargo test --test db_check`
- Details:
  - Capture pass/fail output in implementation report.
  - If failing, repair within implementation phase; escalate only if canonical spec/contract conflict is found.

### T9 Run baseline verification
- Command: `scripts/verify`
- Details:
  - Must run from current task worktree.
  - Must pass `cargo fmt --check`, clippy with `-D warnings`, full tests, and help smoke.

### T10 Final implementation report
- Files: implementation run report chosen by execution phase
- Details:
  - Map every Candidate Acceptance Criteria to evidence.
  - Include command evidence for `scripts/verify` and `cargo test --test db_check`.
  - State that UI/visual/UX evidence is intentionally not provided because the frozen CLI-only contract excludes it.
