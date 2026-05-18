# Tasks: v1-secondary-index-range-scan

## Execution Rules
- Follow `spec.md` and `contracts.md` as frozen inputs.
- Do not edit `ssot/` or `policies/`.
- Keep production changes scoped to `src/`, focused integration tests, and durable docs needed by the contract.
- Add tests before or with behavior changes; prefer deterministic temp database fixtures.
- Do not add third-party crates.
- This is a non-visual CLI/database task; do not use DOM, screenshot, or UX review evidence as acceptance evidence.

## Task List

### T1. Add secondary index primitive and path-use evidence point
Status: ready

Files:
- `src/index.rs`
- `tests/secondary_index.rs`

Details:
- Implement `SecondaryIndex` with deterministic equality and inclusive range APIs.
- Store entries by `(secondary_key, tie_break)` using `BTreeMap`.
- Add duplicate-entry error for corrupt durable contents.
- Add an implementation-visible path marker or helper that tests can assert for secondary equality and secondary range planning.

Subtasks:
- T1.1 Add primitive tests for equality ordering, inclusive range ordering, empty results, duplicate entry rejection, and low-greater-than-high empty range.
- T1.2 Implement minimal `SecondaryIndex` API.
- T1.3 Add path-use evidence without exposing a broad user-facing debug surface.

Acceptance evidence:
- `cargo test --test secondary_index -- --nocapture` includes primitive/path-use tests.

### T2. Extend SQL logical records for secondary indexes
Status: ready

Files:
- `src/sql.rs`
- `tests/secondary_index.rs`
- `docs/file_format.md`
- `docs/sql_subset.md`

Details:
- Add final-layout `X` secondary index metadata, `E` backfill index entry, and `I` atomic indexed-row records.
- Include `u64 build_id` in `X` and `E`; embedded `I` entries carry `index_build_id` matching the committed `X`.
- Use fixed little-endian binary encoding for `build_id`, `indexed_key`, `tie_break`, and `row_position`.
- Preserve UTF-8 spelling for names while comparing names ASCII case-insensitively.
- Preserve decoding of existing catalog/row-only databases.
- Reconstruct metadata-backed secondary indexes on open.
- Validate secondary index contents against durable rows.
- Scope `E` records to committed `X` records by `(case-insensitive index_name, build_id)`.
- Ignore orphan `E` records without a committed `X` in `db exec` and `db check`.
- Decode `I` as one row plus a complete embedded entry set for every committed secondary index on that row's table.

Subtasks:
- T2.1 Add fixture helpers for old no-index catalog/row records.
- T2.2 Add fixture helpers for exact `X`, `E`, and `I` byte layouts using fixed little-endian fields.
- T2.3 Add decode/check tests for valid metadata and entries.
- T2.4 Add corrupt fixture tests for missing entry, wrong key, wrong tie-break, invalid row position, duplicate entry, missing table/column metadata.
- T2.5 Add orphan `E` fixture test showing uncommitted entries are ignored by `db exec` and do not fail `db check`.
- T2.6 Add malformed `I` fixture tests for missing, extra, wrong-index, wrong-key, wrong-tie-break, and wrong-row-position embedded entries.
- T2.7 Implement encoding/decoding and invariant validation.

Acceptance evidence:
- Focused tests cover old no-index reopen, durable metadata/content validation, and deterministic `db check` secondary invariant failure.

### T3. Implement `CREATE INDEX`
Status: ready

Files:
- `src/sql.rs`
- `tests/secondary_index.rs`
- `tests/sql_exec.rs`

Details:
- Parse `CREATE INDEX <name> ON <table>(<integer_column>);`.
- Validate missing table, missing column, unsupported type, and duplicate index.
- Backfill existing rows into durable index entries.
- Persist index metadata.
- Use `build_id = current durable SQL logical record count` before appending backfill records.
- Append all `E` records first, then the final `X` commit record.
- Keep successful stdout/stderr empty and exit code `0`.

Subtasks:
- T3.1 Add exact stderr tests for the four required CREATE INDEX semantic errors.
- T3.2 Add success test that creates the required users fixture and asserts empty stdout/stderr.
- T3.3 Implement parser and semantic validation.
- T3.4 Implement backfill append ordering, final `X` commit, and runtime registration.
- T3.5 Add interrupted-backfill fixture where only `E` records exist; verify reopen treats the index as uncommitted.
- T3.6 Add retry-after-interrupted-backfill test with the same index name; verify the retry commits with a new `build_id` and queries use only the retried committed entries.

Acceptance evidence:
- Required exact error strings and success behavior are asserted by black-box tests.

### T4. Implement indexed equality and inclusive range scan
Status: ready

Files:
- `src/sql.rs`
- `tests/secondary_index.rs`
- `tests/sql_exec.rs`

Details:
- Extend `SELECT * FROM ... WHERE` to route indexed INT equality through secondary indexes.
- Add `BETWEEN <low> AND <high>` inclusive range support for indexed INT columns.
- Preserve primary-key exact lookup behavior.
- Do not use full table scan for accepted secondary indexed predicates.
- Before `CREATE INDEX`, a non-primary-key equality/range predicate on an otherwise valid table/column remains unsupported SQL with exit code `2` and empty stdout.

