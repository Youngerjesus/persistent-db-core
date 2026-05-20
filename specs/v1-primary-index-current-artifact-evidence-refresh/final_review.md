# Final Review: Primary Index Current-Artifact Evidence Refresh

Verdict: PASS

## Scope

- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Base HEAD SHA: `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`
- Verified artifact identity: task worktree based on base HEAD
  `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` plus the current primary-index
  implementation, test, helper, docs, and evidence delta listed below.
- Package identity manifest:
  `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256`
  hashes the tracked source/test/docs diff plus cited helper and stable feature
  evidence files. It excludes the manifest itself and mutable latest review
  reports such as `impl_brake_review.md` and any future `impl_review.md`.
- Tracked source/test/docs delta digest:
  `67ea135ba7948903be9c683b92cc715964b3a6def730672595197f311e738826`
  for `git diff -- src/main.rs src/sql.rs tests/primary_index.rs tests/sql_exec.rs docs/v1_acceptance.md docs/cli_contract.md docs/sql_subset.md docs/file_format.md`.
- Evidence path: `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
- QA mapping: `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`

This review claims only `REQ-7-implement-integer-primary-key-as-9c698e08`.
It does not claim completion for `REQ-7-create-index-must-create-disk-3b71a7dc`,
`REQ-7-insert-update-and-delete-must-997871f9`, or
`EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`.

## Verification Evidence

The command evidence below is for the verified task worktree artifact described
above, not for base HEAD `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` alone.

| Command | Exit code | Result |
| --- | ---: | --- |
| `cargo test --test primary_index` | 0 | PASS: 7 tests passed, including primitive insert/get/missing/duplicate/no-overwrite, ordered positions, empty traversal, persisted reopen/rebuild lookup, and valid duplicate persisted-row invalid-storage failure. |
| `cargo test --test sql_exec primary_key` | 0 | PASS: 16 filtered tests passed, including combined ordered scan/exact lookup/missing lookup stdout, same-path reopen process evidence, duplicate insert stderr/no mutation, and valid duplicate persisted-row reopen failure. |
| `scripts/verify_primary_index_acceptance` | 0 | PASS: focused acceptance script ran `cargo test --test primary_index` and `cargo test --test sql_exec primary_key` from repo root. |
| `scripts/verify` | 0 | PASS: baseline verification passed `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help`. |

## Scenario Mapping

| Scenario | Evidence |
| --- | --- |
| `PI-001` | `tests/primary_index.rs::primary_index_insert_find_missing_duplicate_and_len` verifies `2 -> 0`, `1 -> 1`, exact lookups, missing key, duplicate rejection, and no overwrite. |
| `PI-002` | `tests/primary_index.rs::primary_index_ordered_positions_are_ascending_by_key` and `primary_index_empty_ordered_positions_are_empty` verify `[1, 2, 0]` and empty traversal. |
| `PI-003` | `tests/primary_index.rs::primary_index_rebuild_from_persisted_rows_survives_reopen` verifies persisted SQL rows rebuild the primary index for exact lookup and ordered scan after reopen. |
| `PI-004` | `tests/sql_exec.rs::primary_key_combined_contract_input_outputs_ordered_scan_lookup_and_missing_header` verifies the combined SQL input, exit code `0`, empty stderr, and exact stdout. |
| `PI-005` | `tests/sql_exec.rs::primary_key_same_path_reopen_preserves_ordered_scan_and_exact_lookup` verifies a later `db exec` process on the same path preserves ordering and lookup. |
| `PI-006` | `tests/sql_exec.rs::primary_key_duplicate_insert_in_new_process_keeps_existing_row_unchanged` verifies duplicate insert exit `2`, empty stdout, exact semantic stderr, and no row mutation. |
| `PI-007` | `tests/primary_index.rs::primary_index_duplicate_persisted_key_fails_as_invalid_storage_record` and `tests/sql_exec.rs::primary_key_valid_persisted_duplicate_row_fixture_fails_on_reopen` verify valid catalog and row fixtures with duplicate primary key `2` fail on reopen with exit `1`, empty stdout, and exact invalid-storage stderr. |
| `PI-008` | This final review and `docs/v1_acceptance.md` explicitly connect the gate, requirement id, repaired worktree artifact identity, command evidence, and excluded non-claims. |

## Implementation Review Notes

- `src/sql.rs` now preserves a specific persisted duplicate-primary-key storage error while replaying records for `db exec`.
- `src/main.rs` renders that storage error as `error: invalid SQL storage record: duplicate primary key for table users: 2` with the persisted-storage uniqueness hint.
- `db check` invariant labeling remains stable: duplicate persisted primary keys still map to the documented `primary index` check label.
- `docs/cli_contract.md`, `docs/sql_subset.md`, and `docs/file_format.md`
  document the duplicate persisted primary-key invalid-storage stderr while
  preserving the generic unknown-record-tag invalid-storage stderr for other
  corrupt SQL logical records.

## Worktree Delta

- Modified tracked files: `src/main.rs`, `src/sql.rs`, `tests/primary_index.rs`,
  `tests/sql_exec.rs`, `docs/v1_acceptance.md`, `docs/cli_contract.md`,
  `docs/sql_subset.md`, `docs/file_format.md`.
- Retained focused helper: `scripts/verify_primary_index_acceptance`.
- Task-scoped evidence root:
  `specs/v1-primary-index-current-artifact-evidence-refresh/`.
- Mutable latest review reports are verifier/brake SSOT inputs and are not part
  of the frozen package identity manifest.
