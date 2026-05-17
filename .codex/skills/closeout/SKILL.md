---
name: closeout
description: Thin post-[SHIP] executor for closing an already approved implementation by summarizing evidence, syncing active context, optionally doing small green refactors, routing follow-ups in the final response, and creating a local commit without push, PR, merge, or full re-review.
---

# Closeout

Use this skill after `implementation-brake` has returned a clear `[SHIP]` verdict and the user wants the work operationally closed.

This skill is not the implementation review gate. `implementation-brake` owns the ship-readiness judgment. `closeout` trusts a valid `[SHIP]` verdict, checks that the handoff evidence is present and not contradicted, performs final hygiene, and commits only the intended changes.

## Entry Conditions

- Start only from an explicit `implementation-brake` `[SHIP]` verdict for the current implementation.
- If `[SHIP]` is missing, ambiguous, stale after behavior changes, or mixed with unresolved must-fix findings, route back to `implementation-brake` instead of closing.
- If the `[SHIP]` handoff names required evidence, confirm that evidence is available and not contradicted before committing.
- Do not independently rerun the full implementation review or reinterpret acceptance criteria.

## Verdicts

- `[CLOSED_COMMITTED]`: closeout completed and a local commit was created.
- `[CLOSEOUT_BLOCKED]`: closeout stopped because evidence, verification, Context Sync, or staging isolation is unsafe.
- `[FOLLOW_UP_ROUTED]`: current work is closed, and separable follow-up is reported in the final response only.

## Workflow

### 1. Confirm the ship handoff

Read the `[SHIP]` verdict and capture:

- acceptance, spec, contract, or caller-expectation evidence
- targeted, broader, browser/e2e, log, or manual verification evidence
- stated plan/spec drift result
- residual risk judgment
- separable follow-up notes, if any

Block with `[CLOSEOUT_BLOCKED]` if required evidence is missing, contradicted, or too vague to identify.

### 2. Inspect the working tree

Run `git status` and inspect intended diffs.

- Existing unrelated dirty files are not blockers by themselves.
- Stage and commit only intended changes for the current task.
- If intended and unrelated changes are mixed in the same file and cannot be isolated safely, stop with `[CLOSEOUT_BLOCKED]` and ask for human direction.
- Never use destructive cleanup to force a clean tree.

### 3. Sync active context

Use `context-sync` when the implementation changed active workflow, policy, requirement, terminology, or durable operating guidance.

Search the relevant active doc surface, including:

- `AGENTS.md`
- `DESIGN.md`
- `README.md`
- `.codex/**`
- `docs/**`
- `specs/**`
- project workflow docs

Do not blindly find-and-replace. Do not rewrite archive/history material unless it is explicitly part of the active operating contract. If active docs conflict and code plus verified behavior do not resolve the conflict, stop with `[CLOSEOUT_BLOCKED]`.

### 4. Optional small green refactor

Small refactors are allowed only after `[SHIP]` and only when they are:

- behavior-preserving
- scoped to the intended task area
- simpler than leaving the code as-is
- safe to verify with targeted tests immediately

Do not change acceptance criteria, user-visible behavior, policy, data shape, or integration contracts during closeout.

After any refactor, run the relevant targeted tests and perform a mini Implementation Brake check:

- no behavior changed
- acceptance evidence still closes
- contracts and caller expectations still match
- no new plan/spec drift was introduced
- residual risk did not increase

If tests fail or the mini check fails, stop with `[CLOSEOUT_BLOCKED]` and route back to `implementation-brake`.

### 5. Route follow-up in the final response only

Follow-up is allowed only as summary routing.

- Name the follow-up candidate and recommended next skill or owner.
- Do not create new task, spec, plan, or queue artifacts.
- Do not expand closeout into a new implementation phase.
- If the current work can close and follow-up is separable, use `[FOLLOW_UP_ROUTED]` alongside the closeout summary.

### 6. Commit locally

When evidence is present, verification is green, context is synced or explicitly unnecessary, and intended changes can be isolated:

1. Stage only intended files or hunks.
2. Re-check staged diff before committing.
3. Create one local commit with a descriptive message.
4. Do not push, open a PR, merge, deploy, or update remote state.

If staging cannot isolate intended changes, stop with `[CLOSEOUT_BLOCKED]`.

## Output Shape

Report:

1. **Verdict**
2. **Ship evidence confirmed**
3. **Verification run**
4. **Context Sync**
5. **Refactor notes**
6. **Follow-up routing**
7. **Commit**
8. **Blocked reason**, if any

## Boundaries

- `implementation-brake` decides whether the implementation is ship-ready.
- `closeout` finalizes a `[SHIP]` implementation locally.
- `finish` is a heavier branch/PR/merge workflow and must not be used for lightweight local closeout unless the user explicitly asks for push, PR, or merge behavior.
- Auto-merge and auto-deploy remain out of scope.
