---
name: sdd-autopilot
description: Automated SDD handoff workflow for an already-approved spec package. Adopts frozen `spec.md` and `contracts.md`, runs a scoped adoption preflight, then prepares downstream implementation-planning artifacts through research, plan, design, tasks, task details, subtasks, analysis, and final readiness without rewriting canonical inputs.
---

# SDD Autopilot Handoff Workflow

This skill prepares an approved Spec-Driven Development package for implementation handoff.
It is not a spec-generation or spec-repair workflow. The upstream CAO/spec_loop process owns product direction, requirement hardening, and canonical contract approval before this skill starts.

**Prerequisite:** The target feature directory already contains an approved `spec.md` and `contracts.md`. An optional `reference-manifest.json` may be present.

## Core Principles

- **Canonical inputs are frozen**: `spec.md` and `contracts.md` are adopted as approved handoff inputs. Do not edit `spec.md` or `contracts.md` during this workflow.
- **`contracts.md` is frozen**: downstream planning artifacts must conform to it. If a downstream artifact conflicts with `contracts.md`, stop and record either `spec_loop` re-entry or human escalation.
- **Handoff, not authorship**: create derived implementation-preparation artifacts (`research.md`, `plan.md`, `design.md`, `tasks.md`, analysis/readiness reports) without re-deciding approved product or contract content.
- **Blocker ambiguity stops the run**: if an implementer would need to make a new product, scope, acceptance, or contract decision, do not resolve it autonomously. Stop and document the blocker.
- **Preflight is narrow**: adoption preflight prevents wasted work; it is not the final GO/NO-GO gate.
- **Final readiness is authoritative**: only final readiness determines whether implementation may begin.
- Communicate and document primarily in **Korean (한국어)**.

## Execution Sequence

Execute the following steps in order. Keep all generated artifacts inside the resolved `specs/[spec-name]/` directory.

### Step 0: Adopt Approved Package

1. Identify `[spec-name]` from the user request, task metadata, branch, or existing feature directory.
2. Resolve the feature directory from the current context in this priority order:
   - existing `specs/[spec-name]/`
   - a `project_manager`-prepared spec directory already present in the current `task-*` worktree
   - `.specify` helper scripts via current branch, `SPECIFY_FEATURE`, or `SPECIFY_FEATURE_DIR`
3. Verify that `spec.md` and `contracts.md` already exist in the resolved directory.
4. If `reference-manifest.json` exists, treat it as optional supporting evidence.
5. Create or update `spec-progress.md` as the handoff evidence index shown below. It is a ledger of evidence and judgments, not a completion checklist.

#### `spec-progress.md` Template

```markdown
# SDD Handoff Progress: [spec-name]

## Canonical Inputs
- spec: spec.md
- contract: contracts.md
- source: [control-plane or worktree source]
- adopted_without_rewrite: yes

## Gates
| Step | Status | Evidence | Notes |
|---|---|---|---|
| Adoption preflight | pending | readiness-preflight.md | |
| Research | pending | research.md | |
| Plan | pending | plan.md | |
| Design | pending | design.md | |
| Tasks | pending | tasks.md | |
| Task details | pending | tasks.md | details injected |
| Subtasks | pending | tasks.md | complex tasks expanded |
| Analyze | pending | analysis_report.md | |
| Final readiness | pending | readiness.md | |
```

### Step 1: Adoption Preflight

Perform a read-only check of the approved package before spending work on derived artifacts. Write the result to `readiness-preflight.md` and update the `Adoption preflight` row in `spec-progress.md`.

Check only:

- missing canonical artifacts (`spec.md`, `contracts.md`)
- unresolved `TODO`, `TBD`, or equivalent placeholders in canonical inputs
- missing approval evidence or lifecycle status indicating the package was approved upstream
- external dependency blockers that prevent planning from proceeding
- blocker ambiguity that would force a new implementation-time product, scope, acceptance, or contract decision

Do not treat missing downstream artifacts as preflight blockers. At this stage, `plan.md`, `design.md`, `tasks.md`, `research.md`, `analysis_report.md`, and `readiness.md` may not exist yet.

