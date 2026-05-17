# QA Mapping: Minimal SQL Schema/Execute Path

## Scope And Evidence Classification

- Phase: `qa_prep_exec`
- Current run id source: scheduler metadata `qa_prep_exec_fresh_20260517_200233_876240_fa5473a7`
- Canonical inputs: `spec.md`, `contracts.md`, `tasks.md`, `plan.md`, `design.md`, `research.md`
- Evidence-heavy classification: not evidence-heavy. This is a CLI-only Rust database task; acceptance evidence is deterministic test and command output plus durable docs, with browser/visual/exported/redaction artifacts explicitly out of scope.
- Provenance contract: not required for this phase because no generated launch evidence, screenshots, browser artifacts, exported reports, evaluator outputs, redaction proof, or reusable external artifact bundle is accepted as proof.

## Scenario Expansion Lens

| Scenario ID | Pressure Path | QA Coverage |
|---|---|---|
| `S01` | Happy path creates schema, appends rows, selects in insertion order. | `tests/sql_exec.rs::happy_path_creates_inserts_and_selects_rows_in_insert_order` |
| `S02` | Empty/partial state select on an existing empty table. | `tests/sql_exec.rs::empty_table_select_outputs_header_only` |
| `S03` | Multiple result sets in one command repeat headers without separators. | `tests/sql_exec.rs::multiple_selects_repeat_headers_without_separators` |
| `S04` | Failure after prior `SELECT` must suppress partial stdout. | `tests/sql_exec.rs::failed_command_suppresses_partial_select_stdout` |
| `S05` | Restart/re-entry after a successful command preserves catalog and rows. | `tests/sql_exec.rs::restart_persists_catalog_and_rows` |
| `S06` | Mid-command semantic failure keeps prior durable records and skips later statements after restart. | `tests/sql_exec.rs::mid_command_failure_keeps_prior_successes_and_skips_later_statements` |
| `S07` | Identifier equality is ASCII case-insensitive while stored/output spelling is preserved. | `tests/sql_exec.rs::identifiers_compare_case_insensitively_but_preserve_catalog_spelling` |
| `S08` | Duplicate/already-done table and column declarations fail with new input spelling. | `tests/sql_exec.rs::semantic_failure_matrix_reports_exact_errors` |
| `S09` | Missing dependency table for `INSERT` and `SELECT` fails deterministically. | `tests/sql_exec.rs::semantic_failure_matrix_reports_exact_errors` |
| `S10` | Type/value boundary failures produce exact semantic errors. | `tests/sql_exec.rs::semantic_failure_matrix_reports_exact_errors` |
| `S11` | Unsupported and malformed SQL are separated and exact. | `tests/sql_exec.rs::unsupported_sql_reports_exact_statement`, `tests/sql_exec.rs::malformed_sql_reports_exact_statement` |
| `S12` | Trust boundary: arbitrary pre-SQL PageStore payload is rejected as invalid SQL storage. | `tests/sql_exec.rs::unknown_sql_storage_record_fails_deterministically` |
| `S13` | CLI command surface promotes `exec` while preserving help and unsupported command behavior. | `tests/cli_contract.rs` |

## Task Mapping

| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
|---|---|---|---|---|---|---|
| `T001` | QA mapped | Repo context inspection, protected-area check, latest docs/test read | n/a | `git rev-parse HEAD`, `git status --short`, `rg --files specs/v1-sql-parser-schema-exec` | Implementation pass starts only after confirming current HEAD, dirty state, and no `ssot/` or `policies/` edits. | QA prep observed feature spec directory untracked and no protected edits required. |
| `T002` | Red scaffold added | Black-box CLI integration tests, exact stdout/stderr/exit assertions, restart persistence, invalid storage fixture | `tests/sql_exec.rs` | `cargo test --test sql_exec` | All SQL contract tests pass for happy path, multi-select, empty table, partial stdout suppression, case-insensitive identifiers, semantic matrix, restart, mid-command failure, and unknown SQL storage record. | Current implementation has no `exec`, so this command is expected red before implementation. |
| `T003` | Red scaffold updated | Help contract assertions, unsupported command assertions, malformed `exec` arity assertion | `tests/cli_contract.rs` | `cargo test --test cli_contract` | Help lists `db exec <path> <sql>` as supported, `open` remains unsupported, `exec` with missing SQL exits `2` with existing unsupported CLI format, and `--help`/`help` match. | Current help still lists `exec` as reserved, so this command is expected red before implementation. |
| `T004` | QA mapped | Unit/integration-observable behavior through CLI tests, storage fixture for decode boundary | `tests/sql_exec.rs` | `cargo test --test sql_exec` | Parser, catalog rebuild, executor, logical record encode/decode, and typed errors satisfy every test without panics. | No application implementation is added in QA prep. |
| `T005` | QA mapped | CLI route and exit-code mapping through black-box tests | `tests/sql_exec.rs`, `tests/cli_contract.rs` | `cargo test --test sql_exec`, `cargo test --test cli_contract` | `db exec <path> <sql>` succeeds/fails with exact stdout, stderr, and exit codes while legacy unsupported CLI remains stable. | Missing-arity `exec` remains covered as unsupported input. |
| `T006` | QA mapped | Durable docs must mirror exact test strings and storage logical record contract | `tests/sql_exec.rs`, `tests/cli_contract.rs`; docs review | `cargo test --test sql_exec`, `cargo test --test cli_contract`, `./scripts/verify` | `docs/cli_contract.md`, `docs/sql_subset.md`, and `docs/file_format.md` describe the same command surface, SQL grammar, error strings, and `PDBSQL1\0` logical records asserted by tests. | Docs are not updated in QA prep except this mapping. |
| `T007` | QA mapped | Required command execution evidence | all integration tests | `cargo test --test sql_exec`, `cargo test --test cli_contract`, `./scripts/verify`, required CLI smoke command | All required commands pass after implementation; pre-implementation red evidence fails for expected missing SQL exec behavior. | QA prep runs red commands and records current failure mode below. |
| `T008` | QA mapped | Run result/report evidence | `specs/v1-sql-parser-schema-exec/qa_mapping.md`, scheduler `result.md` | inspect result file and final scheduler termination block | Result records phase status, command evidence, changed QA artifacts, and any blockers. | This phase result should be `success` once mapping, scaffold, and red evidence are complete. |

## Testing-Review Lens

- Task coverage: all `T001` through `T008` entries are mapped.
- Preferred commands: every task with executable evidence references concrete cargo/script/smoke commands.
- Task-scoped green: each entry defines the implementation condition needed to turn its QA red state green.
- Negative and boundary coverage: unsupported SQL, malformed SQL, duplicate table, missing table, duplicate column, column count mismatch, type mismatch, failed-command empty stdout, mid-command persistence, unknown SQL storage record, and malformed `exec` arity are covered.
- Flake controls: temp database paths include process id and nanosecond suffix; tests assert exact process output and clean temp dirs best-effort.
- Scope control: no application core/source implementation files are modified by QA prep.

## Red Evidence

Commands were run from the managed repo root after scaffold generation.

| Command | Expected Pre-Implementation Result | Observed Result |
|---|---|---|
| `cargo test --test sql_exec` | red | Red as expected. `0 passed; 11 failed`; failures show `db exec` currently exits `2` with `error: unsupported argument or command: exec` instead of the SQL contract outputs. |
| `cargo test --test cli_contract` | red | Red as expected. `3 passed; 2 failed`; help tests fail because current help omits `db exec <path> <sql>` from supported usage and still lists it as reserved. |

Formatting note: `cargo fmt` was run after adding QA scaffolds.
