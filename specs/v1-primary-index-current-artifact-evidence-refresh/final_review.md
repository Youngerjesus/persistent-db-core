# Final Review: Primary Index Current-Artifact Evidence Refresh

Verdict: PASS

## Scope

- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Current managed repo product SHA:
  `6008189f30b8e2cd38ad6ab5994c89c373d386ca`
- Base source SHA before this task:
  `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`
- Verified artifact identity: product commit
  `6008189f30b8e2cd38ad6ab5994c89c373d386ca` contains the primary-index
  implementation, focused tests, verifier helper, docs, and task evidence
  package that were verified for this refresh. This final evidence file,
  `docs/v1_acceptance.md`, and `artifact_identity.sha256` are current
  code-review-retry evidence repairs layered on that product SHA; they do not
  claim that these post-commit wording repairs are already contained inside
  `6008189f30b8e2cd38ad6ab5994c89c373d386ca`.
- Package identity manifest:
  `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256`
  hashes the tracked source/test/docs diff plus cited helper and stable feature
  evidence files. It excludes the manifest itself and mutable latest review
  reports such as `impl_brake_review.md` and any future `impl_review.md`.
- Tracked source/test/docs delta digest:
  `050453ffbeb520f80573f01d5c9413acbeb4d3a4f28797d972cc129c93994b4e`
  for `git diff main -- src/main.rs src/sql.rs tests/primary_index.rs tests/sql_exec.rs docs/v1_acceptance.md docs/cli_contract.md docs/sql_subset.md docs/file_format.md | shasum -a 256`.
- Evidence path: `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
- QA mapping: `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`

This review claims only `REQ-7-implement-integer-primary-key-as-9c698e08`.
It does not claim completion for `REQ-7-create-index-must-create-disk-3b71a7dc`,
`REQ-7-insert-update-and-delete-must-997871f9`, or
`EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`.

## Verification Evidence

The command evidence below was run from the current task worktree at product
SHA `6008189f30b8e2cd38ad6ab5994c89c373d386ca` plus the current evidence
repair files, not from base source SHA
`69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` alone.

| Command | Exit code | Result |
| --- | ---: | --- |
| `cargo test --test primary_index` | 0 | PASS: 7 tests passed, including primitive insert/get/missing/duplicate/no-overwrite, ordered positions, empty traversal, persisted reopen/rebuild lookup, and valid duplicate persisted-row invalid-storage failure. |
| `cargo test --test sql_exec primary_key` | 0 | PASS: 16 filtered tests passed, including combined ordered scan/exact lookup/missing lookup stdout, same-path reopen process evidence, duplicate insert stderr/no mutation, and valid duplicate persisted-row reopen failure. |
| `scripts/verify_primary_index_acceptance` | 0 | PASS: focused acceptance script ran `cargo test --test primary_index` and `cargo test --test sql_exec primary_key` from repo root. |
| `scripts/verify` | 0 | PASS: baseline verification passed `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, full `cargo test`, and `cargo run --bin db -- --help`. |

Finish-session recheck on 2026-05-20T22:57:28+0900:

- `cargo test --test primary_index` exited `0`; 7 tests passed.
- `cargo test --test sql_exec primary_key` exited `0`; 16 filtered primary-key tests passed.
- `scripts/verify` exited `0`; baseline verification completed successfully.
- `scripts/verify_primary_index_acceptance` exited `0`; focused acceptance helper completed successfully.

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
| `PI-008` | This final review and `docs/v1_acceptance.md` explicitly connect the gate, requirement id, current managed repo product SHA, current evidence repair identity, command evidence, and excluded non-claims. |

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

## Closure Checks

- `gate-v1-indexes` and `REQ-7-implement-integer-primary-key-as-9c698e08` are explicitly mapped to focused tests, the acceptance helper, baseline verification, `qa_mapping.md`, and `docs/v1_acceptance.md`.
- Non-target index requirements remain excluded from this claim.
- No protected `ssot/` or `policies/` files were changed.

## Open Items

None.

## Remote State

Pending finish push, PR, and merge.

## Next Action

Run finish commit, push, PR creation, merge, and scheduler manifest handoff.

## Updated At

2026-05-20T22:57:28+0900
