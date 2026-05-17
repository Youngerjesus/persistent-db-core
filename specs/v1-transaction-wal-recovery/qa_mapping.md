# QA Mapping: v1-transaction-wal-recovery

## Scope
- Phase: QA Prep Execution
- Current run id source: task metadata `active_run_id=qa_prep_exec_fresh_20260518_000602_594462_3e6c0674`
- Frozen inputs: `spec.md`, `contracts.md`, `tasks.md`, `plan.md`, `design.md`, `research.md`
- Implementation files intentionally untouched in this phase.

## Scenario Expansion Lens
| Scenario ID | Path | Pressure Applied | QA Consequence |
|---|---|---|---|
| WAL-A | committed mutation reopen | Separate writer and reader `db exec` processes; exact stdout/stderr/exit assertions | `committed_wal_replay_survives_reopen_via_cli` must assert create/insert exits `0` with empty streams, WAL sidecar exists, and reopen select returns exactly `id\|name\n1\|ada\n2\|bea\n`. |
| WAL-B | incomplete or rollback mutation absence | Public CLI has no rollback/incomplete command; fixture authors WAL bytes directly after CLI-created catalog | `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` must write one committed `1|ada` frame plus incomplete `9|ghost` frame, then assert CLI select shows only `1|ada`. |
| WAL-C | duplicate replay / already done | Retained WAL frames are replayed across repeated opens | Scenario B repeats the select after first replay; result must stay exactly one row and must not duplicate `1|ada`. |
| WAL-D | partial state / incomplete tail | Short payload after a syntactically valid frame prefix | Scenario B truncates the ghost frame to force incomplete-tail handling and absence of `9|ghost`. |
| WAL-E | dependency failure / semantic rejection | Failed SQL statements must not become committed WAL records | Covered by T3 mapping to existing `mid_command_failure_keeps_prior_successes_and_skips_later_statements` and primary-key duplicate tests; implementation may add/adjust only if routing changes create a gap. |
| WAL-F | old file compatibility | Existing database files without WAL must remain openable | Covered by existing SQL restart and storage tests plus T4 doc review; no new CLI output contract expected. |
| WAL-G | permission / trust boundary | WAL sidecar is local filesystem state derived from user-supplied DB path | Use temp directories only; no network, daemon, or cross-process concurrency assumptions in tests. |
| WAL-H | retry/re-entry | Fresh verification pass must not reuse previous launch evidence as current proof | Provenance Contract below requires current-run regeneration for implementation evidence. |

## Provenance Contract
This task is evidence-heavy because acceptance depends on generated command evidence, a task-scoped WAL fixture, smoke output, WAL file-state evidence, and current-run provenance beyond static checks.

- Evidence root: current implementation or verification phase report under the active run directory, plus inline command evidence referenced from that report.
- Required artifact list: `tests/wal_recovery.rs`, `docs/file_format.md`, optional `docs/cli_contract.md` only if public CLI behavior changes, verification command output for `cargo test`, `cargo test --test wal_recovery`, `./scripts/verify`, canonical CLI smoke output, and WAL file-state evidence summary.
- Scenario/evidence IDs: WAL-A committed CLI reopen, WAL-B incomplete ghost absence, WAL-C retained-frame idempotence, WAL-F old-file compatibility, DOC-WAL file-format compatibility note, VERIFY-BASE required command suite, SMOKE-WAL canonical smoke commands.
- Current-run id source: task metadata `active_run_id` and the scheduler result path for the current run.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: smoke outputs, WAL file-state summaries, and verification logs from previous runs are invalid as current proof even if command text matches.
- Writer/validator separation expectation: implementation phase may write code/docs and collect evidence; verifier/reviewer phase must independently validate the current files and command results without editing this mapping as a substitute for evidence.
- Redaction targets: temp database paths, machine-specific parent directories, process ids, timestamps/nanos suffixes, and any absolute path fragments not needed to identify repo files. No secrets are expected.

