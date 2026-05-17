# Tasks: v1-differential-property-tests

## Execution Rules
- Treat `spec.md` and `contracts.md` as frozen inputs.
- Do not edit `docs/cli_contract.md`, `ssot/`, or `policies/`.
- Keep implementation scoped to `tests/differential_property.rs`, `scripts/verify_differential_property`, `docs/testing.md`, and test-only `Cargo.toml` dependency metadata unless a blocker is reported.
- Use SQLite through a Rust test-only dev-dependency. Do not use an external `sqlite3` binary.
- Run `./scripts/verify` and `./scripts/verify_differential_property` before reporting completion.

## Task List

### T1. Add SQLite dev-dependency
Status: ready

Files:
- `Cargo.toml`
- `Cargo.lock`

Details:
- Add `rusqlite` under `[dev-dependencies]` only.
- Keep `[dependencies]` unchanged.
- Let Cargo update the lockfile.

Subtasks:
- T1.1 Add the dependency in the test-only section.
- T1.2 Confirm `cargo tree --edges normal` does not show SQLite in production dependencies if needed.

Acceptance evidence:
- `Cargo.toml` diff and `./scripts/verify` output.

### T2. Create deterministic differential/property harness
Status: ready

Files:
- `tests/differential_property.rs`

Details:
- Add a deterministic PRNG and operation generator.
- Default seeds must each produce at least 100 operations and at least 25 successful unique rows.
- Include forced operations for create table, successful insert, duplicate insert, missing lookup, existing lookup, and ordered full scan.
- Run `db` through `env!("CARGO_BIN_EXE_db")` with one `db exec` per operation.
- Use fresh temp database paths per seed and per replay attempt.

Subtasks:
- T2.1 Define `Operation`, `Row`, seed suite, and replay options.
- T2.2 Implement deterministic ASCII value generation that avoids unsupported text characters.
- T2.3 Implement sequence generation with coverage assertions.
- T2.4 Implement `db` runner and stdout row parser.

Acceptance evidence:
- `cargo test --test differential_property -- --nocapture` executes the generated suite.

### T3. Implement SQLite oracle comparisons
Status: ready

Files:
- `tests/differential_property.rs`

Details:
- Use in-memory `rusqlite::Connection` per replay.
- Use parameterized insert and lookup statements for SQLite.
- For full scans, compare against `SELECT id, value FROM kv ORDER BY id`.
- For duplicate inserts, assert SQLite errors and `db` errors; compare error class instead of SQLite wording.
- For missing lookups, assert both sides produce an empty row vector.

Subtasks:
- T3.1 Build SQLite schema and operation executor.
- T3.2 Compare successful select rows against `db` parsed rows.
- T3.3 Compare duplicate insert error class.
- T3.4 Assert ordered full scan rows are ascending by `id`.

Acceptance evidence:
- The differential test source and passing `cargo test --test differential_property -- --nocapture`.

### T4. Add failure capture and replay support
Status: ready

Files:
- `tests/differential_property.rs`

Details:
- On mismatch, compute shortest reproducible operation prefix for the same seed.
- Print seed, failing operation index, minimal operation prefix, SQLite expected rows, `db` actual rows, replay command, and artifact path to stdout.
- Write failure JSON to `target/differential_property/failures/<seed>.json`.
- Support seed and prefix replay through documented environment variables.

Subtasks:
- T4.1 Define mismatch/failure report structs.
- T4.2 Implement prefix replay minimization.
- T4.3 Implement JSON escaping/writing without adding unnecessary dependencies.
- T4.4 Ensure panic message does not hide the required stdout block under `--nocapture`.

Acceptance evidence:
- Manual review of failure-report code plus passing normal test command. If a deliberate local failure is used during development, do not preserve generated failure artifacts.

### T5. Add task-specific verification script
Status: ready

Files:
- `scripts/verify_differential_property`

Details:
- Resolve repo root from script path.
- `cd` to repo root.
- Run exactly `cargo test --test differential_property -- --nocapture`.
- Mark the script executable.
- Verify the script works from a cwd outside the repo.

Subtasks:
- T5.1 Add script content modeled after `scripts/verify`.
- T5.2 Run from repo root.
- T5.3 Run by absolute path from a temporary outside cwd if needed for evidence.

Acceptance evidence:
- `./scripts/verify_differential_property` output and outside-cwd invocation evidence.

### T6. Document verification workflow
Status: ready

Files:
- `docs/testing.md`

Details:
- Document `./scripts/verify_differential_property`.
- Document seed replay command.
- Document prefix replay command if supported.
- Document failure artifact path `target/differential_property/failures/<seed>.json`.
- State generated failure artifacts are local evidence and not durable SSOT.

Subtasks:
- T6.1 Create `docs/testing.md` if absent.
- T6.2 Keep the doc short and focused on test workflow.
- T6.3 Confirm `docs/cli_contract.md` remains unchanged.

Acceptance evidence:
- Documentation diff and final unchanged CLI contract note.

### T7. Verify and report acceptance mapping
Status: ready

Files:
- implementation result/report path owned by implementation phase

Details:
- Run:
  - `./scripts/verify`
  - `./scripts/verify_differential_property`
- Include stdout/stderr summaries in the final implementation report.
- Map evidence explicitly to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`.
- Include confirmation that `docs/cli_contract.md` was not modified.

Subtasks:
- T7.1 Capture required command results.
- T7.2 If a command fails, repair once in phase.
- T7.3 Escalate if a second recovery attempt is required.

Acceptance evidence:
- Final report with command summaries, artifact delta, and gate/requirement mapping.

## Dependency Order
1. T1
2. T2
3. T3
4. T4
5. T5
6. T6
7. T7

## Readiness Notes
- No human decision is required before implementation.
- The only approved new dependency is a test-only SQLite oracle dependency.
- The highest-risk implementation details are failure minimization correctness and avoiding accidental CLI syntax expansion.

