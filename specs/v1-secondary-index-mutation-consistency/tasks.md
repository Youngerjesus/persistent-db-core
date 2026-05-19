# Tasks: v1-secondary-index-mutation-consistency

## Execution Rules
- Treat `spec.md` and `contracts.md` as frozen canonical inputs.
- Keep implementation scoped to secondary-index mutation consistency.
- Do not edit `ssot/` or `policies/`.
- Add focused tests before or with production behavior changes.
- Do not add third-party crates.
- This is a CLI/database task; visual, browser, screenshot, and UX design-review artifacts are not acceptance evidence for the approved spec.

## Task List

### T1. Add red contract tests for mutation fixture outputs
Status: ready

Files:
- `tests/secondary_index.rs`

Details:
- Add the exact fixed users fixture from `contracts.md`.
- Run setup, `UPDATE`, each post-update query, `DELETE`, each post-delete query, and `db check` through separate `db` process invocations.
- Assert exact exit codes, empty mutation stdout/stderr, and exact query stdout.

Subtasks:
- T1.1 Add fixture helper for the exact schema and seed rows.
- T1.2 Add post-update old key, new key, range, primary-key lookup, and table scan assertions.
- T1.3 Add post-delete equality, range, primary-key lookup, and table scan assertions.
- T1.4 Add positive `db check` after update/delete.

Acceptance evidence:
- `cargo test --test secondary_index -- --nocapture` includes the fixed fixture test names and exact output assertions.

### T2. Add deterministic `db check` negative fixtures
Status: ready

Files:
- `tests/secondary_index.rs`
- optionally `tests/db_check.rs`

Details:
- Use deterministic fixture builders, not checked-in binary blobs.
- Build independent fixtures for stale secondary entry, dangling/deleted pointer, and missing indexed visible row.
- Each fixture must assert exit `1`, empty stdout, and stderr exactly `error: db check failed: secondary index\n`.

Subtasks:
- T2.1 Build stale entry fixture where visible `id=2` has `age=30` and old key `20` remains in `idx_users_age`.
- T2.2 Build dangling pointer fixture pointing to a deleted or nonexistent row position.
- T2.3 Build missing visible-row fixture omitting the committed entry for visible `id=4 age=30`.
- T2.4 If these tests live in `tests/db_check.rs`, record the extra required focused command.

Acceptance evidence:
- The negative fixture tests fail red before implementation and pass after invariant support is implemented.

### T3. Extend SQL parser for narrow mutation forms
Status: ready

Files:
- `src/sql.rs`
- `tests/sql_exec.rs`
- `tests/secondary_index.rs`

Details:
- Parse `UPDATE <table> SET <column> = <value> WHERE <primary_key> = <int>;`.
- Parse `DELETE FROM <table> WHERE <primary_key> = <int>;`.
- Reject unsupported mutation breadth without introducing full table-scan mutation semantics.
- Preserve existing unsupported/malformed error surfaces where the approved contract does not define exact new errors.

Subtasks:
- T3.1 Add parser tests through black-box CLI behavior.
- T3.2 Add statement variants with raw statement text for unsupported fallback.
- T3.3 Validate table existence, primary-key predicate, column existence, and value type.

Acceptance evidence:
- Focused tests prove the accepted forms succeed and unsupported breadth remains outside scope.

### T4. Implement durable update/delete logical records
Status: ready

Files:
- `src/sql.rs`
- `docs/sql_subset.md`
- `docs/file_format.md`
- `tests/secondary_index.rs`

Details:
- Add final `U` and `D` SQL logical record byte layouts.
- Keep lower-level page/WAL format unchanged.
- Encode one complete mutation per successful statement.
- Decode and replay existing `C`, `R`, `E`, `X`, `I` records unchanged for compatibility.
- Decode/replay `U` and `D` records deterministically.

