---
name: finish
description: Closes a completed coding task by validating readiness, syncing repo-core documentation, updating progress/history and touched component memory files, committing the full worktree scope for the active task, pushing the branch, opening a PR, and merging it once required verification is complete.
---

# Finish Workflow

Use this skill when coding work is actually complete and the user wants to wrap up the branch cleanly through documentation sync, records, local commit, push, PR, and merge.

For lightweight local-only closure after an `implementation-brake` `[SHIP]` verdict, use `closeout` instead. `finish` is the heavier branch/PR/merge workflow and should not be selected when the user only asks for evidence cleanup, Context Sync, follow-up summary, and a local commit.

## Core rule

- **Golden Rule for PRs**: PR을 작성할 때, 프로젝트를 처음 접하는 대학생 동료 개발자도 작업 내용을 충분히 이해하고 무엇을 리뷰해야 할지 명확하게 파악할 수 있을 정도로 상세하고 친절하게 작성해야 합니다.
- Do not run finish steps mechanically. First verify that the task is genuinely ready to close. If tests, review, or spec closure are still missing, stop and report that instead of forcing a merge.
- Finish includes documentation sync for this repo. Do not treat docs as optional cleanup after the fact.
- In this repo, documentation sync means keeping the shipped knowledge model aligned with code and verified behavior. It does not mean growing docs by default.

## Workflow

### 1. Validate closure

1. Inspect the current state with `git status`, `git diff`, and the active branch name.
2. Treat the current worktree as the task scope by default. Do not exclude files merely because the change set is broad or spans multiple areas.
3. Only treat files as out of scope when there is clear evidence that they are accidental noise or generated output.
4. Run all relevant project tests (e.g., backend `pytest`, frontend `vitest`/`playwright`) and verify that they pass successfully. Do not proceed if any tests fail.
5. Run the required verification for the work if it has not already been run in the current session or if the tree changed since the last run.
6. If the feature is spec-driven, confirm the relevant `tasks.md`, `contracts.md`, or equivalent exit criteria are not being bypassed.

### 2. Sync shipped documentation

Use the repo-specific rules in `references/documentation-sync.md`.

1. Review the task diff and identify doc-relevant behavior changes before committing.
2. Delegate the actual documentation sync work to `@context-synchronizer` by default. The synchronizer is allowed to edit docs during `finish`.
3. Treat `code + currently verified behavior` as the highest-priority source of truth. Use docs to reflect shipped reality, not to override it.
4. Update only repo-core docs by default:
   - directly impacted companion docs under `specs/**` when shipped behavior makes current guidance stale
   - `work_queue/progress.md`
   - `docs/history_archives/history.md`
   - relevant `docs/*/memory.md`
   - contributor-facing docs such as FAQ or runbooks only when code alone cannot carry the durable operating context
5. Prefer factual sync over broad rewrites. This repo is spec-heavy, so targeted updates and deletions beat repo-wide markdown sweeps.
6. Do not invent generic release artifacts such as `CHANGELOG.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, or `VERSION` unless the repo actually uses them.
7. If no docs needed updates beyond progress/history, say so explicitly in the final report and PR body.
8. If the synchronizer cannot determine the correct doc state from code and verification alone, stop the finish flow and report a blocker instead of guessing.

### 3. Update project records

1. Add a new entry to `work_queue/progress.md`.
2. Keep only the latest 10 entries in `work_queue/progress.md`.
3. Append one concise macro-history sentence to `docs/history_archives/history.md`.

### 4. Update component memory

Inspect component memory files that are relevant to the touched areas, typically `docs/*/memory.md` (for example: `auth`, `payments`, `design_system`, `backend`, `frontend`, or other domain-specific components present in the repo).

Add to memory only if the session introduced a durable decision such as:
- a stable provider choice
- a permanent domain invariant
- a long-lived integration rule
- a non-negotiable architectural boundary

Do not add:
- temporary progress notes
- one-off bug fixes
- environment secrets or URLs
- ephemeral test commands

If no durable memory changed, leave the memory files untouched and say so in the final report.

### 4A. Handle doc-sync blockers

Stop `finish` and ask the user when any of the following occurs:

1. Two documents conflict and `code + current verification` still do not reveal which one is correct.
2. The current runtime behavior is visible, but the intended long-term policy or design rationale cannot be derived from code or active specs.
3. A deletion candidate might actually be an archive/reference artifact rather than stale active guidance.
4. Verification is too weak to justify documentation changes safely.

When blocked, report:
- the conflicting or ambiguous docs
- the concrete code/verification facts that were confirmed
- the unresolved decision that still needs human input
- the exact question that must be answered before `finish` can continue

### 5. Commit cleanly

1. Stage and commit the full set of changes produced in the current worktree for the active task.
2. Do not exclude files merely because the change set is broad or spans multiple areas.
3. Treat the current worktree as one task scope unless there is clear evidence that a file is accidental noise or generated output.
4. Write a descriptive commit message that explains what changed and why.

### 6. Create Pull Request And Merge

1. Push the working branch to origin.
2. Open a PR against `main`. 
   - **CRITICAL**: The PR description must be highly detailed so that colleague developers can fully understand what was done, why it was done, and exactly how they should review the changes.
   - **Recommendation**: Use `@document-writer` to generate the PR body when the explanation burden is non-trivial or when new FAQ/runbook text had to be written. Do not use it as the primary stale-doc detector.
   - Add a `Documentation Sync` section that says which repo-core docs changed, what changed in each, and whether memory files changed.
   - In `Documentation Sync`, also state whether any docs were deleted and why.
   - The PR body must include a `Reviewer E2E Scenarios` section that lists the concrete end-to-end flows a human verifier should actually execute.
   - For each reviewer E2E scenario, include preconditions or setup, exact step-by-step actions, required test account/fixture/env flag if any, and the expected visible outcome or acceptance signal.
3. Merge the PR automatically once the branch is pushed, the PR is created successfully, and the required local verification has already passed in this finish run.
4. If repository policy requires a merge strategy selection, prefer the repository default unless the task or repo documents require a specific strategy.
5. After merge, notify the user with the merged PR reference and confirm whether the branch is ready for cleanup.

If remote access, GitHub CLI auth, or repository policy blocks any step, stop and report the exact blocker.

## Final response

Report:
- what was verified
- what documentation was synced, or whether only progress/history changed
- what source-of-truth evidence justified the doc sync, if there were non-trivial doc changes
- what records were updated
- whether any component memory changed
- commit, PR, and merge status
- whether the worktree is fully committed, pushed, merged, and ready for later cleanup
- any blocker that prevented a full finish
