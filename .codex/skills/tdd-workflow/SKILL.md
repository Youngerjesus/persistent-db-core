---
name: tdd-workflow
description: Use this skill for most bounded implementation, bugfix, or behavior changes. Prefer failing-test-first TDD, but apply it pragmatically.
---

# Test-Driven Development Workflow

reference @./references/tdd-rules.md
reference @./references/verification-checklist.md

This skill is the default workflow for bounded implementation work. Use it when the next step is a small or medium code change that should be driven or constrained by automated verification, even if a bit of context gathering is still needed before the first test.

## Use When

- Implementing a bounded feature or bugfix
- Changing observable behavior
- Adding regression coverage before or during refactoring
- Converting a bug report into a reproducible failing test
- Making a small or medium code change where tests should lead or tightly constrain the implementation

## Do Not Use

- Writing or clarifying product specs
- Large multi-milestone orchestration with no clear next implementation slice
- Pure documentation changes
- Configuration-only changes with no meaningful automated verification path
- Refactors with no observable behavior and no realistic regression coverage to add

## Workflow

1. State the target behavior and success condition in one sentence.
2. Load only the minimum code context needed to write the next test.
3. Write one test for one behavior using real code paths whenever possible.
4. Run that test and confirm it fails for the expected reason.
5. Write the minimum production code needed to make that test pass.
6. Re-run the targeted test, then the relevant broader suite, and keep the output clean.
7. Refactor only after green, while preserving passing tests.
8. Report proof that red was observed, green was observed, and broader verification ran.

## Non-Negotiables

- No production code before a failing test.
- If the test passes immediately, the test is wrong or already-covered behavior; fix the test before coding.
- Do not keep prewritten implementation as reference while writing the test.
- Prefer behavior tests over mock-heavy implementation tests.
- Do not expand scope beyond the current failing test.
