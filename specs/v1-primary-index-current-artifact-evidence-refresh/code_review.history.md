## 2026-05-20T22:26:35+0900 - archived before code_review_verify_2 refresh

# Code Review: Primary Index Current-Artifact Evidence Refresh

Verdict: FAIL

## Scope

- Task: `task-2026-05-20-19-52-09-v1-primary-index-current-artifact-evidence-refresh`
- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Reviewed change set: `git diff main...HEAD` plus current uncommitted worktree diff.
- Commit range: `git log --oneline main..HEAD` is empty; current task delta is uncommitted in the worktree.
- Current HEAD reviewed: `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed`
- QA mapping read: `specs/v1-primary-index-current-artifact-evidence-refresh/qa_mapping.md`
- Evidence report read: `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
- Verification sampled by this review:
  - `cargo test --test primary_index` exited `0`.
  - `cargo test --test sql_exec primary_key` exited `0`.
  - `scripts/verify` exited `0`.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Correctness, regressions, completeness, spec mismatch, merge safety | invoked | Agent `019e457b-5cdb-71f0-b20a-7bfe6fa481ed`: reported one Must Fix finding that evidence is not anchored to the actual merge candidate commit SHA; no additional code/regression findings. | `CR-001` | None | N/A |
| `testing-reviewer` | Coverage gaps, negative paths, edge cases, merge confidence | invoked | Agent `019e457b-5d36-7962-8699-071385c6c9bf`: no Must Fix findings; residual risk that `IndexedRow` replay handling is touched but not directly covered by task-specific duplicate-fixture tests. | None | None | N/A |
| `security-reviewer` | Persisted file decoding, CLI process/file boundary, error rendering | invoked | Agent `019e457b-5d8d-70b1-804e-1db15ff0545d`: no Must Fix findings; residual risk that persisted table/key values are reflected in stderr by contract. | None | None | N/A |
| `performance-reviewer` | Query/replay loops and index lookup/rebuild paths | invoked | Agent `019e457b-5dfb-7a71-8c61-6b6e2da1af09`: no performance findings; new clones occur only on duplicate-key error paths. | None | None | N/A |
| `maintainability-reviewer` | Complexity, duplication, brittle structure | unavailable | Self-applied fallback: reviewed `RecordLoadError` plumbing and duplicated fixture helpers. No open maintainability finding beyond the evidence identity issue. | None | None | No `maintainability-reviewer` agent is available in this runtime. |
| `red-team-reviewer` | Additive bias, proxy-success evidence, cross-category gaps | unavailable | Self-applied fallback: identified proxy-success risk in final evidence using base HEAD plus dirty-worktree identity instead of an actual merge candidate SHA. | `CR-001` | None | No `red-team-reviewer` agent is available in this runtime. |
| `database-reviewer` | SQL replay, indexes, persistence boundary, durable state | unavailable | Self-applied fallback: reviewed primary-key replay paths, duplicate persisted row fixture, `db check` label preservation, and durable docs. No additional open DB finding. | None | None | No `database-reviewer` agent is available in this runtime. |
| `api-reviewer` | CLI/documented error-shape contract changed | unavailable | Self-applied fallback: reviewed `src/main.rs` error rendering and docs updates. The new duplicate-primary-key stderr matches the task contract. | None | None | No `api-reviewer` agent is available in this runtime. |

## Findings

### `CR-001` Must Fix: final evidence is not anchored to the actual merge candidate SHA

- Severity: Medium
- Files:
  - `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md:9`
  - `docs/v1_acceptance.md:21`
- Problem: The evidence package records base HEAD `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` and then describes the verified artifact as that base plus a dirty-worktree identity manifest. The contract requires final evidence tied to the current managed repo SHA. Because the task delta is uncommitted, the cited SHA identifies `main` before the implementation/evidence delta, not the actual merge candidate containing the reviewed changes.
- Impact: The behavior appears correct under test, but the acceptance row and final review cannot be used as durable proof for the shipped artifact. A later reader or gate evaluator cannot map `REQ-7-implement-integer-primary-key-as-9c698e08` to a concrete commit that contains the implementation, tests, verifier script, docs, and evidence.
- Required repair: Commit or otherwise establish the actual managed-repo artifact SHA for the task delta, then refresh `final_review.md` and `docs/v1_acceptance.md` so they cite that current SHA directly. Keep the row scoped only to `gate-v1-indexes` and `REQ-7-implement-integer-primary-key-as-9c698e08`.

## Must Fix Now

- `CR-001`: Refresh final evidence and acceptance documentation so the required proof is anchored to the actual current managed repo SHA for the merge candidate, not only to base HEAD plus a dirty-worktree manifest.

## Residual Risks

- `tests/sql_exec.rs:614` and `tests/primary_index.rs:208` prove the required valid duplicate persisted `R` row fixture. The implementation also routes duplicate-primary-key errors through `IndexedRow` replay in `src/sql.rs:400`, but this task explicitly excludes broader mutation/index-maintenance claims, so this remains a residual coverage note rather than a merge blocker.
- `src/main.rs:114` reflects persisted table name and primary-key value into stderr. The task contract requires that exact error shape, and identifiers are validated before this path, but attacker-supplied database files may now reveal record identifiers in logs where the previous generic error did not.
- Specialist fallbacks were self-applied for unavailable `maintainability-reviewer`, `red-team-reviewer`, `database-reviewer`, and `api-reviewer`.

## Next Action

Send to `code_review_retry` for evidence identity repair. Do not broaden the implementation scope; the open item is the SHA/evidence anchoring problem, not the primary-key behavior under test.

## Updated At

2026-05-20T22:11:45+0900

## 2026-05-20T22:38:52+0900 - archived before code_review_verify_3 refresh

# Code Review Verification: Primary Index Current-Artifact Evidence Refresh

Verdict: FAIL

## Scope

- Task: `task-2026-05-20-19-52-09-v1-primary-index-current-artifact-evidence-refresh`
- Phase: Code Review Verification, round 2
- Gate: `gate-v1-indexes`
- Requirement: `REQ-7-implement-integer-primary-key-as-9c698e08`
- Verified change set: committed range `main..HEAD` plus current uncommitted worktree diff.
- Commit range: `git log --oneline main..HEAD` shows `aec7f00 Refresh primary index current artifact evidence`.
- Current HEAD checked: `aec7f00c684376ab730c120773e6d63d048ab35c`.
- Current uncommitted worktree diff before this report update: `docs/v1_acceptance.md`, `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256`, and `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`.
- Prior latest report was archived to `specs/v1-primary-index-current-artifact-evidence-refresh/code_review.history.md`.
- Verification commands executed in this phase:
  - `cargo test --test primary_index` exited `0`.
  - `cargo test --test sql_exec primary_key` exited `0`.
  - `scripts/verify_primary_index_acceptance` exited `0`.
  - `scripts/verify` exited `0`.
  - `git diff --check` exited `0`.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Correctness, regressions, completeness, spec mismatch, merge safety | prior routing evidence verified; no new subagent invoked in verify phase | Archived prior report cited Agent `019e457b-5cdb-71f0-b20a-7bfe6fa481ed` and accepted `CR-001`; current `git show HEAD:.../final_review.md`, current `git diff`, and `docs/v1_acceptance.md` confirm the evidence-identity issue remains in a different form. | `CR-001` | None | Verify instructions prohibit starting new subagent reviews in this phase. |
| `testing-reviewer` | Coverage gaps, negative paths, edge cases, merge confidence | prior routing evidence verified; no new subagent invoked in verify phase | Archived prior report cited Agent `019e457b-5d36-7962-8699-071385c6c9bf`; focused tests and `scripts/verify` passed in this phase. | None | None | Verify instructions prohibit starting new subagent reviews in this phase. |
| `security-reviewer` | Persisted file decoding, CLI process/file boundary, error rendering | prior routing evidence verified; no new subagent invoked in verify phase | Archived prior report cited Agent `019e457b-5d8d-70b1-804e-1db15ff0545d`; no new security finding found during read-only verification. | None | None | Verify instructions prohibit starting new subagent reviews in this phase. |
| `performance-reviewer` | Query/replay loops and index lookup/rebuild paths | prior routing evidence verified; no new subagent invoked in verify phase | Archived prior report cited Agent `019e457b-5dfb-7a71-8c61-6b6e2da1af09`; full verification passed, and no new performance-side regression was observed in the diff. | None | None | Verify instructions prohibit starting new subagent reviews in this phase. |
| `maintainability-reviewer` | Complexity, duplication, brittle structure | unavailable fallback verified | Archived prior report recorded a self-applied fallback because no `maintainability-reviewer` agent was available; current diff does not add a new maintainability blocker beyond evidence identity drift. | None | None | No `maintainability-reviewer` agent was available to the prior review runtime; verify phase does not start new reviewers. |
| `red-team-reviewer` | Additive bias, proxy-success evidence, cross-category gaps | unavailable fallback verified | Archived prior report recorded a self-applied fallback; this verify pass independently confirms the proxy-success risk persists because the worktree now cites `aec7f00...` as if it contains the refreshed evidence, while `git show HEAD:.../final_review.md` still contains the stale base-SHA wording. | `CR-001` | None | No `red-team-reviewer` agent was available to the prior review runtime; verify phase does not start new reviewers. |
| `database-reviewer` | SQL replay, indexes, persistence boundary, durable state | unavailable fallback verified | Archived prior report recorded a self-applied fallback; focused primary-index and SQL tests passed in this phase. | None | None | No `database-reviewer` agent was available to the prior review runtime; verify phase does not start new reviewers. |
| `api-reviewer` | CLI/documented error-shape contract changed | unavailable fallback verified | Archived prior report recorded a self-applied fallback; CLI stderr behavior is covered by passing tests and baseline verification. | None | None | No `api-reviewer` agent was available to the prior review runtime; verify phase does not start new reviewers. |

## Findings

### `CR-001` Must Fix: evidence still is not anchored to the artifact it claims

- Severity: Medium
- Files:
  - `specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md`
  - `docs/v1_acceptance.md`
  - `specs/v1-primary-index-current-artifact-evidence-refresh/artifact_identity.sha256`
- Problem: The code-review retry committed the task as `aec7f00c684376ab730c120773e6d63d048ab35c`, then left new evidence edits uncommitted that cite that SHA as the "current managed repo artifact SHA." However, `git show HEAD:specs/v1-primary-index-current-artifact-evidence-refresh/final_review.md` still shows the stale wording that the verified artifact is base HEAD `69fc6b95640bdeed3f7d4249d2ffedc5e6c336ed` plus a dirty-worktree identity manifest. The current worktree version claims local commit `aec7f00...` "fixes the primary-index implementation, test, helper, docs, and task evidence into a concrete managed repo commit," but that refreshed evidence wording and the matching acceptance row are not actually in `aec7f00...`.
- Impact: The behavioral implementation passes, but the review/evidence trail still cannot be used as durable proof for the merge artifact. A reader checking the cited commit sees the old evidence package that the prior code review already rejected, while the fixed evidence exists only as another dirty-worktree delta.
- Required repair: Establish a truthful artifact identity for the current merge candidate and refresh `final_review.md`, `docs/v1_acceptance.md`, and `artifact_identity.sha256` so their SHA/manifest language does not claim that uncommitted evidence edits are contained in `aec7f00...`. Keep the repair scoped to evidence identity; no primary-index behavior change is indicated by this verification.

## Must Fix Now

- `CR-001`: Repair the evidence identity drift so the final evidence and acceptance row are anchored to the actual merge candidate artifact, not to a commit that still contains the stale rejected evidence wording.

## Residual Risks

- The code behavior and focused task tests passed in this verify phase; the remaining blocker is evidence/report integrity, not observed primary-index runtime behavior.
- `scripts/verify` is slow because benchmark acceptance tests run as part of the baseline, but it completed successfully in this phase.
- This verify phase did not invoke new specialist subagents, per instruction; it verified the prior routing table and current artifacts only.

## Next Action

Return to `code_review_retry` for evidence identity repair. Do not broaden implementation scope or refactor code; the open item is the inaccurate SHA/evidence anchoring.

## Updated At

2026-05-20T22:26:35+0900
