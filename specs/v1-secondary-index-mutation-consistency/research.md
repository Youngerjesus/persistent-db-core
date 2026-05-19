# Research: v1-secondary-index-mutation-consistency

## Context
The current repo already has:
- durable SQL logical records over the V1 page/WAL layer;
- `CREATE INDEX` for secondary `INT` columns;
- persisted secondary-index metadata/content records `X`, `E`, and post-index atomic insert record `I`;
- equality and inclusive range query paths over `SecondaryIndex`;
- restart/WAL replay and `db check` validation for committed secondary-index contents.

The remaining gate slice is mutation maintenance after `UPDATE` and `DELETE`: table rows, primary index behavior, secondary equality/range results, reopen/WAL replay, and `db check` invariants must stay mutually consistent.

## Decisions

### R1. Add append-only mutation records above the existing page/WAL format
Decision: implement mutation durability with new SQL logical record kinds at the SQL layer, while preserving the lower-level page file and WAL framing.

Rationale:
- The existing page/WAL layer already commits one opaque SQL payload per appended record.
- The contract accepts either no storage format change or an explicit durable-record format update. Because current SQL docs still list `UPDATE` and `DELETE` as unsupported, adding these commands requires documenting the SQL logical-record evolution.
- Rebuilding mutation state from append-only SQL records is consistent with existing catalog, row, index metadata, and indexed-row reconstruction.

Planned shape:
- `U` update record: table name, primary-key identity for the target row, full replacement row values, and complete secondary-index mutation delta.
- `D` delete record: table name, primary-key identity for the target row, and complete secondary-index removal delta.

The exact byte layout should be locked in implementation before durable docs are finalized. No lower-level `PDBV1` page header, data page, record framing, or WAL frame change is planned.

### R2. Restrict V1 mutation support to primary-key targeted single-row mutations
Decision: parse and execute only the contract-required form:
- `UPDATE users SET age = 30 WHERE id = 2;`
- `DELETE FROM users WHERE id = 3;`

Generalized predicates, multi-row updates/deletes, text predicates, non-primary-key WHERE mutation, arithmetic expressions, partial column-list update breadth, and transaction syntax remain out of scope.

Rationale:
- The acceptance fixture uses an `INT PRIMARY KEY`; primary-key targeting gives deterministic one-row identity without introducing table-scan mutation semantics.
- It avoids defining broader SQL behavior in a plan phase that is supposed to preserve the approved scope.
- It keeps primary and secondary index maintenance testable with black-box CLI output.

### R3. Represent current row visibility during replay
Decision: reconstruction should keep stable durable row positions and row visibility state rather than physically removing old rows from the historical vector.

Rationale:
- Existing secondary entries point at durable row positions.
- `db check` negative fixtures need stale entries, dangling/deleted pointers, and missing visible-row entries to be representable.
- A visibility-aware model lets a deleted row position remain in history while becoming invisible to table scans, primary-key lookup, and secondary-index lookup.

Implementation implication:
- Runtime row storage may need a `RowState` or equivalent wrapper containing values plus `visible/deleted` state.
- Primary and secondary indexes should contain only visible rows.
- Table scan ordering for primary-key tables remains primary key ascending over visible rows only.

### R4. Mutation records must include enough index delta to make `db check` independent of rebuilding-only behavior
Decision: durable `U` and `D` records should carry committed secondary-index removal/addition information, not just the new table values.

Rationale:
- If secondary indexes were always rebuilt from the final row set, stale/missing/dangling secondary entries would not be representable.
- The contract specifically requires `db check` to catch stale secondary entries, dangling row pointers, and missing indexed visible rows.
- The previous secondary-index milestone already made committed secondary-index contents checkable; mutation records should extend that model instead of bypassing it.

For an indexed-column update:
- remove old entry `(old_key, old_tie_break, row_position)`;
- add new entry `(new_key, new_tie_break, same row_position)`.

For delete:
- remove entries for every committed secondary index on the target table;
- remove primary-key lookup visibility for the target row.

### R5. Statement atomicity should remain one SQL logical append per mutation
Decision: each supported `UPDATE` or `DELETE` should append one complete SQL logical record before mutating runtime state.

Rationale:
- Current post-index insert uses one `I` record to avoid partial row/index states.
- A single mutation record gives WAL replay a clear all-or-nothing statement unit.
- If encoding or append fails, runtime state remains unchanged and no partial SQL logical record is durable.

### R6. Tests should prove process boundaries explicitly
Decision: focused tests should run setup, update, delete, each query, and `db check` as separate `db` process invocations.

Rationale:
- The contract requires restart/reopen evidence, not only in-process execution.
- The existing integration test helpers already run the compiled `db` binary and are suitable for this evidence.

### R7. Negative fixtures should live in `tests/secondary_index.rs`
Decision: keep the stale, dangling, and missing-entry deterministic fixture builders in `tests/secondary_index.rs` unless implementation makes `tests/db_check.rs` clearly closer.

Rationale:
- Existing secondary-index fixture helpers already encode catalog, row, `E`, `X`, `I`, and WAL frame data there.
- Keeping mutation corruption fixtures near the secondary-index encoding helpers reduces duplicated byte-construction code.
- If the negative coverage is moved to `tests/db_check.rs`, the implementation phase must also run `cargo test --test db_check -- --nocapture`.

## Compatibility Findings
- The lower-level page/WAL format can remain unchanged.
- Existing row-only and existing secondary-index databases using `C`, `R`, `E`, `X`, and `I` records must reopen.
- New mutation records require durable docs updates in `docs/file_format.md`, `docs/sql_subset.md`, and `docs/cli_contract.md`.
- Final implementation evidence must state that the lower-level page/WAL format did not change, and must document SQL logical-record compatibility for existing secondary-index files.

## Risks
- The current public docs list `UPDATE` and `DELETE` as unsupported; implementation must update docs only when behavior is implemented and tests lock exact output.
- Stale entry and missing visible-row fixtures are easy to accidentally make unrepresentable if mutation implementation only rebuilds indexes from rows.
- Delete semantics must not turn row-position references into shifted indexes. Stable row positions are required.
- Primary-key lookup, full table scan, secondary equality, and secondary range must all agree after mutation and after reopen.