## Task Mapping
| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
|---|---|---|---|---|---|---|
| T1 | scaffolded-red | CLI integration; deterministic WAL fixture; exact stdout/stderr/exit; file-state assertion | `tests/wal_recovery.rs` | `cargo test --test wal_recovery` | Both WAL recovery tests pass without loosening exact outputs; WAL sidecar exists after committed mutations; incomplete `9|ghost` never appears; repeated reopen/select does not duplicate rows. | Covers T1.1 through T1.4. Fixture helpers freeze `PDBWAL1\0`, version `1`, frame id, `record_count_before`, state, payload kind, payload length, checksum, and SQL row record bytes. |
| T2 | mapped | Storage replay behavior; idempotence; incomplete-tail handling; old-file compatibility | `tests/wal_recovery.rs`, existing storage/SQL tests | `cargo test --test wal_recovery`; `cargo test` | Replay applies complete committed frames exactly once, ignores incomplete trailing frames, keeps existing files without WAL readable, and preserves existing storage tests. | Negative/boundary coverage is primarily WAL-B/C/D/F. Malformed non-trailing corruption can be covered in implementation if exposed through user-facing error behavior without broadening scope. |
| T3 | mapped | SQL mutation routing; semantic failure ordering; duplicate PK pre-append behavior; CLI output stability | `tests/wal_recovery.rs`, `tests/sql_exec.rs`, `tests/primary_index.rs` | `cargo test --test wal_recovery`; `cargo test` | Create-table and insert mutations write committed WAL before page-store append; failed statements do not create committed ghost rows; public stdout/stderr/exit contract remains unchanged. | Existing tests cover mid-command failure and duplicate primary-key failure before durable row exposure. Implementation should add focused tests only if WAL routing introduces a new observable gap. |
| T4 | mapped | Documentation/manual review; baseline verify | `docs/file_format.md`; `docs/cli_contract.md` only if CLI contract changes | `./scripts/verify`; manual doc diff review | `docs/file_format.md` documents WAL path, frame layout/framing, replay order, committed/rollback/incomplete handling, retained frames/idempotence, and old database behavior. | Expected `docs/cli_contract.md` status is unchanged because no public command/output/exit/stderr behavior should change. Final report must state this reason. |
| T5 | mapped | Verification evidence; smoke evidence; WAL file-state evidence; acceptance traceability | implementation phase result/report path | `cargo test`; `cargo test --test wal_recovery`; `./scripts/verify`; canonical `cargo run --bin db -- exec <temp-db> ...` smoke commands | Final report connects every acceptance item to current-run command output, docs, tests, WAL file-state summary, or an explicit blocker. | Current QA phase supplies red scaffold and red evidence only; implementation phase owns green verification and final report. |

## Subtask Coverage
| Subtask ID | Covered By |
|---|---|
| T1.1 | `committed_wal_replay_survives_reopen_via_cli` in `tests/wal_recovery.rs` |
| T1.2 | `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` in `tests/wal_recovery.rs` |
| T1.3 | WAL fixture constants/helpers in `tests/wal_recovery.rs` |
| T1.4 | `row_record` and `write_string_u16` helpers in `tests/wal_recovery.rs` |
| T2.1 | T1.3 helper constants define expected frozen values for implementation |
| T2.2 | WAL-B/D fixture requires reader to handle complete committed and incomplete trailing frames |
| T2.3 | WAL-C repeated select requires retained-frame idempotence by record count |
| T2.4 | Existing SQL/storage tests plus T4 old-file docs |
| T2.5 | WAL-A/B assert sidecar existence/retention expectations |
| T3.1 | WAL-A create-table command creates/uses WAL without CLI output changes |
| T3.2 | WAL-A insert commands create/uses WAL without CLI output changes |
| T3.3 | Existing `tests/sql_exec.rs` mid-command failure coverage; implementation may add if needed |
| T3.4 | Existing primary-key duplicate tests |
| T4.1-T4.4 | T4 doc mapping and `docs/file_format.md` manual review |
| T5.1-T5.3 | T5 evidence mapping and implementation phase report requirements |

## Testing-Review Lens
- All major Task IDs T1 through T5 are mapped.
- Preferred commands are concrete and runnable from repo root.
- Task-scoped green criteria use exact behavior, not generic completion language.
- Negative/boundary coverage includes incomplete tail, absent public rollback CLI, duplicate replay, failed semantic mutation ordering, old database compatibility, and temp-path-only trust boundary.
- Red scaffold avoids core implementation edits and should initially fail until WAL sidecar creation/replay is implemented.

## Red Evidence
- `cargo fmt --check`: pass after formatting the new scaffold.
- `cargo test --test wal_recovery`: red as expected. `committed_wal_replay_survives_reopen_via_cli` fails because no retained `<db-path>.wal` sidecar is created yet. `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` fails because fixture-authored committed WAL row `1|ada` is not replayed; observed stdout is `id|name\n` instead of expected `id|name\n1|ada\n`.
- `cargo test`: red only at `tests/wal_recovery.rs`; existing `cli_contract`, `page_storage`, `primary_index`, and `sql_exec` integration tests pass before the new WAL tests fail.
- `./scripts/verify`: red at the same `tests/wal_recovery.rs` failures after earlier verify stages pass.
