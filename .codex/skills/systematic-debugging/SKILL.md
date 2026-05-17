---
name: systematic-debugging
description: Use when encountering any bug, test failure, or unexpected behavior, before proposing fixes
---

# Systematic Debugging

reference @./references/phase-gates.md
reference @./references/test-layer-selection.md
reference @./references/debug-report-template.md
reference @./root-cause-tracing.md
reference @./defense-in-depth.md
reference @./condition-based-waiting.md

## Overview

Random fixes waste time and create new bugs. Quick patches mask underlying issues.

**Core principle:** ALWAYS find root cause before attempting fixes. Symptom fixes are failure.

**Violating the letter of this process is violating the spirit of debugging.**

## The Iron Law

```text
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

If you have not completed Phase 1, you cannot propose fixes.

## Use When

Use for any technical issue:
- Test failures
- Bugs in production
- Unexpected behavior
- Performance problems
- Build failures
- Integration issues

Use this especially when:
- You are under time pressure
- A quick fix seems obvious
- You already tried multiple fixes
- A previous fix did not work
- You do not fully understand the issue

Do not skip the process because:
- The issue seems simple
- You are in a hurry
- Someone wants it fixed immediately

## The Four Phases

You must complete each phase before proceeding to the next. For detailed gates, use `references/phase-gates.md`.

### Phase 1: Root Cause Investigation

Before attempting any fix:
- Read the full error message, stack trace, warning, and failing assertion
- Reproduce the issue consistently or gather enough data to explain why you cannot
- Check recent changes, config differences, and environmental differences
- In multi-component flows, gather evidence at each boundary to locate where behavior diverges
- Trace data backward to find the original trigger instead of fixing where the error surfaced

Phase 1 is complete only when you can name:
- The observed symptom
- The expected behavior
- The most likely failing boundary, component, or assumption
- The evidence that supports that conclusion

If you cannot do that, you are still investigating.

### Phase 2: Pattern Analysis

Find the pattern before fixing:
- Locate similar working code in the same codebase when possible
- Compare broken and working paths completely, not selectively
- Identify all meaningful differences in code, config, inputs, environment, and execution order
- Check whether the issue is tied to a contract, configuration expectation, data assumption, or integration boundary

Do not assume the issue is only a spec mismatch or only an implementation defect. It may also be:
- A test defect
- Environment or config drift
- Missing observability
- Bad data
- External dependency behavior

### Phase 3: Hypothesis and Testing

Use the scientific method:
- Form one explicit hypothesis: `I think X is the root cause because Y`
- Define one minimal test or experiment for that hypothesis
- Define the expected observation before running it
- Run the smallest experiment that can confirm or reject the hypothesis

If the hypothesis fails:
- Do not stack additional fixes on top
- Return to Phase 1 with the new evidence

If you do not understand the problem yet:
- Say so directly
- Gather more evidence
- Ask for help only after documenting what you checked

### Phase 4: Implementation

Fix the root cause, not the symptom:
- Choose the lowest-cost reproduction layer that can prove the bug
- Create a failing reproduction before changing production code
- Prefer automated tests, but use a one-off script, targeted command, or browser reproduction when needed
- Implement one root-cause fix at a time
- Verify the reproduction now passes and relevant broader checks still pass

Use `@tdd-workflow` when the change should be driven by an automated failing test.

If repeated fixes fail:
- Return to investigation immediately
- Prioritize the failure pattern over the raw attempt count
- If multiple fixes reveal new issues in different places, stop and question the architecture

## Root Cause Definition

`Root cause identified` does not mean `I found where it crashed`.

It means you can explain:
- What input, state, ordering, assumption, or boundary is wrong
- Why it became wrong
- What evidence proves that explanation better than competing explanations

If you cannot do that, you have a symptom, not a root cause.

## Debugging Output Contract

While debugging, communicate in this shape:
- `Symptom:` what is failing
- `Expected:` what should happen
- `Reproduced:` yes, no, or partial
- `Evidence:` logs, traces, diffs, or comparisons gathered so far
- `Current hypothesis:` one candidate root cause
- `Next step:` one experiment or verification action
- `Fix gate:` blocked until root cause is evidenced

Use `references/debug-report-template.md` for the exact template.

## Project Integration

When used in this repository:
- Check spec artifacts only when the issue is actually connected to a feature spec or contract
- If relevant, start from the direct SSOT for that scope: `spec.md`, `contracts.md`, `tasks.md`, or the active phase review document
- If not spec-related, start from code, runtime behavior, logs, config, and recent diffs
- Do not force every issue into `spec mismatch` or `implementation bug`
- Use the categories in `references/debug-report-template.md` to keep classification honest

Repository-specific operating rules:
- When adding diagnostic instrumentation, avoid leaking secrets or PII
- Remove temporary instrumentation after the investigation unless it should become permanent observability
- Prefer parallel evidence gathering for search, working-example comparison, and environment diffing when collaboration tools are available
- Do not move into implementation until the evidence is strong enough that another engineer could understand why the fix is justified

## Red Flags

If you catch yourself thinking any of these, stop and return to Phase 1:
- Quick fix for now, investigate later
- Just try changing X and see if it works
- Add multiple changes, then run tests
- Skip the test, manual verification is enough
- It is probably X, let me fix that
- I do not fully understand but this might work
- One more fix attempt
- The issue must be in the code I touched most recently

## Trivial-Issue Exception

For obvious syntax, import, or wiring mistakes, you may compress the process.

Even then, do not skip:
- Reading the actual error
- Verifying the precise cause
- Running minimum verification after the fix

## Supporting Techniques

Available supporting documents in this directory:
- `references/phase-gates.md`
- `references/test-layer-selection.md`
- `references/debug-report-template.md`
- `root-cause-tracing.md`
- `defense-in-depth.md`
- `condition-based-waiting.md`

## Outcome

The goal of this skill is not to make debugging slower.

The goal is to prevent guess-driven edits, force evidence before fixes, and make it obvious when the problem is still not understood.
