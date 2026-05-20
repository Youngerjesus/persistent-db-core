# Implementation Verification Review: Primary Index Current-Artifact Evidence Refresh

Verdict: PASS

## Scope

- Phase: `impl_verify`
- Run: `impl_verify_1_fresh_20260520_215558_420212_18370c88`
- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Base HEAD SHA: `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`
- Verified artifact identity: base HEAD `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` plus the current uncommitted task worktree delta and task-scoped evidence package.
- Diff shape: `main...HEAD` has no committed delta; current verification target is the worktree delta shown by `git status --short` and `git diff`.
- Reviewed surfaces:
  - `specs/v1-primary-index-current-artifact-evidence-refresh/spec.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/contracts.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/impl_brake_review.md`
  - `docs/v1_acceptance.md`
  - `tests/primary_index.rs`
  - `tests/sql_exec.rs`
  - `src/sql.rs`
  - `src/main.rs`
  - `scripts/verify_primary_index_acceptance`

## Executed Checks

| Check | Exit code | Result |
| --- | ---: | --- |
| `git status --short` | 0 | Current worktree contains the expected modified source/test/docs files plus untracked `scripts/verify_primary_index_acceptance` and `specs/v1-primary-index-current-artifact-evidence-refresh/`. |
| `git rev-parse HEAD` | 0 | `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`. |
| `git log --oneline main..HEAD` | 0 | No committed delta from `main`; verification target is the worktree artifact. |
| `git diff --stat main...HEAD && git diff --name-only main...HEAD` | 0 | No committed diff from `main...HEAD`. |
| `git diff --stat && git diff --name-only` | 0 | Tracked delta is limited to docs, `src/main.rs`, `src/sql.rs`, `tests/primary_index.rs`, and `tests/sql_exec.rs`. |
| `cargo test --test primary_index` | 0 | 7 tests passed. |
| `cargo test --test sql_exec primary_key` | 0 | 16 filtered tests passed. |
| `scripts/verify_primary_index_acceptance` | 0 | Focused helper ran `primary_index` and filtered `sql_exec primary_key`; both passed. |
| `scripts/verify` | 0 | Baseline verification passed, including fmt, clippy, full test suite, and `cargo run --bin db -- --help`. |
| `git diff -- ... | shasum -a 256` | 0 | Tracked source/test/docs diff hash is `67ea135ba7948903be9c683b92cc715964b3a6def730672595197f311e738826`, matching `artifact_identity.sha256`. |
| `shasum -a 256 scripts/verify_primary_index_acceptance final_review.md qa_mapping.md` | 0 | Hashes match the manifest for the helper, final review, and QA mapping. |

## Evidence

