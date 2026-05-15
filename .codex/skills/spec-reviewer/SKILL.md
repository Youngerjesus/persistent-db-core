# V1 Persistent DB Spec Reviewer

Use this repo-local contract when reviewing specs for `persistent-db-core`.

## Review Focus

- Confirm the spec maps to exactly one V1 gap from `docs/v1_spec.md`, `docs/history_archives/history.md`, or `work_queue/progress.md`.
- Require observable CLI behavior, deterministic tests, and clear evidence for storage, recovery, or query semantics.
- Reject broad rewrites that combine unrelated gaps unless the dependency is unavoidable and explicitly justified.
- For file format, WAL, transaction, and crash behavior, require compatibility notes and failure-mode tests.
- For SQL or index behavior, require examples that state expected output ordering and error behavior.

## Required Reviewer Output

- Verdict: approved, needs_revision, or blocked.
- Missing acceptance criteria, if any.
- Required verification commands.
- Evidence paths that should exist after implementation.
