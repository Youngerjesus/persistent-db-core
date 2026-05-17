# Final Readiness

## Verdict
GO

The approved package is ready for implementation. `spec.md` and `contracts.md` were adopted without rewrite, and derived planning artifacts close the implementation boundary, evidence strategy, risk controls, and verification commands.

## Implementation Handoff Summary
- Build `db check <path>` as a CLI-only invariant checker.
- Keep the checker read-only: no missing-file creation, no WAL replay mutation, no repair.
- Add focused black-box CLI tests in `tests/db_check.rs` for valid and corrupted fixtures.
- Update `tests/cli_contract.rs` because `check` moves from reserved to supported.
- Add a narrow checker module plus storage/sql helper APIs instead of embedding invariant logic in `main.rs`.
- Update durable docs for the new command and checker compatibility note.

## Required Evidence After Implementation
- `cargo test --test db_check`
- `scripts/verify`
- Evidence mapping for all Candidate Acceptance Criteria.
- Explicit note that UI/visual/UX evidence is not applicable under the frozen CLI-only contract.

## Blockers
None for implementation handoff.
