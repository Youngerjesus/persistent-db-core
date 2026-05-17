# Tasks: v1-primary-btree-index

## Execution Rules
- Follow spec.md and contracts.md as frozen inputs.
- Do not edit ssot/ or policies/.
- Keep production changes scoped to the intended files unless repo reality requires a narrow adjacent edit.
- Add tests before or with behavior changes; keep deterministic temp file fixtures.
- Do not add third-party crates unless a blocker is documented and approved.

## Task List

### T1. Add primary index primitive
Status: ready

Files:
- src/index.rs
- src/lib.rs
- tests/primary_index.rs

Details:
- Implement PrimaryIndex using std::collections::BTreeMap<i64, usize>.
- Provide duplicate-aware insert, exact lookup, and ascending traversal APIs.
- Export the module from src/lib.rs.

Subtasks:
- T1.1 Add primitive tests for insert/find, missing key, duplicate key, empty traversal, and ordered traversal.
- T1.2 Implement PrimaryIndex minimal API.
- T1.3 Keep error type simple and deterministic; avoid panic for duplicate user data.

Acceptance evidence:
- cargo test --test primary_index covers primitive behavior.

### T2. Extend SQL schema model and catalog compatibility
Status: ready

Files:
- src/sql.rs
- tests/primary_index.rs
- docs/file_format.md
- docs/sql_subset.md

Details:
- Represent one optional INT primary key column per table.
- Parse PRIMARY KEY only on INT column declarations.
- Encode new primary key metadata in catalog records while decoding existing catalog records as no-primary-key.
- Rebuild primary indexes from row records in Database::from_records.
- Treat duplicate persisted primary-key rows as SqlError::InvalidStorageRecord.

Subtasks:
- T2.1 Add tests or fixtures for old catalog records without primary metadata.
- T2.2 Add restart/rebuild test with out-of-order inserted keys.
- T2.3 Add corrupt persisted duplicate primary-key row test.
- T2.4 Implement backward-compatible catalog decoding and new catalog encoding.

Acceptance evidence:
- cargo test --test primary_index demonstrates rebuild, duplicate, missing, empty, and corrupt row handling.

### T3. Add primary-key SQL execution paths
Status: ready

Files:
- src/sql.rs
- tests/sql_exec.rs

Details:
- Extend SELECT parser to accept exact WHERE primary_key_column = signed_int.
- Add Statement representation for exact primary key lookup.
- INSERT must check duplicate primary key before append_record.
- SELECT * on a PK table must use ascending PrimaryIndex traversal.
- SELECT * on a non-PK table must remain insert-order.
- Missing key lookup returns header only.

Subtasks:
- T3.1 Add sql_exec tests whose names contain primary_key for setup, exact lookup, ordered scan, missing key, duplicate key, empty table scan, and restart/reopen.
- T3.2 Implement parser changes for exact primary-key WHERE only.
- T3.3 Implement insert duplicate check and select routing.
- T3.4 Preserve existing unsupported/malformed behavior outside the approved grammar.

Acceptance evidence:
- cargo test --test sql_exec primary_key.
- ./scripts/verify keeps existing non-PK behavior green.

### T4. Update durable docs
Status: ready

Files:
- docs/file_format.md
- docs/sql_subset.md
- docs/cli_contract.md

Details:
- Document primary-key grammar and exact lookup.
- Document output ordering split: PK tables sort by primary key, non-PK tables keep insert order.
- Document duplicate primary key error.
- Document persistence model: no separate index metadata, rebuild from durable row records, existing row-only files compatible.
- Document corrupt SQL row records still fail with invalid SQL storage record.
- State missing index metadata is not a failure mode because no index metadata exists.

Subtasks:
- T4.1 Update file-format SQL logical record notes after the implementation's catalog extension layout is final.
- T4.2 Update SQL subset grammar, output, error contract, logical records, and non-goals.
- T4.3 Update CLI contract SQL execution and non-goals.

Acceptance evidence:
- Manual review in final report plus ./scripts/verify.

### T5. Verification and implementation report
Status: ready

Files:
- implementation final report/result path owned by implementation phase

Details:
- Run required commands:
  - ./scripts/verify
  - cargo test --test primary_index
  - cargo test --test sql_exec primary_key
- Report command output summaries.
- Map each acceptance criterion to tests, docs, command output, or blocker.
- Include query path mapping showing SQL execution uses PrimaryIndex for duplicate checks, exact lookup, and ordered scan.

Subtasks:
- T5.1 Capture baseline command evidence after implementation.
- T5.2 If a command fails, repair once within phase; escalate if a second recovery attempt is needed.
- T5.3 Do not use browser/visual evidence as acceptance evidence for this non-visual task.

Acceptance evidence:
- Final implementation report with command summaries and acceptance mapping.

## Dependency Order
1. T1
2. T2
3. T3
4. T4
5. T5

## Readiness Notes
- No human decision is required before implementation.
- The highest-risk item is catalog compatibility; implement tests before changing decoder behavior.

