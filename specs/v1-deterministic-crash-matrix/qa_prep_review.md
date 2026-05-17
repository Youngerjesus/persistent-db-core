# QA Prep Verification Review: v1-deterministic-crash-matrix

Verdict: success

## Scope Checked

- Canonical QA mapping: `specs/v1-deterministic-crash-matrix/qa_mapping.md`
- Implementation task list: `specs/v1-deterministic-crash-matrix/tasks.md`
- Spec/contract/design/plan consistency:
  - `specs/v1-deterministic-crash-matrix/spec.md`
  - `specs/v1-deterministic-crash-matrix/contracts.md`
  - `specs/v1-deterministic-crash-matrix/design.md`
  - `specs/v1-deterministic-crash-matrix/plan.md`
- Generated QA scaffold:
  - `tests/crash_matrix.rs`
  - `tests/fixtures/crash_matrix/README.md`
  - `scripts/verify_crash_matrix`
  - `docs/file_format.md`

## Findings

No retry-blocking QA prep gaps found.

The QA mapping covers every `tasks.md` item T001 through T027 with concrete preferred commands and task-scoped green criteria. The mapping also identifies current red expectations for the crash matrix test and runner without claiming implementation success.

Scenario coverage is not limited to happy paths. The mapped and scaffolded cases cover absent/empty WAL, partial WAL header or payload, uncommitted/rolled-back WAL state, idempotent repeated reopen, interrupted recovery re-entry, corrupt/incomplete trailing WAL bytes, report-missing failure, stale/current-run evidence, CLI contract preservation, and protected-area avoidance.

The scaffold is consistent with the spec, contract, plan, and design. It uses `db exec <db_path> "SELECT * FROM items;"` rather than unsupported general `ORDER BY`, matching the design rationale that primary-key table scans provide deterministic ordering in the current CLI contract. CM-003 correctly maps "commit marker absent" to current V1 WAL state byte `0x02` (`WAL_STATE_ROLLED_BACK`) rather than an unknown-state corrupt frame. CM-006 is constrained to incomplete/invalid-length trailing fragments, preserving complete corrupt frame storage-error behavior.

The fixture manifest names `seed_committed_one`, CM-001 through CM-006, crash points, evidence ids, expected rows, and deterministic byte sources. `docs/file_format.md` already includes the WAL sidecar compatibility note for absent sidecar, incomplete trailing frame cleanup, committed prefix replay/idempotence, and complete corrupt frame storage errors.

## Command Evidence

- `cargo test --test crash_matrix`: expected red. Result: failed with 1 inventory test passing and all six CM scaffold tests failing with case id, crash point, evidence id, reopen command, expected rows, and WAL/file-format assertion text.
- `./scripts/verify_crash_matrix`: expected red. Result: failed because it delegates to the red crash matrix test before report validation.

## Implementation Readiness

`impl_exec` can proceed without additional QA clarification. The next implementation owner should replace the six red scaffold panics with executable deterministic fixture/CLI assertions, ensure observed actual rows and exit status flow into `target/crash_matrix/crash_matrix_report.md`, and make `scripts/verify_crash_matrix` validate the current-run report.
