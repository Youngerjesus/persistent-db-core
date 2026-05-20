# QA Mapping: v1-transaction-wal-current-artifact-evidence-refresh

## Scope

This QA prep manifest covers the Transaction WAL recovery current-artifact evidence refresh package. The feature is evidence-heavy: acceptance depends on freshly generated command logs, current repo SHA identity, WAL sidecar/reopen proof, crash matrix report evidence, and exact requirement-ID mapping under this artifact slug. It is not a visual or UX task.

Core application files are out of scope for QA prep. The implementation phase may remain evidence-only if the existing product behavior and verification scripts pass.

## Scenario Expansion Lens

| Scenario ID | Pressure Path | QA Expectation |
| --- | --- | --- |
| `WAL-CUR-001` | stale evidence from prior WAL tasks | Current proof must come from this artifact root and current `git rev-parse HEAD`, not from scheduler success or prior evidence packages. |
| `WAL-CUR-002` | missing or dirty live repo identity | `current-repo-sha.txt` records `git rev-parse HEAD`, `git status --short`, and required file presence before claiming current-artifact proof. |
| `WAL-CUR-003` | committed write happy path | Focused WAL test and separate-process smoke prove committed rows survive reopen/recovery. |
| `WAL-CUR-004` | rollback or uncommitted WAL bytes | Focused fixture test proves rolled-back or uncommitted frame content is not public after recovery. |
| `WAL-CUR-005` | incomplete tail or partial WAL state | Focused fixture tests prove incomplete tail is excluded and later committed frames remain replayable after cleanup. |
| `WAL-CUR-006` | duplicate/retry/re-entry recovery | Idempotence evidence remains distinct from incomplete-tail evidence; repeated recovery must not duplicate committed data. |
| `WAL-CUR-007` | checkpoint/log-truncation interruption | `scripts/verify_crash_matrix` must validate `CM-001` through `CM-006`; generic green tests are insufficient for the data-loss-risk row. |
| `WAL-CUR-008` | dependency or validator failure | Missing/failing `scripts/verify_crash_matrix`, missing focused test names, or missing artifact refs are blockers, not partial success. |
| `WAL-CUR-009` | trust boundary and temp path leakage | WAL sidecar smoke may redact machine-specific temp parent paths, but must keep command, exit code, stdout/stderr, sidecar identity, existence, and byte lengths. |
| `WAL-CUR-010` | retry/re-entry artifact reuse | Fresh repair or verification pass deletes, replaces, or regenerates canonical current evidence; historical artifacts remain audit-only. |

## Provenance Contract

- Evidence root: `specs/v1-transaction-wal-current-artifact-evidence-refresh/`.
- Required artifact list:
  - `qa_mapping.md`
  - `verify_evidence_contract.sh`
  - `evidence/current-repo-sha.txt`
  - `evidence/command-log.md`
  - `evidence/requirement-evidence.md`
  - `evidence/wal-sidecar-smoke.md`
  - `evidence/crash-matrix-log.md`
  - `final_review.md`
