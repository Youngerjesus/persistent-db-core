# Implementation Brake Review: v1-sql-parser-schema-exec

## Verdict: PASS

Latest outcome: `success`

Fresh Repair Required: no

The implementation is ready to enter strict `impl_verify`. The prior verifier-blocking documentation parity finding, `IBR-007`, is resolved: `docs/cli_contract.md` now documents the duplicate table and duplicate column case-variant target-spelling notes that were already present in `docs/sql_subset.md`.

This phase remains review-only; no production code, tests, or product docs were repaired here.

## Scope

- Phase: `impl_brake_exec`
- Review mode: read-only implementation-brake current-state audit and recent-diff review.
- Inputs reviewed: approved `spec.md`, `contracts.md`, `qa_mapping.md`, prior `impl_brake_review.md`, failed strict verifier report `impl_review.md`, latest implementation retry result `impl_retry_1_resume_20260517_211110_227549_fdb851ff/result.md`, current diff and untracked task files, `src/main.rs`, `src/lib.rs`, `src/sql.rs`, `tests/sql_exec.rs`, `tests/cli_contract.rs`, `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md`.
- Companion reviewers:
  - `implementation-brake-reviewer`: completed. It found one proposed verify-blocking issue for untracked task files and two verify-risk issues. The untracked-file issue is reconciled as `IBR-009` verify-risk rather than verify-blocking because strict `impl_verify` is executable against the current worktree and the required commands include the untracked files; closeout/merge must still include them. Its storage-corruption concern is accepted as `IBR-008` verify-risk. Its parser-classification concern is the existing `IBR-005`.
  - `code-reviewer`: completed. It found no verify-blocking issue, confirmed the case-variant CLI doc parity repair, and carried parser classification plus untracked task files forward as verify risks.
- Commands run from repo root:
  - `cargo test --test sql_exec`: pass, 18 tests.
  - `cargo test --test cli_contract`: pass, 5 tests.
  - `./scripts/verify`: pass.
  - Required CLI smoke command with explicit capture: exit `0`, stdout `id|name\n1|ada\n2|bea\n`, stderr empty.

## Finding Checklist

- [resolved] `IBR-001` - kind: missing behavior; risk category: correctness, edge/failure path, test gap; severity: `verify-blocking`; source attempt: `impl_brake_exec_fresh_20260517_201501_213784_07a8bb88` plus `code-reviewer` companion.
  - Evidence: prior implementation rejected spec-valid signed decimal spelling such as `-0` as malformed SQL.
  - Repair target: update `INT` literal acceptance to match signed 64-bit decimal grammar and add focused black-box coverage.
  - Closure evidence: `tests/sql_exec.rs::signed_decimal_int_literals_accept_noncanonical_zero_spelling`; this brake pass reran `cargo test --test sql_exec`, `cargo test --test cli_contract`, and `./scripts/verify`, all passing.

- [resolved] `IBR-002` - kind: verification gap; risk category: persisted-data compatibility, documentation precision; severity: `verify-risk`; source attempt: `impl_brake_exec_fresh_20260517_201501_213784_07a8bb88` plus `code-reviewer` companion.
  - Evidence: prior report noted row-value encoding documentation could be clearer for persisted-data compatibility review.
  - Repair target: clarify whether `INT` values are encoded as canonical decimal UTF-8 bytes and `TEXT` values as literal text bytes.
  - Closure evidence: `docs/sql_subset.md` documents canonical decimal UTF-8 bytes for `INT` row values and literal text bytes for `TEXT`.

- [resolved] `IBR-003` - kind: behavior defect; risk category: correctness, edge/failure path, persisted-data compatibility; severity: `verify-blocking`; source attempt: `impl_brake_exec_fresh_20260517_202332_158924_31be8a4d` plus `implementation-brake-reviewer` companion.
  - Evidence: prior implementation accepted SQL-prefixed logical records that violated SQL catalog/value invariants and then returned successful `SELECT` output.
  - Repair target: reject SQL-prefixed catalog and row records that violate SQL-layer invariants during load with the documented invalid SQL storage record error, and add restart/load-path tests for duplicate-column catalog data and output-breaking `TEXT` row data.
  - Closure evidence: `src/sql.rs` validates loaded catalog identifiers, nonempty schema, duplicate columns, duplicate tables, row table existence, row arity/type, canonical persisted `INT` spelling, and output-safe text values. `tests/sql_exec.rs` includes SQL-prefixed invalid-record regressions. Required checks pass in this brake pass.

