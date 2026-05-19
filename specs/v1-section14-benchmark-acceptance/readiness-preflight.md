# Adoption Preflight: v1-section14-benchmark-acceptance

## Verdict
PASS

## Checks
| Check | Result | Evidence |
|---|---|---|
| `spec.md` exists | pass | `specs/v1-section14-benchmark-acceptance/spec.md` |
| `contracts.md` exists | pass | `specs/v1-section14-benchmark-acceptance/contracts.md` |
| upstream approval evidence | pass | `spec.md` contains `Status: APPROVED` |
| unresolved placeholders | pass | `rg "TODO|TBD"` found no canonical-input matches |
| protected-area dependency | pass | implementation plan does not require `ssot/` or `policies/` edits |
| blocker ambiguity | pass | `contracts.md` fixes CLI sentinel, JSON schema, fixture generation, formulas, thresholds, recovery bound, docs, and verification commands |

## Current Repo Context
- Latest checked HEAD: `d943f7404a992203822d00ef9a8194e766f15f87`.
- Current worktree state at preflight included the new untracked feature spec directory only.
- Current CLI still treats `bench` as a reserved future command in `src/main.rs`, `docs/cli_contract.md`, and `tests/cli_contract.rs`.
- Existing `scripts/verify_bench_acceptance` is the older 1k script-local benchmark and does not satisfy the frozen Section 14 contract.

## Notes
- `spec.md` and `contracts.md` are adopted without rewrite.
- No spec-loop re-entry or human escalation is required before derived planning.