- `tests/primary_index.rs::primary_index_insert_find_missing_duplicate_and_len` verifies primitive `PrimaryIndex` insert/get/missing duplicate behavior, including no overwrite after duplicate insert.
- `tests/primary_index.rs::primary_index_ordered_positions_are_ascending_by_key` and `primary_index_empty_ordered_positions_are_empty` verify `[1, 2, 0]` key ordering and empty traversal.
- `tests/primary_index.rs::primary_index_rebuild_from_persisted_rows_survives_reopen` verifies persisted SQL rows rebuild the primary index after reopen for exact lookup and ordered full scan.
- `tests/sql_exec.rs::primary_key_combined_contract_input_outputs_ordered_scan_lookup_and_missing_header` verifies the combined SQL input, exit code `0`, empty stderr, and exact stdout required by the contract.
- `tests/sql_exec.rs::primary_key_same_path_reopen_preserves_ordered_scan_and_exact_lookup` verifies the same database path through a later `db exec` process.
- `tests/sql_exec.rs::primary_key_duplicate_insert_in_new_process_keeps_existing_row_unchanged` verifies duplicate insert exit `2`, empty stdout, exact semantic stderr, and unchanged existing row.
- `tests/primary_index.rs::primary_index_duplicate_persisted_key_fails_as_invalid_storage_record` and `tests/sql_exec.rs::primary_key_valid_persisted_duplicate_row_fixture_fails_on_reopen` use valid SQL catalog and row records with duplicate primary key `2` and payloads `bea`/`dupe`, then verify exit `1`, empty stdout, and exact invalid-storage stderr.
- `docs/v1_acceptance.md` has a `gate-v1-indexes` row for only `REQ-7-implement-integer-primary-key-as-9c698e08`, citing `final_review.md`, `artifact_identity.sha256`, focused commands, and `scripts/verify`.
- `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md` explicitly excludes `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, and `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`.
- `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256` matches the current tracked diff and stable cited evidence artifacts checked in this verification pass.

## Primary Success Claims

1. The current worktree artifact supplies current-artifact evidence for `gate-v1-indexes` / `REQ-7-implement-integer-primary-key-as-9c698e08` without claiming the excluded index requirements.
2. Primary-key behavior is covered at both primitive index and black-box CLI SQL levels, including ordering, exact lookup, missing lookup, duplicate insert rejection, same-path process reopen, and persisted duplicate-row rebuild failure.
3. The final evidence package connects the current artifact identity, required commands, QA mapping, final review, and `docs/v1_acceptance.md` row to the same verified worktree.

## Evidence Used

- Commands executed in this verification pass: `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify_primary_index_acceptance`, and `scripts/verify`, all exit code `0`.
- Runtime observations from test output: `primary_index` ran 7 tests and passed; filtered `sql_exec primary_key` ran 16 tests and passed; full `scripts/verify` ran the repository verification suite and `db --help` successfully.
- Artifact observations: `git rev-parse HEAD` returned `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`; the tracked diff digest and the helper/final-review/QA-mapping file hashes match `artifact_identity.sha256`.
- Manual review observations: `qa_mapping.md` maps `PI-001` through `PI-008` to concrete tests and commands; `final_review.md` and `docs/v1_acceptance.md` cite only the target requirement for this task.

## Proxy Gap / Reward-Hacking Risk

- Prior verifier/brake history had the same broad failure class: stale or ambiguous current-artifact evidence could make old green tests look like proof for a different artifact identity.
- The worker modified tests, fixture writers, and a focused verifier script, so green tests could be a false pass if the fixtures were malformed or if the helper replaced rather than supplemented canonical evidence.
- `qa_mapping.md` still contains QA-prep red evidence as historical context, so a verifier could confuse stale red scaffold text with current final evidence or, inversely, let generated final artifacts substitute for actual command execution.
- Because `main...HEAD` is empty, a merge/closeout phase must preserve the untracked evidence and helper artifacts; otherwise `docs/v1_acceptance.md` would cite paths not present in the final tracked artifact.

## Gap-Closing Check

- The false-pass risk from stale artifact identity was closed by recomputing the tracked diff hash with `git diff -- src/main.rs src/sql.rs tests/primary_index.rs tests/sql_exec.rs docs/v1_acceptance.md docs/cli_contract.md docs/sql_subset.md docs/file_format.md | shasum -a 256`; it produced `67ea135ba7948903be9c683b92cc715964b3a6def730672595197f311e738826`, matching `artifact_identity.sha256`.
- The fixture-substitution risk was closed by reviewing `tests/primary_index.rs` and `tests/sql_exec.rs`: both duplicate-persisted-row fixtures append `PDBSQL1\0` catalog records with `P` primary-key extension and two valid row records for key `2` with payloads `bea` and `dupe`; they do not use malformed tags, broken prefixes, or corrupt length fields.
- The helper-substitution risk was closed by running canonical commands directly (`cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, and `scripts/verify`) in addition to `scripts/verify_primary_index_acceptance`.
- The stale-red-context risk was closed by comparing `qa_mapping.md` scenario entries with current `final_review.md`, current command output, and the current `docs/v1_acceptance.md` row.

## Open Findings

- None.

## Repair Targets

- None.

## Next Action

- Proceed to the next scheduler phase. Closeout/merge work must include the untracked `scripts/verify_primary_index_acceptance` and `specs/v1-primary-index-current-artifact-evidence-refresh/` artifacts because current docs and final evidence cite them.

## Updated At

- `2026-05-20T13:00:19Z`