- Scenario/evidence IDs: `WAL-CUR-001` through `WAL-CUR-010`, plus crash matrix case IDs `CM-001` through `CM-006`.
- Product evidence identity source: the managed-repo product invocation and current repo identity from `git rev-parse HEAD`; sidecar identity comes from the explicit implementation smoke invocation using a fresh `$DB_PATH` and the derived `$DB_PATH.wal`. Scheduler/control-plane ids such as `active_run_id`, `qa_prep_*`, `plan_*`, `impl_*`, `code_review_*`, and `final_*` are not product evidence identities and must not be used as exact acceptance values in managed-repo evidence.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass is deleted, replaced, or regenerated from the current verifier/product invocation. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: prior `v1-transaction-wal-recovery` and `v1-wal-recovery-current-sha-proof` command output, WAL byte lengths, smoke transcripts, and reviews are reference context only; they do not satisfy this current-artifact contract unless regenerated under this evidence root.
- Writer/validator separation expectation: QA prep writes this mapping and the red evidence scaffold; implementation writes the required evidence artifacts and `final_review.md`; a later verifier or reviewer validates the outputs with `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` and contract-required commands.
- Redaction target list: user-local absolute temp directories, machine-specific temp parents, and transient shell working directories may be redacted in prose. Do not redact command names, requirement IDs, current SHA, dirty-state output, exit codes, stdout/stderr, WAL sidecar identity `$DB_PATH.wal`, WAL existence, WAL byte lengths, or crash matrix case IDs. No secrets are expected.

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `T1` | qa-ready | Provenance, repo identity, dirty-state review, required-file existence | `verify_evidence_contract.sh` validates `evidence/current-repo-sha.txt` | `git rev-parse HEAD`; `git status --short`; `test -f tests/wal_recovery.rs`; `test -f tests/crash_matrix.rs`; `test -x scripts/verify`; `test -x scripts/verify_crash_matrix` | `evidence/current-repo-sha.txt` records commands, exit codes or observed output, current SHA, dirty state, and presence of the required files. | Covers stale evidence, partial state, and retry/re-entry identity risk. |
| `T2` | qa-ready | Baseline repo verification | Existing Rust/unit/integration tests through `scripts/verify`; scaffold checks command-log shape | `scripts/verify` | `evidence/command-log.md` records `scripts/verify` exit `0` and enough output to prove fmt, clippy, full tests, and help smoke ran at current SHA. | Missing tools, skipped checks, or non-zero exit are blockers. |
| `T3` | qa-ready | Focused WAL recovery tests, negative WAL fixture coverage, idempotence coverage | `tests/wal_recovery.rs`; scaffold checks requirement/test names | `cargo test --test wal_recovery`; focused test commands for all contract-named cases | Full WAL suite and each focused test exit `0`; `requirement-evidence.md` maps committed replay, rollback/uncommitted exclusion, incomplete-tail exclusion, idempotence, and ahead-of-store deterministic failure to exact requirement IDs. | Keeps incomplete-tail and repeated recovery/idempotence as distinct proof layers. |
| `T4` | qa-ready | CLI smoke, separate process reopen, WAL sidecar file-state proof | Existing `db` binary via `cargo run --bin db -- exec`; scaffold checks `wal-sidecar-smoke.md` | `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`; `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"` | Create/insert exits `0` with empty stdout/stderr; separate reopen/select exits `0`, stderr empty, stdout exactly `id\|name\n1\|ada\n2\|bea\n`; `$DB_PATH.wal` exists with positive byte length after both steps. | Temp path may be redacted, but sidecar identity and byte lengths must remain visible. |
| `T5` | qa-ready | Crash matrix validator, checkpoint/log-truncation interruption safety, crash report provenance | `tests/crash_matrix.rs`; `scripts/verify_crash_matrix`; generated `target/crash_matrix/crash_matrix_report.md` | `scripts/verify_crash_matrix` | Script exits `0`; `crash-matrix-log.md` records validator outcome, report path, and `CM-001` through `CM-006` summaries that directly support deterministic recovery or deterministic failure. | If the script is absent, fails, or lacks direct safety evidence, the package is `blocking`. |
| `T6` | qa-ready | Requirement evidence matrix, exact requirement ID coverage, blocker routing | `verify_evidence_contract.sh` validates `evidence/requirement-evidence.md` and `final_review.md` | manual write; `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` after implementation evidence exists | Every `REQ-8-*` and `REQ-9-*` row includes exact ID, command, expected behavior, observed result, artifact refs, and blocker status. | Generic "verification passed" wording is insufficient. |
| `T7` | qa-ready | Durable doc drift review | `docs/file_format.md`; `docs/cli_contract.md`; `docs/v1_acceptance.md` | manual review; `git diff -- docs/file_format.md docs/cli_contract.md docs/v1_acceptance.md`; rerun `scripts/verify` only if docs change | Docs are unchanged if current WAL sidecar, replay, retained-frame, rollback/incomplete, and crash-matrix behavior are already accurate; otherwise smallest doc update is made and verified. | Manual-only QA criterion: the scaffold cannot determine semantic doc drift, so implementation must record the doc review outcome in `final_review.md`; QA prep does not edit docs. |
| `T8` | qa-ready | Final review artifact, non-visual evidence note, artifact gate linkage | `final_review.md`; scaffold validates final review shape | `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`; all contract-required verification commands | `final_review.md` includes current SHA, command evidence, exact requirement IDs, artifact paths, `gate-v1-transactions-wal-recovery`, non-visual not-applicable note, and `Verdict: PASS` only when all required evidence exists. | Use explicit blocker language instead of PASS if crash matrix or any requirement row is unproven. |

