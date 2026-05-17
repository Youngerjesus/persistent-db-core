# Implementation Brake Review: v1-differential-property-tests

Verdict: PASS

Updated At: 2026-05-17T20:39:06Z

## Scope

- Phase: `impl_brake_exec`
- Current brake run: `impl_brake_exec_fresh_20260518_053519_929101_6b59a4ff`
- Gate: `gate-v1-differential-property-tests`
- Requirement: `req-v1-differential-property-proof`
- Reviewed inputs: `spec.md`, `contracts.md`, `qa_mapping.md`, current worktree delta, prior implementation-brake report, and latest implementation retry result `runs/impl_retry_1_resume_20260518_053319_723988_a0882161/final.md`.
- Review posture: verify-readiness pressure review only. This phase did not repair code, tests, production files, durable docs, or implementation-owned task reports.
- Commands run in this brake pass:
  - `./scripts/verify_differential_property`: passed, 1 test.
  - `(cd /tmp && <repo>/scripts/verify_differential_property)`: passed, 1 test.
  - `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture`: passed, 1 test.
  - `cargo tree --edges normal`: showed only the root crate in the normal dependency tree.
  - `git diff -- docs/cli_contract.md`: empty.
  - `ls -l scripts/verify_differential_property`: script is executable.
  - `./scripts/verify`: passed, including fmt, clippy, full test suite, and `db --help` smoke.
- Companion review reconciliation:
  - `implementation-brake-reviewer` companion was invoked as a read-only input, but did not return within the review wait window. Fallback lens was applied by the main brake reviewer and recorded under Residual Risks.
  - No companion finding was available to accept or reject in this run.

## Finding Checklist

- ID: `IB-001`
  - Status: `superseded`
  - Kind: `verification gap`
  - Risk category: `evidence provenance`
  - Source attempt: previous implementation-brake companion `implementation-brake-reviewer`
  - Evidence: The initial implementation result was only `success` plus `PM_PHASE_COMPLETE: yes`, while `contracts.md` and `tasks.md` require command summaries, explicit `gate-v1-differential-property-tests` / `req-v1-differential-property-proof` mapping, and a `docs/cli_contract.md` unchanged note.
  - Repair target: Regenerate the implementation result/report with required evidence mapping.
  - Closure evidence: Superseded by `IB-004`, which tracked the same provenance requirement against the first retry report.

- ID: `IB-002`
  - Status: `resolved`
  - Kind: `verification gap`
  - Risk category: `correctness, test gap`
  - Source attempt: previous implementation-brake companion `code-reviewer`, accepted by main brake reviewer
  - Evidence: Prior `tests/differential_property.rs` modeled failures as generic errors, allowing duplicate primary-key checks to accept any error.
  - Repair target: Preserve a normalized error kind for duplicate primary-key operations on both SQLite and `db`; assert the duplicate-key class specifically instead of accepting any error.
  - Closure evidence: Current `tests/differential_property.rs` maps SQLite constraint failures and `db` duplicate stderr to `DuplicatePrimaryKey` and asserts duplicate inserts are `DuplicatePrimaryKeyError` on both sides. `./scripts/verify_differential_property` and `./scripts/verify` passed in this brake pass.

- ID: `IB-003`
  - Status: `resolved`
  - Kind: `verification gap`
  - Risk category: `edge/failure path, evidence provenance`
  - Source attempt: previous implementation-brake companion `code-reviewer`, accepted by main brake reviewer
  - Evidence: Prior `minimal_failing_prefix` accepted the first prefix where replay returned any error, regardless of whether it reproduced the same mismatch category.
  - Repair target: Return structured mismatch details from replay and accept a shorter prefix only when it reproduces the same failure category.
  - Closure evidence: Current `minimal_failing_prefix` now requires matching operation identity plus expected and actual observations, which is stricter than category matching. `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture` passed in this brake pass.

- ID: `IB-004`
  - Status: `resolved`
  - Kind: `verification gap`
  - Risk category: `evidence provenance`
  - Source attempt: previous implementation-brake companion `implementation-brake-reviewer`, accepted by main brake reviewer
  - Evidence: Earlier implementation retry report `runs/impl_retry_1_resume_20260518_051747_440769_bce0991f/final.md` listed passed commands and `docs/cli_contract.md` unchanged, but did not explicitly record the artifact delta or map evidence to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`.
  - Repair target: Refresh the implementation-owned result/report with the required artifact inventory, command summaries, unchanged `docs/cli_contract.md` confirmation, and explicit gate/requirement mapping.
  - Closure evidence: Later implementation retry reports record artifact delta, command evidence, unchanged `docs/cli_contract.md`, and explicit mapping to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`.

