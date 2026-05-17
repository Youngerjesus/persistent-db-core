# Implementation Plan: `db check` Invariant Validation

## Goal
Add `db check <path>` as a supported CLI command that validates V1 persisted database invariants without broadening V1 scope or changing existing `db exec` behavior.

## Non-Goals
- No repair, mutation, checkpointing, benchmark, background task, network surface, browser/UI evidence, or multi-process behavior.
- No new external crates unless implementation discovers an unavoidable std-only blocker.
- No SSOT or policy changes.

## Affected Contracts
- CLI behavior: supported command list, help text, exit codes, stdout/stderr.
- Persisted-data compatibility: page file, SQL logical records, primary-key rebuildability, WAL sidecar consistency.
- Documented errors: stable `db check` success/failure output.
- Tests: black-box CLI tests and deterministic corruption fixtures.
- Durable docs: `docs/cli_contract.md`; `docs/file_format.md` if needed for checker compatibility note.

## Implementation Strategy
1. Add a checker domain surface:
   - Add `src/check.rs` and export it from `src/lib.rs`.
   - Define `check::check(path) -> Result<(), CheckError>`.
   - Keep CLI rendering and exit codes in `src/main.rs`.
2. Add read-only storage validation helpers:
   - Validate an existing path without creating it.
   - Read page-store records without invoking mutating WAL replay.
   - Count durable page-store records.
   - Parse/check documented WAL sidecar frames read-only for ahead-of-store committed page-append frames.
3. Add SQL/check validation helpers:
   - Reuse existing SQL record decoding and catalog/row validation.
   - Return labeled errors for catalog/record invariant failures.
   - Verify rebuilt primary-key key set and row-position mapping are complete and duplicate-free.
4. Wire CLI:
   - Move `check <path>` from reserved to supported command.
   - Update help text with `db check <path>`.
   - Success: exit `0`, stdout exactly `ok: db check passed\n`, empty stderr.
   - Invariant failure: non-zero exit, empty stdout, stderr prefix `error: db check failed:`.
   - Missing or directory path: non-zero exit, empty stdout, `error:` prefix with stable open/read wording and path context.
5. Add tests:
   - `tests/db_check.rs` for valid DB, storage readability corruption, catalog/record invariant corruption, primary-index rebuild failure, WAL ahead-of-store failure, missing path, and temp-directory path.
   - Update `tests/cli_contract.rs` help/support expectations so `check` is no longer reserved/unsupported.
6. Update durable docs:
   - `docs/cli_contract.md`: supported command, success/failure output, exit-code notes.
   - `docs/file_format.md`: compatibility note for `db check` read-only validation and primary-index rebuild semantics if implementation needs explicit wording.

## Error Mapping Plan
| Scenario | Exit | Stdout | Stderr |
|---|---:|---|---|
| valid database | 0 | `ok: db check passed\n` | empty |
| storage parse/readability failure | non-zero | empty | starts `error: db check failed:` and names storage/readability |
| catalog/record invariant failure | non-zero | empty | starts `error: db check failed:` and names catalog/record invariant |
| primary index consistency failure | non-zero | empty | starts `error: db check failed:` and includes `primary index` |
| WAL replay consistency failure | non-zero | empty | starts `error: db check failed:` and includes `wal replay consistency` |
| missing path | non-zero | empty | starts `error:` and includes open/read meaning plus path |
| directory path | non-zero | empty | starts `error:` and includes open/read meaning |

## Verification Plan
- Focused during implementation: `cargo test --test db_check`
- Regression after full change: `scripts/verify`
- Required final evidence: both commands run in the current task worktree after implementation, plus changed test/doc paths listed in the implementation report.

## Risks And Controls
- Risk: using `PageStore::open` would create missing files or replay/truncate WAL during check.
  - Control: use read-only storage helpers for `db check`.
- Risk: private SQL decoder/index types force duplicate parser logic.
  - Control: expose narrow validation APIs from `sql.rs`, not broad internals.
- Risk: primary-index invariant could imply nonexistent persisted index metadata.
  - Control: document and test rebuildability/key-set consistency only.
- Risk: task metadata asks for visual/UX proof while canonical contract excludes it.
  - Control: final implementation report should state CLI-only evidence supersedes visual evidence request under the frozen contract.
