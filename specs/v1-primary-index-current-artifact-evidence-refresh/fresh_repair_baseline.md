# Fresh Repair Baseline

## Trigger Finding

- `IB-001`: `final_review.md` and `docs/v1_acceptance.md` cited base HEAD
  `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` as current managed repo SHA
  while the verified implementation and evidence lived in the dirty task
  worktree.
- `IB-002`: the repaired artifact identity covered only tracked
  source/test/docs diff content while the canonical proof surface also cited
  untracked helper and feature evidence files.
- `IB-003`: the new persisted duplicate-primary-key invalid-storage stderr was
  asserted by tests but not documented in durable CLI, SQL subset, and file
  format docs.
- `IB-004`: `artifact_identity.sha256` included
  `impl_brake_review.md`, a mutable latest brake report that changes after the
  manifest is generated and therefore makes the frozen identity self-invalidating.

## Root Cause

The prior evidence used a commit SHA as the artifact identity without saying
that the tested artifact was the task worktree based on that SHA plus the
current uncommitted implementation, test, script, docs, and evidence delta.
That made the evidence reusable-looking and could imply the dirty delta already
existed in the base commit.

The first repair narrowed the digest to tracked source/test/docs changes, but
that still left cited proof artifacts outside the recorded identity. The
canonical identity must cover the focused verifier script and feature evidence
files when those files are named by the acceptance guide, QA mapping, or final
review. The durable docs drift came from stabilizing a new user-visible error
message through tests without adding the matching contract text.

The second repair overcorrected by including mutable latest review output in
the identity. Current brake and verify reports are validator SSOT surfaces, not
stable acceptance proof inputs. A frozen package identity must exclude mutable
latest review reports while still covering the stable task proof artifacts.

## Changed-File Classification

- Production: `src/main.rs`, `src/sql.rs`
- Test: `tests/primary_index.rs`, `tests/sql_exec.rs`
- Helper: `scripts/verify_primary_index_acceptance`
- Durable docs: `docs/v1_acceptance.md`
- Durable docs for duplicate persisted primary-key storage errors:
  `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md`
- Generated/runtime evidence: `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`, `specs/v1-primary-index-current-artifact-evidence-refresh/fresh_repair_baseline.md`
- Mutable latest review reports excluded from frozen identity:
  `specs/v1-primary-index-current-artifact-evidence-refresh/impl_brake_review.md`
  and any future `impl_review.md`
- Spec package inputs and QA prep artifacts retained as context: `specs/v1-primary-index-current-artifact-evidence-refresh/spec.md`, `contracts.md`, `qa_mapping.md`, and review-loop assets

## Stale/Generated Evidence Cleanup

The stale evidence identity in `final_review.md` and `docs/v1_acceptance.md`
will be replaced. Current launch evidence must identify the verified artifact
as base HEAD `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` plus the explicit task
worktree delta, not as that commit alone. The identity must include both the
tracked diff and stable cited untracked proof artifacts, including
`scripts/verify_primary_index_acceptance` and required feature evidence files
under `specs/v1-primary-index-current-artifact-evidence-refresh/`, while
excluding mutable latest brake/verify reports. Historical command results from
previous implementation passes are not reused as final proof; required commands
will be rerun after this evidence and documentation repair.

## Temporary Or Test-Only Workaround Cleanup

No test-only workaround or temporary product behavior was introduced for this
repair. The retry will not rewrite fixtures or weaken assertions.

## Legitimate Helper Or Harness Components Retained

`scripts/verify_primary_index_acceptance` is retained as a focused helper
because `qa_mapping.md` lists it as a preferred command and it runs the two
focused primary-index acceptance suites from the repo root.

## Regenerated Evidence Plan

- Refresh `final_review.md` to state:
  - base HEAD SHA,
  - dirty worktree status and changed/cited path set,
  - package identity digest covering tracked diff plus cited untracked helper
    and stable evidence files, excluding mutable latest review reports,
  - required command exit codes gathered after this repair.
- Refresh the `docs/v1_acceptance.md` row to cite the same repaired artifact
  identity and final evidence path.
- Update `docs/cli_contract.md`, `docs/sql_subset.md`, and
  `docs/file_format.md` to document the duplicate persisted primary-key
  invalid-storage stderr alongside the existing generic unknown-record-tag
  invalid-storage stderr.
- Rerun:
  - `cargo test --test primary_index`
  - `cargo test --test sql_exec primary_key`
  - `scripts/verify_primary_index_acceptance`
  - `scripts/verify`

## Remaining Verify Risk

Independent `impl_verify` must still rerun all required commands and confirm
that the evidence identity describes the exact artifact under verification.

## Next Brake Approval Target

The next `impl_brake_exec` pass should verify that `IB-001` is closed and may
record `Fresh Repair Cleared: yes` if the repaired evidence identity and command
proof are acceptable.
