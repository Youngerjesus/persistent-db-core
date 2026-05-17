# QA Mapping: v1-wal-recovery-current-sha-proof

## Scope

This QA prep manifest covers the current-SHA WAL recovery proof closure task.
It is intentionally evidence-heavy: acceptance depends on freshly generated
command transcripts, WAL sidecar file-state evidence, current-run provenance,
and an acceptance mapping report, not only on static code or Rust test results.

Core implementation files are not modified during QA prep.

## Scenario Expansion Lens

| Scenario ID | Pressure Path | QA Expectation |
| --- | --- | --- |
| WAL-PROV-A | stale prior SHA evidence | Final evidence must capture `git rev-parse HEAD` from the current task worktree and must not reuse the prior manifest SHA proof. |
| WAL-COMMIT-A | committed mutation process reopen | Separate `db exec` create/insert and select processes must show `id\|name\n1\|ada\n2\|bea\n` with empty stderr. |
| WAL-UNCOMMITTED-A | no public rollback or incomplete command | Direct WAL fixture bytes are acceptable only with rationale that they represent V1-observable recovery-boundary bytes. |
| WAL-INCOMPLETE-A | incomplete trailing WAL entry | The `9|ghost` row must not appear after recovery; cleanup must leave the sidecar replayable or cleaned to a replayable prefix. |
| WAL-RETRY-A | duplicate/already-applied retained frames | Repeated reopen/select must be idempotent and must not duplicate replayed rows. |
| WAL-CORRUPT-A | dependency/order failure | A committed frame ahead of the page store must fail deterministically with the documented storage error surface. |
| WAL-FILESTATE-A | partial evidence capture | WAL sidecar existence and byte length must be captured immediately after create/insert and again after reopen/select. |
| WAL-DOC-A | stale docs or public CLI drift | Docs must be reviewed for WAL sidecar/replay semantics and CLI output stability; update only if stale or contradictory. |
| WAL-PERM-A | trust boundary and filesystem state | Temp paths must be local, no network or daemon evidence, and temp DB path may be redacted while keeping sidecar metadata auditable. |
| WAL-RERUN-A | retry/re-entry | Fresh verification or repair pass must delete, replace, or regenerate current proof artifacts; historical proof remains audit-only. |

## Provenance Contract

