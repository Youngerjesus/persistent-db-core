# QA Mapping: v1-differential-property-tests

## Scope
- Phase: `qa_prep_exec`
- Current run: `qa_prep_exec_fresh_20260518_050147_541335_140c5fcc`
- Gate: `gate-v1-differential-property-tests`
- Requirement: `req-v1-differential-property-proof`
- Canonical inputs: `spec.md`, `contracts.md`, `plan.md`, `design.md`, `research.md`, `tasks.md`

## Provenance Contract
- Evidence root: `target/differential_property/`
- Required artifact list:
  - Current-run command evidence for `./scripts/verify`
  - Current-run command evidence for `./scripts/verify_differential_property`
  - Failure artifacts, when mismatches occur: `target/differential_property/failures/<seed>.json`
  - Final implementation report mapping evidence to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`
- Scenario ids / evidence ids:
  - `DP-SQLITE-ORACLE`: SQLite-backed oracle compares generated SQL behavior.
  - `DP-GENERATOR-COVERAGE`: deterministic seed suite produces at least 100 operations and 25 successful unique rows per seed.
  - `DP-DUPLICATE-ERROR`: duplicate primary-key inserts error in both SQLite and `db`.
  - `DP-MISSING-LOOKUP`: missing primary-key lookup returns an empty row set.
  - `DP-ORDERED-SCAN`: `SELECT *` rows match SQLite `ORDER BY id`.
  - `DP-FAILURE-CAPTURE`: failures print seed, operation index, minimal prefix, expected rows, actual rows, replay command, and artifact path.
  - `DP-OUTSIDE-CWD`: task-specific verification script resolves repo root from any caller cwd.
- Current-run id source: task metadata `active_run_id` and this QA mapping header.
- Clean generation rule: canonical launch evidence for a fresh repair or verification pass must be deleted, replaced, or regenerated from the current run. Historical artifacts may remain only as audit evidence and must not be reused as current proof.
- No artifact reuse rule: implementation and verification must not cite stale `target/differential_property/failures/*.json`, previous terminal logs, or prior run reports as current proof.
- Writer/validator separation expectation: implementation writes the harness, docs, script, dependency delta, and generated failure artifacts; verifier independently runs required commands and validates this mapping against current output.
- Redaction target list: no secrets are expected; redact local absolute temp paths from durable reports unless needed for reproducing a current-run failure, and never include environment secrets if future local environments add them.

## Scenario Expansion Lens
| Scenario ID | Pressure Path | QA Expectation |
|---|---|---|
| `DP-HAPPY` | Fresh database, create table, insert unique rows, select all, select by existing id | SQLite expected rows exactly equal parsed `db` rows. |
| `DP-EMPTY-PARTIAL` | Early scan after create and before all inserts, missing lookup against valid table | Header-only or empty row vectors are accepted only when SQLite also returns empty rows. |
| `DP-DUPLICATE` | Duplicate primary key after at least one successful insert | SQLite and `db` both error; compare error class, not exact wording. |
| `DP-MISSING` | Lookup for deterministic key never inserted | Both systems return empty rows and `db` exits successfully. |
| `DP-ORDERING` | Full scan after mixed positive/negative i64 keys | Rows must be ascending by `id`, matching SQLite `ORDER BY id`. |
| `DP-RESTART` | One `db exec` process per operation | Durability and reopen behavior are naturally exercised between operations. |
| `DP-RETRY-REPLAY` | Re-run with same seed and prefix after mismatch | Same prefix must reproduce the failure category using fresh DB paths and fresh SQLite connections. |
| `DP-DEPENDENCY` | SQLite oracle dependency unavailable or added as production dependency | Report blocker or fail review; do not replace with an in-memory oracle. |
| `DP-TRUST-BOUNDARY` | External `sqlite3` binary unavailable | Test must still run because oracle is Rust `rusqlite` dev-dependency only. |

## Task Mapping
| Task ID | Status | Verification Layers | Test Files | Preferred Commands | Task-Scoped Green | Notes |
|---|---|---|---|---|---|---|
| T1 | QA mapped | Dependency metadata review; production dependency boundary check; baseline verify | `Cargo.toml`, `Cargo.lock` | `cargo tree --edges normal`; `./scripts/verify` | `rusqlite` appears only under `[dev-dependencies]`; `[dependencies]` remains free of SQLite; baseline verify passes. | Negative path: production dependency addition fails review. |
| T2 | Red scaffolded | Integration test scaffold; deterministic generator contract; CLI process runner contract | `tests/differential_property.rs` | `cargo test --test differential_property -- --nocapture` | Default seeds generate at least 100 operations and 25 successful unique rows per seed; forced coverage includes create, successful insert, duplicate insert, missing lookup, existing lookup, and ordered scan; `db` is invoked via `env!("CARGO_BIN_EXE_db")` once per operation. | Current QA scaffold intentionally fails until implementation replaces it. |
| T3 | Red scaffolded | SQLite oracle comparison; row parser; ordered scan assertion; duplicate error class comparison | `tests/differential_property.rs` | `cargo test --test differential_property -- --nocapture` | Every select assertion compares parsed `db` rows to SQLite rows; duplicate insert errors on both sides; missing lookup returns empty rows on both sides; full scans assert ascending `id`. | Must use SQLite as oracle, not generator bookkeeping. |
| T4 | Red scaffolded | Failure reporting contract; replay minimization; generated artifact contract | `tests/differential_property.rs`; `target/differential_property/failures/<seed>.json` when failing | `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture` | Failure stdout includes seed, failing operation index, minimal operation prefix, SQLite expected rows, `db` actual rows, replay command, and artifact path; JSON artifact is written only as local generated evidence. | Failure artifacts are not durable SSOT and must not be reused across current-run verification. |
| T5 | QA mapped | Script review; outside-cwd execution | `scripts/verify_differential_property` | `./scripts/verify_differential_property`; `(cd /tmp && <repo>/scripts/verify_differential_property)` | Script resolves repo root from its own path, changes to repo root, and runs exactly `cargo test --test differential_property -- --nocapture`; executable bit is set. | Script is not created in QA prep; implementation owns it. |
| T6 | QA mapped | Documentation review; CLI contract non-change check | `docs/testing.md`; `docs/cli_contract.md` | `git diff -- docs/testing.md docs/cli_contract.md`; `./scripts/verify` | Testing docs mention task-specific script, seed replay, prefix replay, failure evidence location, and generated-local-evidence status; `docs/cli_contract.md` remains unchanged. | CLI contract already documents primary-key scan ordering, so no conflict found in QA prep. |
| T7 | QA mapped | Final command evidence; gate and requirement mapping; dirty diff review | Implementation run report | `./scripts/verify`; `./scripts/verify_differential_property`; `git diff -- docs/cli_contract.md` | Final report includes command summaries, artifact delta, explicit `gate-v1-differential-property-tests` and `req-v1-differential-property-proof` mapping, and confirms CLI contract was not modified. | Second recovery attempt after verifier rejection must escalate per contract. |

## Red Evidence
- Red scaffold command: `cargo test --test differential_property -- --nocapture`
- Observed QA-prep result: failed with exit code 101 after running 1 scaffold test, `differential_property_harness_contract_is_not_yet_implemented`.
- Expected QA-prep failure reason: `tests/differential_property.rs` is a pre-implementation scaffold and intentionally panics with the missing implementation contract for `req-v1-differential-property-proof`.
- Implementation must turn this red scaffold green by satisfying T1-T7 without weakening the assertions.

## Testing-Review Lens
- All Task IDs T1 through T7 are covered.
- Preferred commands are concrete and runnable after implementation creates the script and dependency delta.
- Task-scoped green definitions are specific and tied to observable output, diffs, or generated artifacts.
- Negative and boundary paths covered: duplicate primary key, missing lookup, ordered scan with signed `i64` keys, replay prefix, dependency boundary, outside-cwd script execution, and no CLI contract edits.
- Evidence-heavy provenance is defined before implementation; current-run evidence must be regenerated, not reused.
