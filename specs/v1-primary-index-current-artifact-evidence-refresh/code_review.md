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
