# Final Readiness: v1-deterministic-crash-matrix

## Verdict
GO for implementation loop.

## Ready Inputs
- Approved canonical spec: `spec.md`
- Frozen contract: `contracts.md`
- Research: `research.md`
- Implementation plan: `plan.md`
- Technical design: `design.md`
- Dependency-ordered tasks: `tasks.md`
- Analysis report: `analysis_report.md`
- Progress ledger: `spec-progress.md`

## Required Implementation Evidence
- `tests/crash_matrix.rs`
- `tests/fixtures/crash_matrix/`
- `scripts/verify_crash_matrix`
- `docs/file_format.md`
- `target/crash_matrix/crash_matrix_report.md`
- Scheduler final report artifact verification evidence section

## Required Verification Commands
- `./scripts/verify`
- `cargo test --test crash_matrix`
- `./scripts/verify_crash_matrix`

## Guardrails For The Implementer
- Do not edit `spec.md` or `contracts.md`.
- Do not edit protected `ssot/` or `policies/`.
- Keep any crash injection private/test-only unless a human-approved spec change allows public surface.
- Always create `tests/fixtures/crash_matrix/` with a tracked manifest/README or named seed descriptors.
- Populate `target/crash_matrix/crash_matrix_report.md` from observed per-case execution results, not from a static success template.
- Implement CM-006 only as the no-CLI-change successful reopen path using an incomplete/invalid-length trailing fragment after a committed prefix.
- Implement CM-003's "commit marker absent" as current WAL state byte `0x02` (`WAL_STATE_ROLLED_BACK`) and record that mapping in the report.
- Preserve existing `tests/wal_recovery.rs` behavior and baseline `./scripts/verify`.
- Escalate if verifier rejection would require a second recovery attempt.

## Final Note
The package is implementation-ready. Planning found no human-decision blocker or spec_loop re-entry requirement.
