# QA Mapping: Primary Index Current-Artifact Evidence Refresh

## Scope

- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Excluded claims: `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`
- Current HEAD checked during QA prep: `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`
- Dirty state at QA prep entry: feature spec directory was untracked; QA prep added tests, a focused verifier script, and this mapping.

## Provenance Contract

- Evidence root: `specs/v1-primary-index-current-artifact-evidence-refresh/`
- Required artifacts:
  - `qa_mapping.md`
  - `final_review.md` after implementation evidence is green
  - `docs/v1_acceptance.md` row only after current command evidence exists
  - command output for `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, `scripts/verify`, and `scripts/verify_primary_index_acceptance`
- Scenario ids:
  - `PI-001` primitive insert/get/missing/duplicate/no-overwrite
  - `PI-002` ordered positions and empty index traversal
  - `PI-003` persisted row reopen/rebuild lookup and ordered scan
  - `PI-004` combined CLI SQL ordered scan, exact lookup, and missing lookup
  - `PI-005` same database path reopened by a new `db exec` process
  - `PI-006` duplicate primary-key insert exit/stderr/no mutation
  - `PI-007` valid persisted duplicate-row fixture fails on reopen with duplicate-primary-key invalid-storage error
  - `PI-008` final evidence identity, current SHA, and excluded-requirement non-claims
- Product evidence identity source: the managed repo verifier invocation and the current git SHA reported by `git rev-parse HEAD`; product tests must assert product-level CLI, persisted-record, and index behavior, not scheduler/control-plane run ids.
- Clean generation rule: canonical evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current verifier/product invocation. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: previous `v1-primary-btree-index` evidence, prior scheduler reports, and prior command output are references only; they cannot satisfy this current artifact package unless regenerated at current SHA.
- Writer/validator separation expectation: QA prep writes the mapping and red scaffolds; implementation repairs behavior and records `final_review.md`; an independent verify/review phase validates the final evidence.
- Redaction target list: no secrets are expected. If command logs include local temp paths, they may remain in transient logs but must not be promoted as product evidence identity values.

## Scenario Expansion Lens

| Scenario | Happy path | Invalid or boundary path | Duplicate or retry path | Trust/dependency path |
| --- | --- | --- | --- | --- |
| `PI-001` | `PrimaryIndex::insert` stores `2 -> 0`, `1 -> 1` and lookup returns exact positions. | Missing key `3` returns `None`. | Duplicate `2 -> 99` returns error and keeps `2 -> 0`. | No external dependency. |
| `PI-002` | Keys `-5`, `10`, `30` return positions `[1, 2, 0]`. | Empty index returns `[]`. | Repeated ordered calls must be deterministic. | No external dependency. |
| `PI-003` | Persisted SQL rows rebuild primary index after reopen. | Missing lookup covered by `PI-004`. | Reopen must not change row order or state. | Depends on valid page storage and CLI process boundary. |
| `PI-004` | Combined SQL input returns ordered scan, lookup row, and missing header. | Missing key `9` returns header only with exit `0`. | Multiple selects in one command preserve deterministic stdout concatenation. | Black-box CLI contract, stdout/stderr/exit code stable. |
| `PI-005` | Same database path queried by a later process returns same ordering and lookup. | Partial state is represented by persistent store between commands. | Retry/re-entry through new process must rebuild, not reuse memory. | Process boundary protects against in-memory-only evidence. |
| `PI-006` | Existing row `2|'bea'` remains queryable. | Duplicate insert returns exit `2`, empty stdout, exact semantic stderr. | A second process duplicate attempt must not append or overwrite. | CLI error contract is the trust boundary. |
| `PI-007` | Fixture records are valid SQL catalog and row records. | Duplicate persisted key fails on reopen with exit `1`, empty stdout, exact invalid-storage duplicate stderr. | Fixture has two rows with key `2` and payloads `bea` and `dupe`. | Malformed tags, broken prefixes, and corrupt lengths are explicitly disallowed substitutes. |
| `PI-008` | Final evidence cites current SHA and required commands. | Excluded requirement ids remain non-claims. | Fresh verification replaces stale proof. | Scheduler terminal result is supporting evidence only. |

## Task Mapping

| Task | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `T1` | Complete for QA prep | Git state evidence | N/A | `git rev-parse HEAD`; `git status --short` | HEAD and dirty state are recorded before edits. | HEAD was `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`. |
| `T2` | Complete for QA prep | File existence evidence | N/A | shell file existence check | Required files exist before QA scaffold edits. | Confirmed `tests/primary_index.rs`, `tests/sql_exec.rs`, `src/index.rs`, `src/sql.rs`, `docs/v1_acceptance.md`, `scripts/verify`. |
| `T3` | Complete for QA prep | Review SSOT check | N/A | `find specs/v1-primary-index-current-artifact-evidence-refresh ...` | Latest review/report files are read if present. | No latest review/report file was present in the feature root. |
| `T4` | Complete for QA prep | Scope guard | N/A | manual review of spec/contract | No QA prep edit requires `spec.md`, `contracts.md`, `ssot/`, or `policies/`. | This phase did not edit protected areas or frozen inputs. |
| `T5` | Scaffolded | Unit/integration test | `tests/primary_index.rs` | `cargo test --test primary_index` | Insert/get/missing/duplicate/no-overwrite assertions pass. | Existing test covers `PI-001`. |
| `T6` | Scaffolded | Unit/integration test | `tests/primary_index.rs` | `cargo test --test primary_index` | Ordered positions `[1, 2, 0]` and empty `[]` pass. | Existing tests cover `PI-002`. |
| `T7` | Scaffolded | Black-box CLI reopen evidence | `tests/primary_index.rs` | `cargo test --test primary_index` | Same-path persisted rows return `id|name\n2|bea\n` and ordered full scan after reopen. | Existing test covers `PI-003`. |
| `T8` | Scaffolded | Black-box CLI stdout/stderr/exit | `tests/sql_exec.rs` | `cargo test --test sql_exec primary_key` | Exact combined SQL input exits `0`, stderr empty, stdout matches contract. | Added `primary_key_combined_contract_input_outputs_ordered_scan_lookup_and_missing_header`. |
| `T9` | Scaffolded | Process-boundary restart evidence | `tests/sql_exec.rs` | `cargo test --test sql_exec primary_key` | New `db exec` process on same path preserves ordered scan and exact lookup. | Added `primary_key_same_path_reopen_preserves_ordered_scan_and_exact_lookup`. |
| `T10` | Scaffolded | CLI negative-path evidence | `tests/sql_exec.rs` | `cargo test --test sql_exec primary_key` | Duplicate insert exits `2`, stdout empty, exact stderr, existing row unchanged. | Existing same-command test plus added new-process duplicate test cover `PI-006`. |
| `T11` | Red scaffolded | Valid persisted fixture evidence | `tests/primary_index.rs`; `tests/sql_exec.rs` | `cargo test --test primary_index`; `cargo test --test sql_exec primary_key` | Fixture uses valid SQL catalog with primary-key extension and two valid rows with key `2`, payloads `bea` and `dupe`. | Added direct fixture helpers with `P` primary-key extension. |
| `T12` | Red scaffolded | Invalid-storage error contract | `tests/primary_index.rs`; `tests/sql_exec.rs` | `cargo test --test primary_index`; `cargo test --test sql_exec primary_key` | Reopen exits `1`, stdout empty, stderr is duplicate-primary-key invalid-storage text. | Current implementation still emits generic unknown-record-tag stderr. |
| `T13` | Implementation pending | Source repair target | `src/sql.rs` if needed | focused tests above | If tests fail, repair only persisted duplicate-primary-key error labeling/path. | Red evidence identifies this as the implementation target. |
| `T14` | Scaffolded | Focused verifier script | `scripts/verify_primary_index_acceptance` | `scripts/verify_primary_index_acceptance` | Script resolves repo root and runs the two focused commands. | Added executable script; currently red at `primary_index` duplicate fixture. |
| `T15` | Complete for QA prep | QA manifest | `qa_mapping.md` | manual review | Every task and acceptance scenario maps to test files, commands, and green criteria. | This file is the canonical QA prep mapping. |
| `T16` | Red evidence captured | Command evidence | test suite and script | commands listed below | Required commands become green only after implementation repair. | Current focused and baseline evidence is red for `PI-007`. |
| `T17` | Deferred | Final evidence report | `final_review.md` | after green verification | Final review includes SHA, exit codes, pass/fail, and requirement mapping. | Do not create final green evidence during QA prep. |
| `T18` | Deferred | Durable docs traceability | `docs/v1_acceptance.md` | after green verification | Row cites `gate-v1-indexes`, requirement id, current SHA, final evidence path, and no excluded claims. | Update only after final evidence exists. |
| `T19` | Complete for QA prep run | Scheduler run result | run `result.md` | final PM result block | Result cites QA prep artifacts and red command evidence. | Scheduler result is supporting evidence only. |
| `T20` | Not triggered | Escalation guard | N/A | manual phase control | Escalate only if a second recovery attempt is required. | No second recovery attempt occurred in QA prep. |

## Preferred Commands

- `cargo test --test primary_index`
- `cargo test --test sql_exec primary_key`
- `scripts/verify_primary_index_acceptance`
- `scripts/verify`

## QA Prep Red Evidence

| Command | Exit code | Result | Current red reason |
| --- | ---: | --- | --- |
| `cargo test --test primary_index` | `101` | red | `primary_index_duplicate_persisted_key_fails_as_invalid_storage_record` expected duplicate-primary-key invalid-storage stderr, actual stderr was generic `unknown record tag`. |
| `cargo test --test sql_exec primary_key` | `101` | red | `primary_key_valid_persisted_duplicate_row_fixture_fails_on_reopen` expected duplicate-primary-key invalid-storage stderr, actual stderr was generic `unknown record tag`. |
| `scripts/verify_primary_index_acceptance` | `101` | red | Stops at `cargo test --test primary_index` with the same persisted duplicate fixture stderr mismatch. |
| `scripts/verify` | `101` | red | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` passed; full `cargo test` failed in `tests/primary_index.rs` on the persisted duplicate fixture stderr mismatch. |

## Testing-Review Lens

- All task ids `T1` through `T20` are represented in the mapping.
- Preferred commands are concrete and runnable from the repo root.
- Task-scoped green criteria name exact stdout, stderr, exit-code, command, and artifact requirements where applicable.
- Negative and boundary coverage includes missing lookup, empty index, duplicate insert, duplicate persisted storage, same-path reopen, and process-boundary rebuild.
- The red scaffolds do not require implementation shortcuts. They expose the contract gap in persisted duplicate primary-key invalid-storage reporting.