Subtasks:
- T4.1 Add required equality example with stdout `id|age|name\n2|20|bea\n3|20|cal\n`.
- T4.2 Add required range example with stdout `id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n`.
- T4.3 Add no-primary-key duplicate-secondary-key tie-break test using insertion order.
- T4.4 Add primary-key duplicate-secondary-key tie-break test using primary-key order.
- T4.5 Add path-use assertions for equality and range.
- T4.6 Add regression test for `SELECT * FROM users WHERE age = 20;` before `CREATE INDEX`; assert unsupported SQL stderr for the exact statement.
- T4.7 Implement parser and execution routing.

Acceptance evidence:
- `tests/secondary_index.rs` proves examples, ordering, tie-breaks, and secondary index path use.

### T5. Maintain secondary indexes on post-index inserts and reopen
Status: ready

Files:
- `src/sql.rs`
- `tests/secondary_index.rs`

Details:
- When inserting into a table with no secondary indexes, keep the existing `R` row-record append behavior.
- When inserting into a table with secondary indexes, append exactly one atomic `I` indexed-row record containing the row and all matching index entries.
- Do not implement post-index insert as `R` plus standalone `E` records.
- Reopen must reconstruct rows and secondary indexes deterministically.
- Existing no-index database reopen must continue to work.
- Failed post-index insert append must leave runtime state unchanged and no partial SQL logical record durable.

Subtasks:
- T5.1 Add compatibility scenario: create old/no-index table and rows, reopen, then create index.
- T5.2 Add backfill scenario over existing rows.
- T5.3 Add post-index insert scenario where the new row appears in equality and range output.
- T5.4 Add process reopen scenario where equality/range output is unchanged.
- T5.5 Add post-index insert fixture/reopen test proving one `I` record contributes both row and index entries.
- T5.6 Add failed/interrupted post-index insert retry scenario: either no `I` exists and retry inserts, or a whole recovered `I` exists and retry follows normal table constraints/output.
- T5.7 Add multi-index post-index insert test proving one `I` contains entries for all committed indexes and no partial per-index state exists.
- T5.8 Implement insert maintenance and reopen reconstruction.

Acceptance evidence:
- Focused persisted compatibility tests cover all contract-required reopen/backfill/post-index cases.

### T6. Add `db check` secondary index invariants
Status: ready

Files:
- `src/sql.rs`
- `src/check.rs`
- `tests/secondary_index.rs`
- `docs/file_format.md`
- `docs/cli_contract.md`

Details:
- Validate secondary metadata and entry contents against durable rows.
- Preserve successful `ok: db check passed\n` stdout.
- Report deterministic invariant failure for secondary index mismatch.
- Ignore orphan `E` records that do not have a matching committed `X(build_id, index_name)` record.
- Validate `I` embedded entries atomically with their row.

Subtasks:
- T6.1 Add positive `db check` test for a valid secondary-indexed database.
- T6.2 Add corrupted committed-index fixture test that produces exact `error: db check failed: secondary index\n`.
- T6.3 Add orphan-entry check test that still passes, proving interrupted uncommitted builds are ignored.
- T6.4 Add corrupted `I` fixture tests for row-with-missing-entry and multi-index partial-entry cases; both must fail as `secondary index`.
- T6.5 Implement validation and label mapping.
- T6.6 Document the check behavior.

Acceptance evidence:
- Black-box `db check` success and failure tests plus docs.

### T7. Update durable docs
Status: ready

Files:
- `docs/cli_contract.md`
- `docs/file_format.md`
- `docs/sql_subset.md`

Details:
- Document `CREATE INDEX`, indexed equality, inclusive `BETWEEN`, success and error contracts, ordering/tie-break rules.
- Document secondary index metadata/storage encoding, old no-index compatibility, backfill behavior, post-index insert maintenance, and `db check` validation.
- Document the final `build_id` state machine: `E` records first, final `X` commit, orphan `E` ignored, retry allowed with a new `build_id`.
- Document post-index insert atomicity: no-index tables use `R`; indexed tables use one `I` record containing row plus all embedded entries; `R + standalone E` is not a valid post-index insert encoding.
- Document exact unsupported behavior for non-primary-key indexed predicates before `CREATE INDEX`.
- Keep docs aligned with exact test strings and final encoding.

Subtasks:
- T7.1 Update CLI contract after tests lock exact behavior.
- T7.2 Update file format after encoding is final.
- T7.3 Update SQL subset grammar/logical-record reference.

Acceptance evidence:
- Manual final report mapping and `scripts/verify`.

### T8. Verification and final implementation report
Status: ready

Files:
- implementation phase report path, such as `final_review.md` or scheduler final report

Details:
- Run:
  - `scripts/verify`
  - `cargo test --test secondary_index -- --nocapture`
- Record stdout/stderr summaries and exit codes.
- Map `REQ-7-create-index-must-create-disk-3b71a7dc` to CLI examples, path-use evidence, persisted compatibility evidence, and `db check` evidence.
- State explicitly that sibling gate requirements were not closed by inference.

Subtasks:
- T8.1 Capture command evidence.
- T8.2 Repair first-pass failures inside the implementation phase when feasible.
- T8.3 Escalate immediately if a second recovery attempt is needed.

Acceptance evidence:
- Required commands pass and final report contains acceptance mapping.

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
- The highest-risk item is durable secondary index consistency and crash/interruption behavior during multi-record backfill.
- Implement invariant tests before wiring query execution, so disk-backed mismatch cannot be accidentally reduced to a rebuild-only implementation.
