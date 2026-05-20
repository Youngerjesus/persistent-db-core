## 2026-05-20T15:19:50Z - Archived Stale QA Prep Review

# QA Prep Verification Review

Verdict: RETRY

## Findings

1. Missing canonical QA prep review artifact.
   - Expected: `specs/v1-transaction-wal-current-artifact-evidence-refresh/qa_prep_review.md`
   - Observed: file was absent at verification start.
   - Impact: the QA prep phase does not provide the required latest review/report SSOT for implementation handoff.

2. `verify_evidence_contract.sh` does not fully enforce the `qa_mapping.md` Task-Scoped Green criteria.
   - `T1` requires `evidence/current-repo-sha.txt` to record required file presence for `tests/wal_recovery.rs`, `tests/crash_matrix.rs`, `scripts/verify`, and `scripts/verify_crash_matrix`; the scaffold only checks SHA and `git status --short` command text.
   - `T2` and `T3` require command-specific exit `0` proof; the scaffold only requires command names plus at least one `exit_code: 0`, so a generic passing marker can satisfy multiple command rows.
   - `T6` requires every requirement row to include command, expected behavior, observed result, artifact refs, and blocker status; the scaffold only checks that the requirement IDs appear.
   - Impact: implementation could produce broad keyword-only evidence that passes the scaffold while leaving the QA contract under-specified.

3. Scheduler/control-plane ID leakage is documented as disallowed, but product evidence artifacts are not guarded by the scaffold.
   - `qa_mapping.md` correctly says `active_run_id`, `qa_prep_*`, `plan_*`, `impl_*`, `code_review_*`, and `final_*` must not be used as product evidence identity values.
   - The scaffold does not reject those exact scheduler/control-plane IDs in implementation-owned product evidence files such as `evidence/*.md`, `evidence/*.txt`, or `final_review.md`.
   - Impact: a future implementation pass could accidentally satisfy product identity fields with scheduler-run IDs instead of managed-repo SHA, command output, WAL sidecar identity, or crash matrix evidence.

## Required Retry Checklist

- Keep `qa_mapping.md` task coverage for `T1` through `T8`, but strengthen `verify_evidence_contract.sh` so each Task-Scoped Green criterion is mechanically checked or explicitly documented as manual-only in the mapping.
- Add checks that `current-repo-sha.txt` records required file presence for `tests/wal_recovery.rs`, `tests/crash_matrix.rs`, `scripts/verify`, and `scripts/verify_crash_matrix`.
- Require command-specific passing evidence in `command-log.md`, not just one global `exit_code: 0`. Each required command should have an adjacent or structured `exit_code: 0` marker.
- Require each `requirement-evidence.md` row to include exact requirement ID, command, expected behavior, observed result, artifact refs, and blocker status.
- Add scaffold rejection for scheduler/control-plane identity values in implementation-owned product evidence files: `active_run_id`, `qa_prep_*`, `plan_*`, `impl_*`, `code_review_*`, and `final_*`, while allowing explanatory mentions in `qa_mapping.md` and phase-owned run reports.
- Re-run and record QA-prep red evidence:
  - `bash -n specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` exits `0`.
  - `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` exits non-zero before implementation evidence exists.

## Verification Notes

- `qa_mapping.md` covers all `tasks.md` tasks `T1` through `T8`.
- Scenario coverage is not happy-path-only; it includes stale evidence, dirty state, missing files, rollback/uncommitted WAL bytes, incomplete tail, idempotence, crash interruption, temp-path redaction, and retry/re-entry.
- `bash -n specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` exited `0`.
- `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh` exited `1` on missing `evidence/current-repo-sha.txt`, which is the expected pre-implementation red state.
- No blocking scheduler phase ID leakage was found in `qa_mapping.md` Task-Scoped Green criteria or durable managed tests. Existing mentions are explanatory/control-plane context, not product evidence identity values.