Subtasks:
- T4.1 Add fixture helper encoders/decoders for `U` and `D` records.
- T4.2 Add compatibility test reopening existing secondary-index database without mutation records.
- T4.3 Add update record replay test over page records.
- T4.4 Add delete record replay test over page records.
- T4.5 Add malformed/wrong-entry mutation fixture tests that fail under `db check`.
- T4.6 Implement encoding, decoding, and replay.

Acceptance evidence:
- Existing secondary-index tests still pass; new mutation replay tests pass.

### T5. Maintain runtime primary and secondary indexes on mutation
Status: ready

Files:
- `src/sql.rs`
- `src/index.rs`
- `tests/secondary_index.rs`

Details:
- Introduce stable visible row state.
- Apply update at the same row position.
- Apply delete by marking row invisible.
- Ensure primary index and secondary indexes contain only visible rows.
- Add exact secondary entry removal support if `SecondaryIndex` lacks it.

Subtasks:
- T5.1 Add `SecondaryIndex` removal/update tests if a new helper is introduced.
- T5.2 Update primary-key table scan to skip deleted rows and preserve primary-key ascending output.
- T5.3 Update primary-key lookup to return header-only for deleted rows.
- T5.4 Update secondary equality/range to exclude deleted rows and stale old keys.
- T5.5 Ensure runtime state changes only after append succeeds.

Acceptance evidence:
- Contract fixture outputs match exactly after update and delete.

### T6. Add restart and WAL replay evidence
Status: ready

Files:
- `tests/secondary_index.rs`

Details:
- Prove setup, update, delete, query, and `db check` happen in separate compiled `db` process invocations.
- Prove retained complete WAL frame replay produces the same query and `db check` result while page file and `<path>.wal` sidecar exist.

Subtasks:
- T6.1 Add process-boundary test with separate invocations for every contract step.
- T6.2 Add WAL-sidecar retained-frame test for mutation record replay.
- T6.3 Assert `db check` exit `0`, stdout `ok: db check passed\n`, empty stderr.

Acceptance evidence:
- Focused command output includes the restart/WAL replay test names.

### T7. Update durable docs
Status: ready

Files:
- `docs/cli_contract.md`
- `docs/sql_subset.md`
- `docs/file_format.md`

Details:
- Document the supported mutation forms and exact successful behavior.
- Remove/update statements that say `UPDATE` and `DELETE` are out of scope for the now-supported forms.
- Document SQL logical `U` and `D` record byte layouts.
- Record compatibility: lower-level page/WAL format unchanged; existing row-only and secondary-index files reopen.

Subtasks:
- T7.1 Update CLI contract examples and non-goals.
- T7.2 Update SQL grammar and logical records.
- T7.3 Update file-format compatibility note and `db check` invariant list.

Acceptance evidence:
- `./scripts/verify` passes with docs aligned to tests.

### T8. Verification and implementation report
Status: ready

Files:
- implementation phase report path, such as scheduler final report/result artifact

Details:
- Run required commands.
- Summarize command exit status and stdout/stderr.
- Map evidence to `REQ-7-insert-update-and-delete-must-997871f9` and `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`.
- State storage compatibility outcome: either `no storage format change` for lower-level page/WAL with documented SQL logical-record extension, or document any intentional format change.

Subtasks:
- T8.1 Run `./scripts/verify`.
- T8.2 Run `cargo test --test secondary_index -- --nocapture`.
- T8.3 If negative fixtures are in `tests/db_check.rs`, run `cargo test --test db_check -- --nocapture`.
- T8.4 Record command summaries, test names, requirement ids, and compatibility note.

Acceptance evidence:
- Required commands pass and final report contains traceability.

## Dependency Order
1. T1
2. T2
3. T3
4. T4
5. T5
6. T6
7. T7
8. T8

## Readiness Notes
- No human decision is required before implementation.
- The highest-risk item is making corruption representable without falling back to rebuild-only secondary indexes.
- Keep the first implementation pass test-driven around the fixed fixture and three negative `db check` fixtures.