- ID: `IB-005`
  - Status: `resolved`
  - Kind: `verification gap`
  - Risk category: `evidence provenance, edge/failure path`
  - Source attempt: previous implementation-brake companion `code-reviewer`, accepted by main brake reviewer
  - Evidence: Prior `tests/differential_property.rs` defined `MismatchSignature` from only `operation_class`, `expected_kind`, and `actual_kind`; `minimal_failing_prefix` could accept a shorter prefix that reproduced the same coarse class but not the same expected/actual payload.
  - Repair target: Make failure-prefix minimization require replay equality against the original mismatch payload before accepting a shorter prefix, including operation identity and expected/actual observations, or otherwise ensure the printed/artifact expected and actual rows are from the accepted minimal prefix.
  - Closure evidence: Current `tests/differential_property.rs` requires `replayed.operation == mismatch.operation`, `replayed.expected == mismatch.expected`, and `replayed.actual == mismatch.actual` before accepting a prefix, and `report_failure` / `write_failure_artifact` use the accepted reproduced mismatch. `./scripts/verify_differential_property`, outside-cwd script execution, replay-prefix test, and `./scripts/verify` passed in this brake pass.

## Must Fix Now

- None. No open `verify-blocking` finding remains.

## Verify Risks

- `VR-001`: `impl_verify` should independently record final command evidence and gate/requirement mapping in verifier-owned outputs. This is not verify-blocking because implementation-owned evidence and this brake pass evidence are current and executable.
- `VR-002`: Failure-reporting stdout/artifact behavior remains primarily source-reviewed on the green path. The minimizer blocker is resolved, but `impl_verify` should decide whether normal green evidence plus code review is sufficient or whether to induce a controlled local mismatch without preserving generated artifacts.
- `VR-003`: New files are currently untracked in the dirty worktree (`docs/testing.md`, `scripts/verify_differential_property`, `tests/differential_property.rs`, and the task spec directory). This is a merge-safety risk for closeout, but it is not verify-blocking because `impl_verify` can execute against the current worktree and no commit is being created in this phase.
- `VR-004`: The approved task text names SQLite semantics with `CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)` and `INSERT INTO kv (id, value) VALUES (?, ?)`, while the `db` side uses the currently documented SQL subset syntax `INT PRIMARY KEY` and `INSERT INTO kv VALUES (...)`. This appears consistent with the no-CLI-expansion scope and semantic-oracle intent, but `impl_verify` should explicitly judge sufficiency.
- `VR-005`: The requested companion reviewer timed out without findings. The main brake reviewer applied the implementation-brake lens directly, so this is not verify-blocking, but independent `impl_verify` should not treat companion silence as positive evidence.

## Blocked On Evidence

- None.

## Blocked On Human Decision

- None.

## Repair Targets

- None for `impl_retry`.

## Closure Evidence

- `tests/differential_property.rs` implements deterministic seeds, generator coverage assertions, SQLite-backed expected observations, one `db` process per operation, duplicate primary-key error-class checks, missing-key lookup comparison, and ordered scan comparison.
- `tests/differential_property.rs` failure reporting prints seed, failing operation index, reproduced operation sequence, SQLite expected observation, `db` actual observation, artifact path, and replay command; failure artifacts are written under `target/differential_property/failures/<seed>.json`.
- `scripts/verify_differential_property` resolves the repo root from its own path, changes there, and runs `cargo test --test differential_property -- --nocapture`; executable bit is present.
- `Cargo.toml` adds `rusqlite` only under `[dev-dependencies]`; `cargo tree --edges normal` showed no production dependency expansion.
- `docs/testing.md` documents `./scripts/verify_differential_property`, seed/prefix replay, failure artifact location, and local generated evidence status.
- `docs/cli_contract.md` has no diff.
- Required commands passed in this brake pass:
  - `./scripts/verify`
  - `./scripts/verify_differential_property`
  - `(cd /tmp && <repo>/scripts/verify_differential_property)`
  - `PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture`

## Residual Risks

- Performance review was not invoked. The delta is test-only, the default suite runtime remained short, and no production query/runtime path changed.
- Companion reviewer did not return before the timeout. Main review applied the same verify-readiness lens and found no open verify-blocking issue.
- This brake pass is not final acceptance proof. It only determines that the implementation is ready for independent `impl_verify`.

## Next Action

Proceed to strict `impl_verify`. Current evidence maps the implementation to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`; verifier should regenerate and own final acceptance evidence.