- Evidence root: `specs/v1-wal-recovery-current-sha-proof/`.
- Required artifact list: `qa_mapping.md`, `verify_evidence_contract.sh`, implementation-phase `final_report.md` or equivalent evidence transcript, `cargo test --test wal_recovery` transcript or captured output, `./scripts/verify` transcript or captured output, canonical CLI smoke transcript, WAL sidecar state after create/insert, WAL sidecar state after reopen/select, acceptance mapping to `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, and `req-v1-wal-recovery-proof`.
- Scenario/evidence IDs: `WAL-PROV-A`, `WAL-COMMIT-A`, `WAL-UNCOMMITTED-A`, `WAL-INCOMPLETE-A`, `WAL-RETRY-A`, `WAL-CORRUPT-A`, `WAL-FILESTATE-A`, `WAL-DOC-A`, `WAL-PERM-A`, `WAL-RERUN-A`.
- Current-run id source: implementation-phase scheduler metadata observed at implementation time. The final evidence must record the active implementation run id, the implementation scheduler result path, and the observed `git rev-parse HEAD`. The QA-prep run ids are historical QA-prep provenance only and are not valid as implementation current-run evidence.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass is deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: prior `v1-transaction-wal-recovery` reviews, prior smoke output, prior WAL byte lengths, and prior verification logs are invalid as current proof even when command text and observed behavior match.
- Writer/validator separation expectation: the implementation pass writes `final_report.md` or equivalent evidence transcript; the verifier or QA reviewer validates it against this mapping and may run `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`.
- Redaction target list: temp database parent directories, user-local absolute temp paths, and machine-specific run directories may be redacted in prose, but command text, exit codes, stdout/stderr, HEAD SHA, dirty-state output, WAL sidecar existence, and WAL byte lengths must remain visible.

## Final Evidence Schema

`verify_evidence_contract.sh` validates the implementation-phase
`final_report.md` against these machine-checkable evidence block IDs. The report
may include additional prose, but these blocks and fields must be present.

| Evidence Block | Required Fields |
| --- | --- |
| `### EV-PROVENANCE` | `implementation_active_run_id:`, `implementation_result_path:`, and current-run generation wording |
| `### EV-IDENTITY-HEAD` | `command: git rev-parse HEAD`, `exit_code: 0`, `stdout: "<40-hex-sha>"`, `stderr:` |
| `### EV-IDENTITY-STATUS` | `command: git status --short`, `exit_code: 0`, `stdout:`, `stderr:` |
| `### EV-TEST-WAL` | `command: cargo test --test wal_recovery`, `exit_code: 0`, and all five mapped WAL test names |
| `### EV-VERIFY-BASE` | `command: ./scripts/verify`, `exit_code: 0`, and references to fmt, clippy, full test suite, and `db --help` smoke |
| `### EV-SMOKE-CREATE-INSERT` | exact create/insert command, `exit_code: 0`, `stdout: ""`, `stderr: ""` |
| `### EV-WAL-AFTER-CREATE-INSERT` | `exists: true`, positive `byte_length:` |
| `### EV-SMOKE-REOPEN-SELECT` | exact select command, `exit_code: 0`, `stderr: ""`, `stdout: "id\|name\n1\|ada\n2\|bea\n"` |
| `### EV-WAL-AFTER-REOPEN-SELECT` | `exists: true`, positive `byte_length:` |
| `### EV-FIXTURE-RATIONALE` | separate rationale for direct WAL fixture coverage of V1-observable uncommitted/incomplete state and no public rollback/incomplete transaction command |
| `### EV-ACCEPTANCE-MAPPING` | explicit mapping to `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, and `req-v1-wal-recovery-proof` |

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | qa-ready | Provenance, git identity, dirty-state review, relevant-file existence review | `verify_evidence_contract.sh` validates final evidence shape | `git rev-parse HEAD`; `git status --short`; manual read of `tests/wal_recovery.rs`, `docs/file_format.md`, `docs/cli_contract.md`, `src/storage.rs`, `src/main.rs`, `src/lib.rs` | Final evidence records command, exit code, stdout, stderr, and states whether implementation started from observed SHA `33b480cac6cf9d505a86eda4c149a4471454f11d` or a newer task SHA. | Covers stale evidence and retry/re-entry paths. |
| T2 | qa-ready | Rust integration test proof, fixture rationale, negative recovery coverage | `tests/wal_recovery.rs`; `verify_evidence_contract.sh` checks report references | `cargo test --test wal_recovery` | Command passes at current implementation SHA; report maps committed reopen to `committed_wal_replay_survives_reopen_via_cli`, complete uncommitted/rolled-back absence to `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`, incomplete-tail exclusion to `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`, future replayability to `committed_frame_after_incomplete_tail_cleanup_remains_replayable`, and deterministic ahead-of-store failure to `committed_wal_frame_ahead_of_page_store_fails_deterministically`. | Direct WAL fixtures are required because public CLI has no rollback or incomplete transaction command; complete rolled-back and incomplete trailing frames must remain separate scenarios. |
| T3 | qa-ready | Baseline repo verification | `verify_evidence_contract.sh` checks report references | `./scripts/verify` | Report records exit `0` and summarizes fmt, clippy, full test suite, and `db --help` smoke from the script output. | Missing tools or skipped checks are blockers, not acceptable green. |
| T4 | qa-ready | CLI smoke proof, process-reopen proof, WAL file-state proof | `verify_evidence_contract.sh` checks report references | `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`; `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"` | Create/insert exits `0` with stdout `""` and stderr `""`; select exits `0` with stderr `""` and stdout exactly `id\|name\n1\|ada\n2\|bea\n`; WAL sidecar existence and byte length are recorded after both commands. | Captures partial-state risk where temp cleanup or extra opens could destroy required sidecar evidence. |
| T5 | qa-ready | Manual doc/code delta review | `docs/file_format.md`, `docs/cli_contract.md`, `src/main.rs`, `src/lib.rs`, `src/storage.rs` | Manual review; `git diff -- docs/file_format.md docs/cli_contract.md src/main.rs src/lib.rs src/storage.rs tests/wal_recovery.rs` | Report states whether this is evidence-only at current SHA or lists scoped edits with rationale; no public CLI stream/exit-code drift occurs unless escalated. | Existing docs currently describe WAL sidecar path, frame layout, replay order, incomplete-tail cleanup, retained frames, compatibility, and CLI durability wording. |
| T6 | qa-ready | Acceptance mapping, scheduler result, gate linkage | `qa_mapping.md`; implementation-phase `final_report.md`; scheduler `result.md` | `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`; write active run `result.md` | Report maps every Candidate Acceptance Criteria item to concrete evidence and explicitly maps `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, and `req-v1-wal-recovery-proof`; scheduler result uses `success` only when verifier-ready. | This task remains red until the implementation phase creates current-run final evidence. |

## Red Scaffold

`verify_evidence_contract.sh` is a task-scoped QA scaffold that validates the
implementation-phase final evidence transcript shape without touching app/core
implementation. It rejects broad keyword-only reports by requiring distinct
evidence blocks for identity, focused WAL tests, baseline verification, both CLI
smoke commands, both WAL sidecar states, fixture rationale, and gate mapping. It
is expected to fail during QA prep while `final_report.md` does not yet exist,
and to become green only after current-run evidence is generated.

## QA Prep Red Evidence

- `cargo test --test wal_recovery`: green at QA-prep baseline, 4 tests passed. This shows the existing Rust recovery scenarios are present but does not close the evidence-heavy task by itself.
- `test -f specs/v1-wal-recovery-current-sha-proof/final_report.md`: red before QA artifact generation because the current-run final evidence transcript is not present.
- `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`: expected red until implementation writes `final_report.md` with the required evidence block IDs, distinct command records, exact CLI stdout/stderr evidence, two WAL sidecar state records, fixture rationale, and acceptance mapping.

## QA Prep Retry Repair Evidence

- Latest QA review SSOT retained at `specs/v1-wal-recovery-current-sha-proof/qa_prep_review.md`.
- Must Fix 1 repaired by retaining the canonical latest QA-prep review report in the feature directory for verifier re-entry.
- Must Fix 2 repaired by strengthening `verify_evidence_contract.sh` to require separate machine-checkable evidence blocks for identity, focused WAL tests, baseline verification, create/insert smoke, after-create WAL state, reopen/select smoke, after-reopen WAL state, fixture rationale, and acceptance mapping.
- Must Fix 3 repaired by changing the provenance contract so implementation evidence must record the active implementation run id and scheduler result path observed at implementation time; QA-prep run ids are historical only.
- Retry command evidence:
  - `bash -n specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`: exit `0`.
  - `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`: expected red, exit `1`, stderr reports missing required current-run `final_report.md`.
  - `cargo test --test wal_recovery`: exit `0`, 4 passed.

## Testing-Review Lens

- All Task IDs T1 through T6 are covered in the mapping table.
- Preferred commands are concrete and runnable from the repository root.
- Task-scoped green conditions are specific about command output, exit codes, evidence content, and scenario names.
- Negative and boundary coverage includes stale SHA, uncommitted/incomplete WAL bytes, duplicate replay, ahead-of-store corruption, temp file-state capture, and retry/re-entry artifact reuse.
- The scaffold checks evidence artifacts only and does not bypass implementation by changing Rust code or public CLI behavior.
