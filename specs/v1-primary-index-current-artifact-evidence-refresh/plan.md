# Implementation Plan

## Objective
Refresh current-artifact evidence for `gate-v1-indexes` by proving the existing integer primary-key index behavior at current managed repo SHA and mapping that proof to `REQ-7-implement-integer-primary-key-as-9c698e08`.

## Boundaries
- In scope:
  - `tests/primary_index.rs`
  - `tests/sql_exec.rs`
  - `scripts/verify_primary_index_acceptance`
  - `docs/v1_acceptance.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
- Conditional scope:
  - `src/index.rs` or `src/sql.rs` only if the focused current-artifact evidence tests expose a real behavior or error-contract gap.
  - `docs/sql_subset.md`, `docs/file_format.md`, or `docs/cli_contract.md` only if implementation changes a documented user-facing or persisted-data contract. No such change is planned.
- Out of scope:
  - Secondary-index requirements, CREATE INDEX disk persistence requirements, UPDATE/DELETE index-maintenance requirements, new SQL grammar beyond existing primary-key behavior, networking, daemons, background rebuild workers, `ssot/`, and `policies/`.

## Requirement Mapping
| Requirement ID | Gate | Required evidence | Planned artifact |
|---|---|---|---|
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Primitive `PrimaryIndex` insert/get/duplicate/no-overwrite/ordered-position behavior | Focused assertions in `tests/primary_index.rs`; `cargo test --test primary_index`. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Reopen/rebuild primary index from persisted SQL rows; lookup id `2`; ordered full scan `1,2,3` | `tests/primary_index.rs`; `cargo test --test primary_index`. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | CLI combined SQL input exits `0`, stderr empty, stdout exactly matches ordered scan, exact lookup, and missing lookup headers | `tests/sql_exec.rs`; `cargo test --test sql_exec primary_key`. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Same database path reopened in a new `db exec` process preserves ordering and lookup | `tests/sql_exec.rs`; `cargo test --test sql_exec primary_key`. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Duplicate primary key insert exits `2`, stdout empty, exact semantic stderr, and existing row unchanged | `tests/sql_exec.rs`; `cargo test --test sql_exec primary_key`. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Valid persisted duplicate-row fixture fails on reopen with exit `1` and exact invalid-storage duplicate-primary-key stderr | `tests/primary_index.rs` or `tests/sql_exec.rs`; `cargo test --test primary_index`; `cargo test --test sql_exec primary_key` if duplicated there. |
| `REQ-7-implement-integer-primary-key-as-9c698e08` | `gate-v1-indexes` | Final evidence ties current SHA, commands, and scenario mapping to the current artifact id only | `qa_mapping.md`, `final_review.md`, optional `docs/v1_acceptance.md` row, and scheduler result. |

## Implementation Steps
1. Reconfirm current HEAD, dirty state, and relevant file presence before editing.
2. Read latest review/report files for the current feature if present. Since this is a fresh handoff, implementation should still check for `qa_prep_review.md`, `impl_review.md`, `impl_brake_review.md`, `code_review.md`, and `final_review.md` before repair work.
3. Add or refine `tests/primary_index.rs` assertions so the primitive scenarios exactly match the contract:
   - insert `2 -> 0` and `1 -> 1`;
   - `get(2) == Some(0)`, `get(1) == Some(1)`, `get(3) == None`;
   - duplicate `insert(2, 99)` returns error and keeps `get(2) == Some(0)`;
   - `ordered_positions()` returns `[1, 2, 0]` for `30 -> 0`, `-5 -> 1`, `10 -> 2`;
   - empty `ordered_positions()` returns `[]`.
4. Add or refine persisted SQL rebuild evidence in `tests/primary_index.rs`:
   - create `users (id INT PRIMARY KEY, name TEXT)`;
   - insert ids `2`, `1`, `3`;
   - reopen/rebuild through a new `db exec` call;
   - assert `SELECT * FROM users WHERE id = 2;` returns `id|name\n2|bea\n`;
   - assert `SELECT * FROM users;` returns `id|name\n1|ada\n2|bea\n3|cal\n`.
5. Add or refine `tests/sql_exec.rs` `primary_key` filtered tests for the exact combined SQL input and same-path reopen scenario.
6. Add or refine duplicate primary-key CLI evidence:
   - duplicate insert `INSERT INTO users VALUES (2, 'dupe');` exits `2`;
   - stdout is empty;
   - stderr exactly matches the semantic duplicate-primary-key text;
   - a subsequent select proves existing row `2|bea` remains.
7. Add or refine the valid persisted duplicate primary-key fixture:
   - append a valid SQL storage catalog record for `users`;
   - append two valid row records for the same table with primary key `2` and payloads `bea` and `dupe`;
   - execute a new `db exec` reopen path;
   - assert exit `1`, empty stdout, and exact invalid-storage duplicate-primary-key stderr.
8. Add `scripts/verify_primary_index_acceptance` as a focused repo-root portable script that runs:
   - `cargo test --test primary_index`
   - `cargo test --test sql_exec primary_key`
9. Create `qa_mapping.md` with scenario-by-scenario mapping to `gate-v1-indexes`, `REQ-7-implement-integer-primary-key-as-9c698e08`, tests, and required commands.
10. Run required verification:
    - `cargo test --test primary_index`
    - `cargo test --test sql_exec primary_key`
    - `scripts/verify`
    - optionally `scripts/verify_primary_index_acceptance` as focused supplemental evidence.
11. Create `final_review.md` with current managed repo SHA, exit codes, pass/fail results, and final review mapping for only `REQ-7-implement-integer-primary-key-as-9c698e08`.
12. Update `docs/v1_acceptance.md` only after passing evidence exists, adding a `gate-v1-indexes` row for `REQ-7-implement-integer-primary-key-as-9c698e08` with current SHA and final evidence path while not claiming the other excluded index requirement IDs.
13. Record the scheduler run result with final evidence path and command results. Do not treat the scheduler result as a substitute for `qa_mapping.md` and `final_review.md`.

## Verification Strategy
- Focused verification:
  - `cargo test --test primary_index`
  - `cargo test --test sql_exec primary_key`
  - `scripts/verify_primary_index_acceptance` if added
- Baseline verification:
  - `scripts/verify`
- Manual review evidence must explicitly confirm:
  - current SHA;
  - `gate-v1-indexes`;
  - `REQ-7-implement-integer-primary-key-as-9c698e08`;
  - no claims for `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, or `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`;
  - valid duplicate persisted-row fixture shape;
  - exact stdout/stderr/exit codes.

## Stop Conditions
- Stop and report a blocker if implementation requires changing `spec.md`, `contracts.md`, `ssot/`, or `policies/`.
- Stop and report a blocker if the approved contract conflicts with current repo behavior in a way that cannot be repaired inside the intended primary-index evidence scope.
- Escalate if a second recovery attempt becomes necessary.

