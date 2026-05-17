# Documentation Sync

Use this during `finish` after verification passes and before committing.

The goal is not "write more docs." The goal is to keep the repository's shipped knowledge aligned with code and verified behavior, while removing stale or misleading active guidance.

## Source Of Truth Order

Resolve conflicts in this order:

1. `code`
2. current-session verification and test evidence
3. directly impacted active companion docs under `specs/**`
4. relevant `docs/*/memory.md`
5. contributor-facing docs such as README, FAQ, runbooks
6. archive/history documents

This repo does not currently use a standard top-level release-doc stack such as
`CHANGELOG.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, or `VERSION`. Do not invent
those workflows during `finish`.

If the higher-priority layers do not establish the truth clearly enough, block `finish` and ask the user instead of guessing.

## Finish-Sync Workflow

`@context-synchronizer` should inspect the shipped diff and classify each touched or nearby doc as:

- `update`
- `delete`
- `keep`
- `ignore`
- `block`

Each non-`ignore` classification must have a short reason tied to code or verified behavior.

## Update Rules

Update docs when the shipped change clearly introduced any of the following:

- active companion docs that are now stale or contradictory
- durable operating knowledge that code alone does not communicate
- design rationale that future implementers need to understand current structure
- FAQ-worthy behavior that repeatedly causes reviewer or operator confusion
- durable architecture or product decisions worth preserving in `docs/*/memory.md`
- project progress and macro-history, which should always be maintained on finish
- stale active docs that should be deleted because they now mislead contributors

Be conservative with narrative rewrites. Prefer factual corrections and short additions.

## Skip Rules

Do not:

- document local facts that are obvious from code
- duplicate current implementation details in prose
- run a repo-wide markdown cleanup sweep
- update old specs just because they exist
- rewrite product-positioning docs unless the shipped change clearly invalidates them
- add memory entries for one-off bug fixes, temporary workarounds, test commands, or
  ephemeral environment notes
- preserve temporary debugging notes, investigation scratchpads, or session logs
- create new docs unless there is a clear durable context gap that cannot be solved by updating an existing active doc

## Blocking Rules

Return `block` and stop `finish` when:

- documents conflict and code plus verification still cannot prove the right answer
- runtime behavior is clear but policy or design intent is still ambiguous
- a deletion candidate might actually be archive/reference material
- verification evidence is too weak to justify doc changes safely

Blocker reports should include:

- conflicting or ambiguous docs
- verified facts from code/tests/runtime evidence
- the missing decision
- the question that must be answered by the user

## Repo Priority Targets

Check these first, in order:

1. `work_queue/progress.md`
2. `docs/history_archives/history.md`
3. relevant `docs/*/memory.md`
4. directly impacted companion docs under `specs/**`
5. contributor-facing docs only if there is durable context that code cannot carry

### `work_queue/progress.md`

Always add a new progress entry for the finished task and keep only the latest 10 entries.

### `docs/history_archives/history.md`

Always append one concise macro-history sentence describing the shipped milestone in plain
language.

### `docs/*/memory.md`

Only add durable knowledge such as:

- stable provider choices
- architectural boundaries
- long-lived integration rules
- permanent domain invariants

### `specs/**`

Touch only the docs directly impacted by the shipped change. Good candidates:

- `quickstart.md`
- `development_state.md`
- `readiness.md`
- `lessons.md`

Avoid broad rewrites of `spec.md`, `contracts.md`, or `tasks.md` unless `finish` is being
used to close a spec-driven task and the change would otherwise leave those documents
obviously misleading.

## PR Summary Requirement

The PR body should include a `Documentation Sync` section that states:

- which docs changed
- what changed in each
- which docs were deleted and why
- whether any memory docs changed
- whether no doc updates were needed beyond progress/history
