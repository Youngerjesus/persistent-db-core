# QA Prep Verification Review

Verdict: PASS

## Scope Reviewed

- QA mapping: `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`
- Planning inputs: `spec.md`, `contracts.md`, `plan.md`, `design.md`, `tasks.md`
- QA scaffolds: `tests/primary_index.rs`, `tests/sql_exec.rs`, `scripts/verify_primary_index_acceptance`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Gate: `gate-v1-indexes`

## Findings

- `qa_mapping.md` maps all task ids `T1` through `T20` from `tasks.md`.
- Each actionable test task has concrete preferred commands and task-scoped green criteria.
- Deferred implementation-phase tasks `T17` and `T18` are correctly held until green command evidence exists.
- Scenario coverage is not limited to happy paths. It covers missing lookup, empty index traversal, same-path reopen, process-boundary rebuild, duplicate insert rejection/no mutation, valid persisted duplicate-row corruption, excluded requirement non-claims, and current-SHA evidence identity.
- The persisted duplicate fixture scaffold uses a valid SQL catalog payload with the `P` primary-key extension and two valid row payloads with key `2` and payloads `bea` and `dupe`; it does not substitute malformed tags, broken prefixes, or corrupt length fields.
- No blocking scheduler phase id leakage was found in durable product tests, `qa_mapping.md` task-scoped green criteria, or product evidence identity requirements. Scheduler/control-plane ids appear only as planning or review/report references.

## Reproduced Red Evidence

| Command | Exit | Verification Result |
| --- | ---: | --- |
| `cargo test --test primary_index` | 101 | Red at `primary_index_duplicate_persisted_key_fails_as_invalid_storage_record`; expected duplicate-primary-key invalid-storage stderr, actual generic `unknown record tag` stderr. |
| `cargo test --test sql_exec primary_key` | 101 | Red at `primary_key_valid_persisted_duplicate_row_fixture_fails_on_reopen`; expected duplicate-primary-key invalid-storage stderr, actual generic `unknown record tag` stderr. |
| `scripts/verify_primary_index_acceptance` | 101 | Red at the same focused `primary_index` persisted duplicate fixture. |
| `scripts/verify` | 101 | Baseline progressed through fmt/clippy and earlier tests, then failed at `tests/primary_index.rs` on the same persisted duplicate fixture. |

## Implementation Handoff

The QA contract is consumable by `impl_exec`. The intended repair target is narrow: update the persisted duplicate primary-key reopen/rebuild error labeling/path so valid duplicate row records fail with the contracted invalid-storage duplicate-primary-key stderr while preserving existing CLI and storage contracts.

No retry checklist is required.
