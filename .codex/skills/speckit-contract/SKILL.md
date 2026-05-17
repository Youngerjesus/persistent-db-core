---
name: speckit-contract
description: Generate strict exit criteria and implementation contracts for a feature. Use when the user says `speckit.contract` or wants Speckit completion gates that define exactly when implementation is done.
---

# Speckit Contract

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Outline

1. **Setup**: Run `.specify/scripts/bash/check-prerequisites.sh --json` from repo root and parse FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute.

2. **Load design documents**: Read from FEATURE_DIR:
   - **Required**: `spec.md` (business requirements), `plan.md` (tech stack), `tasks.md` (execution steps), `design.md` (architecture).
  - **If present**: existing `contracts.md`, `research.md`, and files under `checklists/`.
  - Note: The contract must ensure that all constraints in these documents are strictly verifiable. `contracts.md` is the single canonical completion contract; do not create or maintain a parallel singular contract file.

3. **Establish Verifiable Exit Criteria (Contracts)**:
   - Your goal is to translate abstract requirements into **physical, executable pass/fail conditions**.
   - **Rule 1. No Subjectivity (환각 방지)**: An AI or human cannot simply say "looks good." The contract must list exact Bash commands, test runner outputs, or CI scripts that must yield `exit code 0` (e.g., `npm run test:e2e`, `pytest -m unit`).
   - **Rule 1a. Command Plus Property**: A command alone is not sufficient. Each contract item must also state what property is being verified and what concrete outcome is expected (for example: response status, payload field, visible UI state, created record, emitted event, or absence of crash/log noise).
   - **Rule 2. Metric-driven**: Define quantitative success metrics (e.g., "All 5 API endpoints in OpenAPI spec return 200 OK under valid payloads", "Lighthouse Performance Score >= 90").
   - **Rule 3. Component Coverage**: Match each major component/user story in `tasks.md` to a specific verification command or test suite.
   - **Rule 4. Quality Gates**: Include static analysis contracts (e.g., "0 High/Critical vulnerabilities in `npm audit`", "ESLint/Ruff generates 0 errors").
   - **Rule 5. E2E Focus With Critical Exceptions**: Require automated E2E tests for the core, successful user scenarios (Happy Paths) defined in `spec.md`. Do NOT mechanically require E2E coverage for every edge case, but do require E2E or runtime verification for any failure path that is business-critical or user-visible, especially around authentication, payment, permissions, quota/capacity enforcement, and irreversible state transitions.
   - **Rule 6. Critical Boundary Contracts**: For each high-value user story, identify the one or two most failure-prone internal or cross-layer boundaries and require at least one explicit verification contract for each selected boundary. Prioritize boundaries involving authentication, authorization, payment/order handoff, API-to-persistence serialization, async enqueue/job creation, error-code propagation, and frontend-to-backend state transitions. Do NOT require contracts for every boundary in the system; focus only on boundaries whose failure would break the user-visible outcome while still passing isolated unit tests.
   - **Rule 7. Real Integration Coverage**: Require at least one real in-repo integration contract for the most important user story in the feature, and require such a contract for any story involving authentication, payment, or other financially or operationally sensitive flows. This must execute against the real integration path between the primary user-facing layer and the backend/runtime layer, not a fully mocked browser test. External third-party systems may still be stubbed or sandboxed when necessary, but the contract must not allow all meaningful internal boundaries to be bypassed by mocks.
   - **Rule 8. Mock Discipline**: Mocking is allowed when it improves determinism or isolates third-party dependencies, but contracts must explicitly state which dependencies are mocked and why. Do NOT use mocked E2E coverage as the only proof for a user story if the real defect could hide in an internal handoff between components owned by this repository.
   - **Rule 9. Evidence Before Completion**: Contracts must be written so that a checkbox can only be marked complete after the cited command or runtime check has actually been executed and its expected outcome observed. Do NOT write contracts that can be checked off by code inspection alone when the requirement concerns runtime behavior.

4. **Generate contracts.md**: You MUST use the file writing tool to directly create and save the generated contract document strictly to the specific path `FEATURE_DIR/contracts.md` (e.g., `specs/[SPEC_NAME]/contracts.md`). Use the following structure, **prioritizing actual functionality and specification fulfillment**. Every single evaluation item MUST be formatted as a strict markdown checkbox (e.g., `- [ ] `) so that system hooks can detect completion status:
   - **# Definition of Done (DoD)**
   - **## 1. Runtime & E2E Contracts (Most Important)**: `- [ ] Specific user flows...`
   - **## 2. Automated Test Contracts**: `- [ ] Exact CLI commands...`
   - **## 3. Architecture Constraints**: `- [ ] Strict bounds defined...`
   - **## 4. Security & Code Quality Contracts**: `- [ ] Linter and auditor...`

5. **Report**: Verify that the file `FEATURE_DIR/contracts.md` was successfully created. Output the path to the file and a brief summary of the hardest exit criteria defined.
  - If a legacy singular contract file exists in the feature directory, explicitly note that it is legacy-only and keep `contracts.md` as the authoritative artifact.

Adapt it to Codex:
- produce objective pass or fail completion criteria
- align contracts with `spec.md`, `plan.md`, `design.md`, and `tasks.md`
- prefer measurable properties over vague completion language
- write the resulting contract artifact into the feature spec area
