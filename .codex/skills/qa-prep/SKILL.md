---
name: qa-prep
description: Prepare canonical QA artifacts before implementation by generating or repairing `qa_mapping.md`, red test scaffolds, and red evidence without touching application core logic.
---

# QA Prep

This skill is the mutable QA preparation workflow for pre-implementation phases such as `qa_prep_exec` and `qa_prep_retry`.
Its job is to close the QA contract before implementation starts, not to ship implementation logic and not to act as the final verification gate.

## Trigger

Use this skill when:

- the user asks for `qa-prep`, `qa_prep`, `qa mapping`, or red test scaffold generation
- the current phase is `qa_prep_exec` or `qa_prep_retry`
- `qa_mapping.md` is missing, incomplete, or needs repair before implementation can continue

Do not use this skill for:

- final QA verdicts or merge gating; that belongs to `qa_prep_verify` or `verify`
- implementation logic changes; that belongs to implementation workflows
- post-implementation code review; that belongs to `code-review`

## Core Contract

- Read the active feature docs first: `spec.md`, `contracts.md`, `tasks.md`, and `plan.md` / `design.md` / `research.md` when present.
- Produce or repair `specs/[spec-name]/qa_mapping.md` as the canonical task-to-test manifest.
- Create or update only QA-oriented files such as tests, fixtures, and helpers needed for the red phase.
- Do not modify app/core implementation files.
- Do not invent interfaces not grounded in the spec, contracts, plan, or design.
- Do not force green. The target state is a usable red QA contract plus evidence.

## Workflow

### 1. Scenario expansion lens

Before writing tests, expand the baseline scenario for each task in `tasks.md`.
Do not stop at the happy path. Pressure-test the task against realistic variations such as:

- invalid input
- empty or partial state
- duplicate or already-done action
- dependency failure or timeout
- permission or trust-boundary behavior
- retry, replay, or interrupted flow

The purpose of this lens is to expose missing verification scenarios that must be reflected in `qa_mapping.md`, not to maximize test count.

### 2. QA generation lens

Translate the chosen scenarios into canonical QA artifacts.

- Inventory every Task ID in `tasks.md`
- Write or update `qa_mapping.md`
- For each task, record:
  - `Status`
  - `Verification Layers`
  - `Test Files`
  - `Preferred Commands`
  - `Task-Scoped Green`
  - optional `Notes`
- Create or repair test scaffolds that match those entries

When details are missing, stop and surface the ambiguity instead of inventing an API or behavior.

### 3. Testing-review lens

Self-review the generated QA artifacts before finishing.
At minimum confirm:

- all Task IDs are covered
- `Preferred Commands` are concrete and runnable
- `Task-Scoped Green` is specific rather than vague
- negative and boundary scenarios are not missing where they materially affect correctness
- the tests are not obviously flaky or assertion-light

This is a self-check lens, not the final gate verdict. The independent verdict still belongs to the verify phase.

### 4. Red evidence

Run the relevant commands and confirm the current state is red for the expected reasons.
Record the evidence in:

- `qa_mapping.md` `Notes` when useful
- the run output / execution log

Do not create a separate canonical artifact just for red evidence unless the caller explicitly requires one.

### 5. Retry re-entry

When re-entering from `qa_prep_retry`:

- treat `qa_prep_review.md` as the latest SSOT for open QA prep defects
- repair the specific gaps first
- then re-run the scenario expansion, QA generation, and testing-review lenses briefly to ensure the fix did not leave other holes

## Completion

This skill is complete when:

- every Task ID has a valid mapping entry
- the required red test scaffolds exist
- `Preferred Commands` and `Task-Scoped Green` are defined per task
- red evidence exists
- no application core implementation files were modified
