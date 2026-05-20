# Tasks

## Phase 1: Pre-edit Confirmation
- [ ] T1 Confirm current HEAD and dirty state with `git rev-parse HEAD` and `git status --short`.
- [ ] T2 Confirm `tests/primary_index.rs`, `tests/sql_exec.rs`, `src/index.rs`, `src/sql.rs`, `docs/v1_acceptance.md`, and `scripts/verify` still exist.
- [ ] T3 Read latest review/report files in this feature directory if present before repair or retry work.
- [ ] T4 Confirm no implementation task requires edits to `spec.md`, `contracts.md`, `ssot/`, or `policies/`.

## Phase 2: PrimaryIndex Primitive Evidence
- [ ] T5 Update or confirm `tests/primary_index.rs` covers `PrimaryIndex::insert` with `2 -> 0` and `1 -> 1`.
  - Assert `get(2) == Some(0)`, `get(1) == Some(1)`, and `get(3) == None`.
  - Assert duplicate `insert(2, 99)` returns an error.
  - Assert the duplicate does not overwrite the original `get(2) == Some(0)`.
- [ ] T6 Update or confirm ordered traversal coverage.
  - Insert `30 -> 0`, `-5 -> 1`, and `10 -> 2`.
  - Assert `ordered_positions() == vec![1, 2, 0]`.
  - Assert an empty index returns `Vec::<usize>::new()`.

## Phase 3: Persisted Rebuild Evidence
- [ ] T7 Update or confirm `tests/primary_index.rs` proves same-path persisted SQL rows rebuild the primary index after reopen.
  - Use table `users (id INT PRIMARY KEY, name TEXT)`.
  - Insert `2/'bea'`, `1/'ada'`, and `3/'cal'`.
  - Assert `SELECT * FROM users WHERE id = 2;` returns `id|name\n2|bea\n`.
  - Assert `SELECT * FROM users;` returns `id|name\n1|ada\n2|bea\n3|cal\n`.

## Phase 4: CLI SQL Evidence
- [ ] T8 Add or confirm a `primary_key` filtered test in `tests/sql_exec.rs` for the exact combined SQL input from the contract.
  - Assert exit code `0`.
  - Assert empty stderr.
  - Assert stdout `id|name\n1|ada\n2|bea\n3|cal\nid|name\n2|bea\nid|name\n`.
- [ ] T9 Add or confirm a same-database-path reopen test in `tests/sql_exec.rs`.
  - Populate through one `db exec` process.
  - Query through a new `db exec` process.
  - Assert ordered scan and exact lookup stay identical.
- [ ] T10 Add or confirm duplicate primary-key insert evidence in `tests/sql_exec.rs`.
  - Assert duplicate insert exits `2`.
  - Assert stdout is empty.
  - Assert exact semantic duplicate-primary-key stderr.
  - Assert a follow-up select proves the existing row is unchanged.

## Phase 5: Persisted Duplicate Fixture Evidence
- [ ] T11 Add or confirm a valid persisted duplicate-primary-key fixture test.
  - Use a valid SQL storage catalog record for `users`.
  - Use two valid SQL row records for the same table.
  - Both rows must use primary key `2`.
  - Payloads must differ: `bea` and `dupe`.
  - Do not substitute malformed tag, unknown tag, broken prefix, or corrupt length evidence.
- [ ] T12 Assert the fresh reopen/rebuild path exits `1`, stdout is empty, and stderr is exactly `error: invalid SQL storage record: duplicate primary key for table users: 2\nhint: primary key values must be unique in persisted SQL storage.\n`.
- [ ] T13 If the test exposes a mismatch, repair only the relevant persisted duplicate-primary-key error labeling/path in `src/sql.rs` or test fixture encoding.

## Phase 6: Focused Verification Script
- [ ] T14 Add `scripts/verify_primary_index_acceptance`.
  - Resolve repo root relative to the script location, matching `scripts/verify` style.
  - Run `cargo test --test primary_index`.
  - Run `cargo test --test sql_exec primary_key`.
  - Ensure executable bit is set.

## Phase 7: Evidence Mapping And Docs
- [ ] T15 Create `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`.
  - Map each acceptance scenario to `gate-v1-indexes`.
  - Map each acceptance scenario to `REQ-7-implement-integer-primary-key-as-9c698e08`.
  - Include required command coverage per scenario.
- [ ] T16 Run required commands and capture exit-code/pass-fail summaries:
  - `cargo test --test primary_index`
  - `cargo test --test sql_exec primary_key`
  - `scripts/verify`
  - `scripts/verify_primary_index_acceptance` if added.
- [ ] T17 Create `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`.
  - Include current managed repo SHA.
  - Include command results and exit codes.
  - Include final mapping for `REQ-7-implement-integer-primary-key-as-9c698e08`.
  - Explicitly state no completion claim is made for `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, or `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`.
- [ ] T18 Update `docs/v1_acceptance.md` only after evidence exists.
  - Add or update the `gate-v1-indexes` row for `REQ-7-implement-integer-primary-key-as-9c698e08`.
  - Include current SHA and final evidence path.
  - Do not claim unrelated index requirements.

## Phase 8: Final Run Reporting
- [ ] T19 Record the implementation run result with final evidence path and command results.
- [ ] T20 Stop and escalate if a second recovery attempt becomes necessary.

## Notes For Implementer
- Do not edit `spec.md` or `contracts.md`.
- Do not edit `ssot/` or `policies/` without explicit escalation.
- Keep changes scoped to primary-index current-artifact evidence.
- Preserve stable CLI stdout, stderr, and exit codes exactly as contracted.
- The scheduler terminal result is supporting evidence only; `qa_mapping.md` and `final_review.md` remain required evidence paths.

