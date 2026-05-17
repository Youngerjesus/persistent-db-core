---
name: sdd-implement-loop
description: Orchestrates the Phase 2 Spec-Driven Development (SDD) implementation and repair loop. Use this skill when Phase 1 specifications are ready and the next job is to generate tests, implement tasks, and repair failures reported by verification or review gates.
license: Complete terms in LICENSE.txt
---

# SDD Implementation Loop Workflow

This skill automates the implementation and repair portion of Phase 2. Your goal is to act as the central implementation engine: generate the task-to-test contract up front, implement tasks, compress context, and accept failure feedback from the separate `verify` and `code-review` gates.

## Core Principle: Unbreakable Loop & Context Management

You must orchestrate the following workflow in a strict loop until all implementation tasks are complete and the feature is ready for the external verification and review gates.

- **Defend against Context Dilution**: The AI's context window is precious. Do not carry over long chat histories between steps. Rely heavily on `development_state.md` to pass only the highest-density information (completed tasks, current blockers, next steps) to the next agent execution.
- **Repair From Gate Feedback**: If the separate `verify` or `code-review` gates fail, consume their latest report files and route the workflow back to the `implement` step with precise repair targets.

## Execution Sequence

### Step 1: Context Injection (Initialization)
1. Read the core specification documents: `spec.md`, `plan.md`, `tasks.md`, `design.md`, `research.md`.
2. Ensure you understand the overall architecture, tech stack constraints (from `plan.md` and `design.md`), and the specific properties to satisfy (from `spec.md`).

### Step 2: Write All Tests Upfront (QA Prep - Batch Generation)
1. **Goal**: Before any implementation logic is written, generate the **entire suite of tests** covering ALL tasks defined in `tasks.md`, and produce the canonical task-to-test mapping manifest `specs/[spec-name]/qa_mapping.md`.
2. **Execution**: Use the `qa-prep` skill as the canonical QA preparation workflow. Apply it with the `Scenario expansion lens`, `QA generation lens`, and `Testing-review lens` in that order.
3. **Constraint Check**: You ONLY write test code. DO NOT write or modify application core logic files yet.
4. Base your imports, class names, method names, and payloads strictly on `design.md`, `plan.md`, and `contracts/`. If details are missing, STOP and route back to the planning/specification loop to close the ambiguity before writing more tests. Do not invent new interfaces in this implementation phase.
5. Execute the test suite (e.g., `pytest` or `npx playwright test`) to confirm that **all** generated tests currently fail (Red phase of TDD) due to missing implementation or import errors.
6. In `qa_mapping.md`, define for every Task ID:
   - `Status`
   - `Verification Layers`
   - `Test Files`
   - `Preferred Commands`
   - `Task-Scoped Green`
   - optional `Notes`
7. Once the failing tests and `qa_mapping.md` are generated, proceed to the Iterative Implementation Loop (Step 3).

---
**ITERATIVE IMPLEMENTATION LOOP STARTS HERE**
Perform Steps 3 through 7 iteratively. If a step fails, go back to Step 3 (Implement).

### Step 3: Implement (Blue Team)
1. Identify the first (or next incomplete) task in `tasks.md`.
2. Read the matching Task ID entry in `specs/[spec-name]/qa_mapping.md` before writing or changing code. If the current task has no mapping entry, STOP and repair the QA mapping first.
3. Execute the implementation for this target task using the `@staff-engineer` agent.
4. **Constraint Check**: Write the minimum amount of code necessary to make the previously written failing tests for *this specific task* pass. Do not invent new interfaces; follow what is defined in the tests, `qa_mapping.md`, and `design.md`.
5. A task is only “task-scoped green” when the current task’s `Preferred Commands` and `Task-Scoped Green` conditions in `qa_mapping.md` are satisfied.
6. Once the code is written and the current task is task-scoped green, proceed to Step 4.

### Step 4: Compress (State Management)
1. Update the `development_state.md` file located inside the feature's `specs/[spec-name]/` directory. (This ensures state is tracked per feature, making it easily reviewable by others).
2. Summarize the exact actions just taken:
   - What feature/task was implemented?
   - What were the key technical decisions made just now?
   - Are there any known blockers or remaining test failures?
   - Was this a fresh implementation pass or a repair pass driven by `impl_review.md` or `code_review.md`?

### Step 5: Exit Condition Check
1. Manually evaluate the physical contract files (`specs/[spec-name]/contracts.md`, `specs/[spec-name]/checklists/`).
2. Verify that all tasks in `tasks.md` are marked as complete (`[x]`).
3. Ensure that all technical requirements and acceptance criteria have been satisfied without compromise.
4. **Outcome**:
   - **INCOMPLETE**: If there are remaining `[ ]` tasks in `tasks.md` or any unfulfilled contract items, update `specs/[spec-name]/development_state.md` with the next target task, and **Return to Step 3 (Implement)** for the next task.
   - **READY FOR GATES**: If all tasks are checked off and all contracts/checklists are fulfilled from the implementation side, stop this skill and hand off to the separate `verify` gate, then the separate `code-review` gate.

## Gate Feedback Re-entry

This skill is also the canonical repair engine for Phase 2.

- `verify` writes the latest implementation verification report to `specs/[spec-name]/impl_review.md`.
- `code-review` writes the latest review report to `specs/[spec-name]/code_review.md`.
- Both reports use `PASS | FAIL | BLOCK` verdicts and keep prior versions in matching `.history.md` files.
- On `FAIL`, the scheduler or caller must rerun `sdd-implement-loop` and point it at the latest gate report.
- On re-entry, read the latest report first, extract `Repair Targets` or `Must Fix Now`, update `development_state.md`, and repair the highest-priority item before resuming normal task progression.
- On `BLOCK`, stop and require human intervention. Do not continue implementing around the blocker.

## Completion
Once the Exit Condition Check verifies implementation completeness, notify the caller that the feature is ready for the external `verify` and `code-review` gates. Do not declare the feature done until those gates pass.