If preflight finds blockers:

1. Stop immediately.
2. Mark `Adoption preflight` as `blocked` in `spec-progress.md`.
3. Record the blocker and recommended next path in `readiness-preflight.md`: `spec_loop` re-entry for canonical package defects, or human escalation for policy/product approval questions.
4. Do not edit `spec.md` or `contracts.md`.

If preflight passes, mark the row as `passed` and continue.

### Step 2: Research (`/speckit:research`)

1. Run the research command to investigate technical unknowns and architecture choices, producing `research.md`.
2. Keep decisions within the bounds already established by `spec.md` and `contracts.md`.
3. If research reveals a required change to canonical scope, acceptance criteria, or contracts, stop and document `spec_loop` re-entry or human escalation instead of changing canonical inputs.
4. Update the `Research` row in `spec-progress.md` with status and evidence.

### Step 3: Plan (`/speckit:plan`)

1. Run the plan command to generate `plan.md` based on the approved package and completed research.
2. Verify that `plan.md` references any generated downstream authorities such as `data-model.md`, `contracts/`, or `quickstart.md` without contradicting `contracts.md`.
3. If the plan conflicts with `spec.md` or `contracts.md`, stop and record the conflict. Do not edit `spec.md` or `contracts.md`.
4. Update the `Plan` row in `spec-progress.md`.

### Step 4: Technical Design (`speckit-design`)

1. Run the design skill or command to generate `design.md` or the locally expected design artifact.
2. Verify that the design is implementation-ready and contract-compatible.
3. If the design requires a canonical spec or contract change, stop and record the issue for `spec_loop` re-entry or human escalation.
4. Update the `Design` row in `spec-progress.md`.

### Step 5: Tasks (`/speckit:tasks`)

1. Run the tasks command to generate `tasks.md`.
2. Verify that tasks are traceable to `spec.md`, `contracts.md`, `research.md`, `plan.md`, and `design.md`.
3. Keep task generation scoped to implementation handoff; do not introduce new product decisions.
4. Update the `Tasks` row in `spec-progress.md`.

### Step 6: Task Details (`/speckit:task-details`)

1. Run task details in explicit autopilot/non-interactive mode to enrich `tasks.md`.
2. Resolve only implementation-level details that are already implied by the approved package and derived planning artifacts.
3. If task details expose a blocker ambiguity in `spec.md` or `contracts.md`, stop and record the blocker. Do not edit `spec.md` or `contracts.md`.
4. Update the `Task details` row in `spec-progress.md`.

### Step 7: Subtasks (`/speckit:subtasks`)

1. Run subtasks to expand complex tasks in `tasks.md` into smaller execution units.
2. Verify that `tasks.md` remains contract-compatible after decomposition.
3. If decomposition exposes canonical ambiguity or conflict, stop and document the blocker.
4. Update the `Subtasks` row in `spec-progress.md`.

### Step 8: Analyze (`/speckit:analyze`)

1. Run analyze in explicit autopilot/non-interactive mode to perform a cross-artifact scan.
2. Review the resulting analysis report.
3. If findings affect only derived artifacts (`research.md`, `plan.md`, `design.md`, `tasks.md`), fix those artifacts and rerun analyze as needed.
4. If findings require changes to `spec.md` or `contracts.md`, stop and record `spec_loop` re-entry or human escalation. Do not edit `spec.md` or `contracts.md`.
5. Update the `Analyze` row in `spec-progress.md` as `passed` or `blocked`, with the report path as evidence.

### Step 9: Final Readiness (`/speckit:readiness`)

1. Run readiness as the final GO/NO-GO implementation gate.
2. Review the readiness report and write/update `readiness.md`.
3. If the verdict is **NO-GO**, stop immediately and report the blockers. Do not repair canonical inputs inside this workflow.
4. If the verdict is **GO**, update the `Final readiness` row in `spec-progress.md` as `go`.

## Completion

Once final readiness returns **GO**, print a concise summary of the derived planning artifacts and confirm that the package is ready for the implementation loop. Mention that `spec.md` and `contracts.md` were adopted without rewrite.
