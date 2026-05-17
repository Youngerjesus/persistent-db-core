# Implementation Plan: v1-differential-property-tests

## Goal
Add executable differential/property evidence for the supported SQL subset by generating deterministic operation sequences, running them through `db`, comparing observable results against SQLite, capturing failing seeds/prefixes, and adding a task-specific verification script.

## Non-Goals
- No user-facing CLI behavior changes.
- No SQL grammar expansion.
- No production dependency additions.
- No network service, daemon, multi-process concurrency, optimizer, benchmark, or browser evidence.
- No updates to `docs/cli_contract.md`.
- No changes to `ssot/` or `policies/`.

## Affected Contract Surface
- Tests: new black-box integration harness in `tests/differential_property.rs`.
- Verification: new `scripts/verify_differential_property`; baseline `scripts/verify` must remain green.
- Dependency metadata: add SQLite oracle as test-only `[dev-dependencies]`.
- Durable docs: create or update `docs/testing.md` for verification command, seed replay, and failure evidence location.
- CLI contract: inspect only; final report must state `docs/cli_contract.md` was not modified.

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| `tests/differential_property.rs` | Deterministic generator, SQLite oracle, `db` runner, result parser, failure capture | black-box CLI behavior; no production helper exports required |
| `Cargo.toml` | Add `rusqlite` under `[dev-dependencies]` only | no production dependency |
| `scripts/verify_differential_property` | Resolve repo root and run exact cargo test command with `--nocapture` | executable and callable from any cwd |
| `docs/testing.md` | Document command, seed replay, prefix replay, failure artifact path | no CLI contract changes |
| `docs/cli_contract.md` | No edit; final evidence confirms unchanged | stop if ordered scan contract conflicts |

## Harness Flow
1. For each default seed, create an isolated temporary directory and database path.
2. Generate one operation sequence with a forced coverage prelude and randomized deterministic tail.
3. Execute `CREATE TABLE kv (id INT PRIMARY KEY, value TEXT);` once against both systems.
4. For each operation after create:
   - Run SQLite oracle first or in lockstep for the same semantic operation.
   - Run `db exec <path> <sql>` for the equivalent SQL.
   - Compare exit class for duplicate insert errors.
   - Compare parsed row vectors for `SELECT *` and `SELECT * WHERE id = ?`.
   - Assert missing key lookup returns an empty row vector after the header.
   - Assert full scan rows are ordered by ascending `id`.
5. On mismatch, calculate the shortest failing prefix by replaying prefixes of the same generated sequence until the mismatch reproduces.
6. Write failure JSON under `target/differential_property/failures/<seed>.json`.
7. Print seed, failing operation index, minimal prefix, expected rows, actual rows, replay command, and artifact path to stdout before panic.

## Data and Operation Model
- Table: `kv(id INT PRIMARY KEY, value TEXT)`.
- Key domain: deterministic `i64` values generated from the seed, unique for successful inserts.
- Value domain: deterministic ASCII text with no unsupported SQL output characters, quotes, pipes, newlines, or carriage returns.
- Minimum per seed: at least 25 successful unique rows and at least 100 operations.
- Required operation variants:
  - successful insert
  - duplicate primary-key insert
  - missing primary-key lookup
  - existing primary-key lookup
  - full ordered scan
  - empty or early scan where useful to prove header parsing remains stable

## Verification Plan
Required commands after implementation:

```bash
./scripts/verify
./scripts/verify_differential_property
```

Additional command useful during implementation:

```bash
cargo test --test differential_property -- --nocapture
```

The implementation report must map these commands to `gate-v1-differential-property-tests` and `req-v1-differential-property-proof`.

## Acceptance Mapping
| Acceptance Item | Planned Evidence |
|---|---|
| Task-specific script exists and runs exact test command | `scripts/verify_differential_property` plus command output |
| SQLite-backed oracle compares every assertion | `tests/differential_property.rs` using `rusqlite`; cargo test output |
| SQLite dependency is test-only | `Cargo.toml` diff showing `[dev-dependencies]` only |
| Supported SQL subset covered | generated sequence and forced prelude in test source |
| 25 rows and 100 operations per seed | test assertions inside harness |
| Duplicate and missing lookup covered | forced operation variants and mismatch assertions |
| Ordered `SELECT *` by `id` | SQLite `ORDER BY id` expected rows and explicit ascending assertion |
| Failure stdout and artifact contract | failure-report helper covered by harness structure and manual review evidence |
| Baseline verification remains green | `./scripts/verify` output |
| Testing docs updated, CLI contract unchanged | `docs/testing.md` diff and final `git diff -- docs/cli_contract.md` evidence |
| Gate and requirement mapped | final implementation report cites `gate-v1-differential-property-tests` and `req-v1-differential-property-proof` |

## Risks
- `INSERT INTO kv (id, value) VALUES` syntax from the spec is semantic shorthand; current parser supports `INSERT INTO kv VALUES`. Do not add column-list syntax unless a later approved spec changes the CLI contract.
- `INTEGER` in the spec is semantic shorthand; current docs and parser use `INT`. Use the documented `INT` syntax to avoid a CLI behavior change.
- Failure minimization can be expensive if implemented by full linear replay for long sequences. Keep default operation count modest while meeting the minimum, and use deterministic isolated temp paths for each prefix replay.
- The test must avoid relying on unordered SQLite row output; use `SELECT id, value FROM kv ORDER BY id` for full-scan oracle rows.

## Stop Conditions
- Current `db` no longer guarantees `SELECT *` primary-key scan in ascending `id`.
- Implementation needs to change `docs/cli_contract.md` or production SQL behavior.
- SQLite cannot be used as a Rust test-only oracle in the environment.
- A second recovery attempt is required after verifier rejection.