- [resolved] `IBR-004` - kind: behavior defect; risk category: correctness, edge/failure path, test gap; severity: `verify-blocking`; source attempt: `impl_brake_exec_fresh_20260517_202332_158924_31be8a4d`, re-opened in `impl_brake_exec_fresh_20260517_204038_507969_5b231316` after `code-reviewer` companion.
  - Evidence: prior manual repro showed `db exec <tmp> "SELECT * FROM users extra;"` exited `2` with `error: unsupported SQL statement: SELECT * FROM users extra;`, despite the malformed supported-shape contract.
  - Repair target: add an exact `SELECT * FROM users extra;` malformed-SQL regression and refine `SELECT` parse classification so broken supported `SELECT * FROM <table_name>;` shapes return `SqlError::Malformed`, while documented out-of-scope variants such as projection and `WHERE` remain unsupported.
  - Closure evidence: `tests/sql_exec.rs::malformed_select_trailing_token_reports_exact_statement` covers the contract. Prior brake pass manually reproed `SELECT * FROM users extra;` and observed exit `2`, empty stdout, and malformed-SQL stderr; current `cargo test --test sql_exec` remains green.

- [open] `IBR-005` - kind: verification gap; risk category: edge/failure path, test gap; severity: `verify-risk`; source attempt: `impl_brake_exec_fresh_20260517_205009_759666_caad4e24` plus current `code-reviewer` and `implementation-brake-reviewer` companions.
  - Evidence: companion/manual probes showed `INSERT users VALUES (1);`, `CREATE users (id INT);`, and `SELECT FROM users;` are classified as unsupported SQL. Contract text says inputs where the supported statement shape is broken are malformed SQL, but the explicit examples and required matrix do not define these missing-keyword/missing-projection variants.
  - Verifier question: should strict `impl_verify` interpret "broken supported statement shape" broadly enough to require these additional SQL-like cases to be malformed, or treat them as unsupported because they do not match the documented supported command prefix forms?
  - Why not verify-blocking: the current implementation satisfies the explicit malformed examples, prior retry target, and required tests. The previous strict verifier already judged the broader parser-boundary risk acceptable. This remains an interpretive verifier question rather than a direct brake-level contradiction.
  - Repair target if verifier accepts the broader interpretation: add exact malformed tests for missing-keyword/missing-projection shapes and refine top-level parser classification for `CREATE`, `INSERT`, and `SELECT`.
  - Closure evidence: pending verifier judgment.

- [resolved] `IBR-006` - kind: behavior defect; risk category: correctness, persisted-data compatibility, edge/failure path, test gap; severity: `verify-blocking`; source attempt: `impl_brake_exec_fresh_20260517_205009_759666_caad4e24` plus `implementation-brake-reviewer` companion.
  - Evidence: prior implementation accepted a valid SQL catalog record plus SQL-prefixed row record with `INT` bytes `01`, returning successful `SELECT` output `id\n1\n` and exit `0`, even though `docs/sql_subset.md` defines row `INT` payload bytes as canonical decimal UTF-8.
  - Repair target: reject noncanonical persisted `INT` bytes during SQL logical-record decode/load and add a focused fixture-backed integration test for `01` or equivalent noncanonical persisted integer bytes.
  - Closure evidence: latest retry result reports a red repro before repair, then green checks. Current code rejects noncanonical persisted `INT` bytes by requiring `raw_value == parsed.to_string()` during row decode in `src/sql.rs`. `tests/sql_exec.rs::sql_prefixed_noncanonical_int_row_record_fails_deterministically` appends a SQL-prefixed row fixture with `INT` bytes `01` and expects the documented invalid SQL storage record error. This brake pass reran the required checks successfully.

- [resolved] `IBR-007` - kind: verification gap; risk category: documentation contract, completeness; severity: `verify-blocking` after strict verifier `IMPL-VERIFY-001`; source attempt: `impl_brake_exec_fresh_20260517_210010_973356_78475177`, strict `impl_verify_1_fresh_20260517_210742_023746_6ff59960`, latest retry `impl_retry_1_resume_20260517_211110_227549_fdb851ff`.
  - Evidence: strict verifier found `docs/cli_contract.md` did not document duplicate table and duplicate column case-variant target-spelling rules required by `contracts.md`, while `docs/sql_subset.md` did.
  - Repair target: update `docs/cli_contract.md` under SQL semantic errors to include duplicate table `Users` and duplicate column `ID` target-spelling notes, matching the contract and `docs/sql_subset.md`.
  - Closure evidence: current `docs/cli_contract.md` includes both case-variant notes. Latest retry result reports the doc repair plus green `cargo test --test sql_exec`, `cargo test --test cli_contract`, `./scripts/verify`, and CLI smoke. This brake pass reran all required commands successfully.

