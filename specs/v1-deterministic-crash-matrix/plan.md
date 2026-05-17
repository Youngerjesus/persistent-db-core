# Implementation Plan: v1-deterministic-crash-matrix

## Goal
Add deterministic crash matrix evidence for WAL write, commit, corrupt/incomplete tail, and recovery replay boundaries without changing the public `db` CLI contract.

## Non-Goals
- Public crash injection, transaction, rollback, checkpoint, repair, or WAL inspection commands.
- Networked database behavior, multi-process concurrency, distributed storage, or query optimizer changes.
- Broad storage redesign or dependency additions.
- Edits to protected `ssot/` or `policies/`.

## Affected Contract Surface
- CLI behavior: `db exec <path> <sql>` must keep existing stdout, stderr, and exit code behavior.
- Persisted data compatibility: existing page files without WAL and with retained committed WAL must still open.
- WAL/file format docs: `docs/file_format.md` must include the crash matrix compatibility note or be cited as already sufficient.
- Tests and verification: add `tests/crash_matrix.rs`, `tests/fixtures/crash_matrix/`, and `scripts/verify_crash_matrix`.
- Evidence: generate `target/crash_matrix/crash_matrix_report.md` with all CM-001..CM-006 fields.

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| `tests/crash_matrix.rs` | Add six deterministic matrix cases and fixture helpers | each failure message includes `case_id` and crash point |
| `tests/fixtures/crash_matrix/` | Add required tracked fixture manifest/README or named seed descriptors for `seed_committed_one` and CM-001..CM-006, even when bytes are code-generated | no machine-specific paths or generated runtime logs; this directory is unconditional contract evidence |
| `src/storage.rs` or adjacent private module | Add minimal crash/recovery injection only if needed for CM-005 | no public CLI surface or behavior change |
| `scripts/verify_crash_matrix` | Run matrix test and write report | must work from repo root and should resolve repo root like `scripts/verify` |
| `docs/file_format.md` | Add/confirm WAL sidecar crash compatibility note | document incomplete tail cleanup and corrupt-tail handling used by CM-002/CM-006 |
| `docs/cli_contract.md` | Prefer no change | update only if user-facing output/error behavior changes |

## Crash Matrix Execution Model
1. Build or run the `db` binary through Cargo integration test support.
2. For each case, create a unique temp database path.
3. Seed `items` with deterministic schema and `(1, 'seed')`.
4. Place the database and WAL sidecar in the exact crash-point state by either CLI writes, direct WAL bytes, or a minimal non-public test hook.
5. Reopen through `db exec <path> "SELECT * FROM items;"` or a current-SQL equivalent deterministic query.
6. Compare exact stdout row order, stderr, exit status, WAL/file-format assertion, and repeat reopen where required.
7. Capture observed per-case results in `target/crash_matrix/crash_matrix_report.md` through `scripts/verify_crash_matrix`; report rows must be derived from the executed matrix run, including actual visible rows and command exit status.

## Case Plan
| Case | Setup Strategy | Expected Verification |
|---|---|---|
| CM-001 | Seed committed row through CLI; simulate interruption before WAL append by doing no WAL/data mutation for row 2 | stdout only contains `1|seed`; WAL absent or empty accepted; data file header/version unchanged |
| CM-002 | Seed row; append short WAL header or short payload for row 2 | reopen exits 0, seed row only, incomplete sidecar tail ignored/truncated without panic |
| CM-003 | Seed row; write a complete current-format page-append WAL frame for row 2 with state byte `0x02` (`WAL_STATE_ROLLED_BACK`) instead of committed state `0x01`. This is the current WAL-format mapping for "commit marker absent" because V1 has no separate post-frame commit marker. | seed row only; existing `wal_recovery` uncommitted regression remains intact; report documents the state-byte mapping |
| CM-004 | Seed row; write committed WAL frame for row 2 without applying data file | first and second reopen show rows 1 and 2 exactly once |
| CM-005 | Seed row; create WAL with committed frames for rows 2 and 3, then simulate recovery interruption after first apply before cleanup/checkpoint | next reopen shows rows 1, 2, 3 exactly once; repeated reopen remains idempotent |
| CM-006 | Seed row; committed WAL frame for row 2 followed by a deterministic incomplete/invalid-length trailing fragment, such as a short header or declared payload length with missing bytes. Do not use a complete checksum-invalid or complete invalid-magic frame for this case. | reopen exits 0 with rows 1 and 2 visible; trailing fragment ignored/truncated; no CLI output/error change |

## Verification Strategy
Required commands:
- `./scripts/verify`
- `cargo test --test crash_matrix`
- `./scripts/verify_crash_matrix`

`./scripts/verify_crash_matrix` should fail if:
- any required case ID or evidence ID is missing from the report,
- `cargo test --test crash_matrix` fails,
- `target/crash_matrix/crash_matrix_report.md` is not created,
- report rows omit expected rows, actual rows, reopen command, WAL/file-format assertion, or exit status.
- report rows are not derived from the executed crash matrix run. Acceptable implementations are: `tests/crash_matrix.rs` writes per-case JSON/TSV/line records consumed by the script, or the script runs the same case harness in report mode and records observed rows/status before validating required identifiers.

## Acceptance Mapping
| Acceptance Item | Planned Evidence |
|---|---|
| deterministic write/WAL/commit/recovery boundaries covered | `tests/crash_matrix.rs` CM-001..CM-006 |
| fixed seed or named fixture for every case | required tracked `tests/fixtures/crash_matrix/` manifest/README plus test helper names |
| pre-commit row invisible after reopen | CM-001, CM-002, CM-003 assertions |
| committed row visible and idempotent | CM-004, CM-005, CM-006 assertions |
| existing WAL regressions preserved | `./scripts/verify` including `cargo test` and existing `tests/wal_recovery.rs` |
| WAL compatibility documented | `docs/file_format.md` diff or final report rationale if already sufficient |
| no CLI contract change | exact stdout/stderr/exit assertions and no `docs/cli_contract.md` diff unless needed |
| report generated | `target/crash_matrix/crash_matrix_report.md` created by `./scripts/verify_crash_matrix` from observed matrix results |

## Stop Conditions
- Satisfying a case requires changing `spec.md` or `contracts.md`.
- User-facing CLI output/error changes become necessary but cannot be documented and tested in scope.
- A second recovery attempt is required after verifier rejection; contract requires escalation.
- CM-006 cannot be represented as a successful reopen using an incomplete/invalid-length trailing fragment under current documented WAL semantics.
