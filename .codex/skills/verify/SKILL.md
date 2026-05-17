---
name: verify
description: Read-only verification gate for implementation batches. Use when the user says `verify`, `speckit.verify`, or wants the implementation verification gate in Codex.
---

# Verify

## User Input

{{args}}

## Spec Verification Execution

This skill is the read-only implementation verification gate for Phase 2. `speckit-verify` remains a compatibility alias, but the canonical gate name and path are `verify`.

1. Load all available specification documents: `spec.md`, `tasks.md`, `plan.md`, `design.md`, `research.md`, `contracts.md`, `contracts/`, `checklists/`, `development_state.md`, `qa_mapping.md`, and `qa_verdict.md` when present.
2. Delegate to the local `verifier` agent when context isolation helps. If browser-heavy runtime evidence is required, use the repository's browser verification workflow only for read-only evidence collection.
3. Run a balanced verification loop:
   - **Phase 1 (Automated Tests):** Execute the relevant unit, integration, and contract-oriented test commands first.
   - **Phase 2 (Runtime/E2E Checks):** For critical flows or insufficiently covered user paths, verify the actual runtime behavior and browser evidence when required.
   - **QA Mapping Discipline:** Treat `qa_mapping.md` as the canonical task-to-test manifest. If a completed task in `tasks.md` lacks a valid mapping entry, treat that as a verification failure unless there is strong contradictory evidence.
   - **No Code Mutation:** If a failure is found, do not fix code, refactor, or patch tests inside this gate. Record the failure and route back to `sdd-implement-loop`.
4. Write the latest verification report to `specs/[spec-name]/impl_review.md`.
   - Before overwriting the latest report, append the previous contents to `specs/[spec-name]/impl_review.history.md` under a timestamped section.
   - Keep only current open findings in `impl_review.md`. Resolved older findings belong in history, not in the latest file.
5. The report must contain:
   - `Verdict: PASS | FAIL | BLOCK`
   - `Scope`
   - `Executed Checks`
   - `Evidence`
   - `Open Findings`
   - `Repair Targets`
   - `Next Action`
   - `Updated At`
6. Gate semantics:
   - `PASS`: implementation batch is verified and can proceed to `code-review`
   - `FAIL`: implementation is repairable; return to `sdd-implement-loop`
   - `BLOCK`: human decision, approval, or external intervention is required

Adapt it to Codex:
- use the local `verifier` flow when it fits the command path
- do not mutate production code or tests in this skill
- combine automated tests with runtime or browser evidence when needed
- report what passed, what was executed, and exactly what must be repaired next