- [open] `IBR-008` - kind: verification gap; risk category: edge/failure path, documentation contract, regression; severity: `verify-risk`; source attempt: current `implementation-brake-reviewer` companion.
  - Evidence: `main.rs` maps `SqlError::Storage(error)` to `error: storage error: {error:?}` and `hint: database file must use the documented V1 page format.` The SQL tests cover unknown SQL logical records, but do not pin a page-level corruption fixture through `db exec`.
  - Verifier question: is the existing page-storage test suite plus SQL unknown-record fixture enough for this task, or should `impl_verify` require a black-box `db exec` test for representative page corruption because the SQL layer exposes storage errors through user-facing stderr?
  - Why not verify-blocking: the approved task explicitly requires the unknown SQL storage record negative path and allows page-level corruption to retain existing `StorageError` behavior. Baseline `tests/page_storage.rs` and `./scripts/verify` pass, and no direct runtime contradiction was observed.
  - Repair target if verifier requires this coverage: add a CLI integration test for a representative corrupt/truncated page file and document or stabilize the exact storage-error stderr if it is treated as a user-facing contract.
  - Closure evidence: pending verifier judgment.

- [open] `IBR-009` - kind: verification gap; risk category: evidence provenance, merge safety; severity: `verify-risk`; source attempt: current `code-reviewer` and `implementation-brake-reviewer` companions.
  - Evidence: `git status --short --branch` shows task-critical untracked files including `src/sql.rs`, `tests/sql_exec.rs`, `docs/sql_subset.md`, and task-scoped spec artifacts. The current worktree verifies green because those files exist locally.
  - Verifier question: confirm strict `impl_verify` is evaluating the full current worktree, including untracked task files, and that closeout/merge will include every task-critical file in the submitted delta.
  - Why not verify-blocking: this brake phase and required commands ran against the full current worktree, so strict `impl_verify` remains executable. This is a merge-provenance risk for closeout, not a code repair target for `impl_retry`.
  - Repair target if closeout finds missing files: include all task-critical new files in the submitted delta and rerun required verification against that exact state.
  - Closure evidence: pending closeout/merge packaging.

## Must Fix Now

- None.

## Verify Risks

- `IBR-005`: Broader malformed-vs-unsupported classifier boundary. Strict verifier should decide whether SQL-like missing-keyword shapes must be malformed or whether the documented unsupported contract can cover them.
- `IBR-008`: SQL CLI storage-corruption surface is not pinned by a SQL black-box corruption fixture; verifier should decide whether existing page-storage tests plus unknown SQL logical-record fixture satisfy the task.
- `IBR-009`: New task-critical files remain untracked in git status; verifier/closeout should ensure the final submitted delta includes them.

## Blocked On Evidence

- None.

## Blocked On Human Decision

- None.

## Repair Targets

- None for `impl_retry`.
- If strict `impl_verify` accepts `IBR-005` or `IBR-008` as contract failures, repair should target the specific verifier finding.
- If closeout finds `IBR-009` unresolved in the submitted delta, include the missing files and rerun required verification.

## Closure Evidence

- `IBR-001` closed by signed decimal `-0` regression and green required checks.
- `IBR-002` closed by SQL logical row-value encoding docs.
- `IBR-003` closed by invalid SQL-prefixed record validation and tests.
- `IBR-004` closed by `tests/sql_exec.rs::malformed_select_trailing_token_reports_exact_statement`, manual repro in a prior brake pass, and green required checks.
- `IBR-006` closed by `tests/sql_exec.rs::sql_prefixed_noncanonical_int_row_record_fails_deterministically`, row decode canonical-byte validation, and green required checks.
- `IBR-007` closed by `docs/cli_contract.md` case-variant duplicate table/column notes plus green required checks.
- Current command evidence:
  - `cargo test --test sql_exec`: pass, 18 tests.
  - `cargo test --test cli_contract`: pass, 5 tests.
  - `./scripts/verify`: pass.
  - Required CLI smoke with explicit capture: exit `0`, stdout `id|name\n1|ada\n2|bea\n`, stderr empty.

## Residual Risks

- The broad malformed classifier boundary in `IBR-005` remains a verifier question.
- The page-corruption CLI coverage question in `IBR-008` remains a verifier question.
- The untracked-file packaging risk in `IBR-009` remains for verifier/closeout attention.
- This phase did not perform production repair by design.

## Next Action

Proceed to strict `impl_verify`. No open verify-blocking implementation-brake finding remains.

## Updated At

2026-05-17T21:19:01+0900