## Preferred Commands

- `bash -n specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`
- `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`
- `scripts/verify`
- `cargo test --test wal_recovery`
- `cargo test --test wal_recovery committed_wal_replay_survives_reopen_via_cli`
- `cargo test --test wal_recovery rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`
- `cargo test --test wal_recovery incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`
- `cargo test --test wal_recovery committed_frame_after_incomplete_tail_cleanup_remains_replayable`
- `cargo test --test wal_recovery committed_wal_frame_ahead_of_page_store_fails_deterministically`
- `scripts/verify_crash_matrix`

## Red Scaffold

`verify_evidence_contract.sh` is a task-scoped QA scaffold. It validates the implementation-phase evidence file set and rejects broad keyword-only reports by requiring current SHA identity, required-file presence, command-specific adjacent `exit_code: 0` evidence, all exact requirement IDs, per-requirement fields for command/expected behavior/observed result/artifact refs/blocker status, WAL sidecar smoke fields, crash matrix case IDs, final gate linkage, non-visual evidence status, and absence of scheduler/control-plane identity values in implementation-owned evidence files. It is expected to fail during QA prep while the implementation evidence directory and `final_review.md` are not yet present.

## QA Prep Red Evidence

| Command | Exit code | Result | QA-prep finding |
| --- | ---: | --- | --- |
| `bash -n specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` | `0` | scaffold syntax valid | The evidence-contract validator is runnable from the repo root. |
| `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` | `1` | expected red | Fails on missing current implementation evidence artifact `evidence/current-repo-sha.txt`; this is the intended pre-implementation red state. |
| `cargo test --test wal_recovery` | `0` | existing behavior green | 5 tests passed: `committed_wal_replay_survives_reopen_via_cli`, `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`, `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`, `committed_frame_after_incomplete_tail_cleanup_remains_replayable`, and `committed_wal_frame_ahead_of_page_store_fails_deterministically`. This does not close the artifact because current evidence files and final review are still implementation-phase outputs. |

## QA Prep Retry Repair Evidence

Latest review SSOT: `specs/v1-transaction-wal-current-artifact-evidence-refresh/qa_prep_review.md`.

| Review finding | Repair |
| --- | --- |
| Missing canonical QA prep review artifact | `qa_prep_review.md` is present and treated as latest SSOT for this retry pass. |
| Scaffold did not fully enforce Task-Scoped Green criteria | `verify_evidence_contract.sh` now checks required-file presence in `current-repo-sha.txt`, command-specific adjacent `exit_code: 0` proof for every required command, and structured requirement row fields. |
| Product evidence artifacts were not guarded against scheduler/control-plane IDs | `verify_evidence_contract.sh` now rejects `active_run_id` and phase-run style `qa_prep_*`, `plan_*`, `impl_*`, `code_review_*`, and `final_*` values in implementation-owned evidence files while leaving explanatory mentions in this QA mapping allowed. |

Retry command evidence:

- `bash -n specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`: rerun after repair, exit `0`.
- `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`: rerun after repair, expected red exit `1` on missing implementation evidence artifact `evidence/current-repo-sha.txt`.

## Testing-Review Lens

- All task IDs `T1` through `T8` are covered in the mapping table.
- Preferred commands are concrete and runnable from the repository root.
- Task-scoped green criteria name exact commands, exit codes, output, artifact paths, and requirement IDs where relevant.
- Negative and boundary coverage includes stale evidence, dirty state, missing files, rollback/uncommitted WAL bytes, incomplete tail, duplicate/idempotent recovery, ahead-of-store deterministic failure, crash matrix interruption safety, temp path redaction, and retry/re-entry artifact reuse.
- The scaffold checks QA/evidence artifacts only and does not bypass implementation by changing Rust code, public CLI behavior, storage behavior, or durable docs.
