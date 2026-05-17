# Test Layer Selection

Pick the cheapest layer that can reproduce the failure honestly.

## Default Order

1. Unit test
2. Integration or API test
3. Browser or end-to-end reproduction
4. One-off reproduction script or targeted command

Move downward only when the layer above cannot represent the real failure mode.

## Choose Unit Test When

- The bug is in pure logic, transformation, parsing, validation, or branching
- Inputs and outputs are enough to expose the failure
- External systems can be mocked without hiding the defect

## Choose Integration or API Test When

- The bug depends on database behavior, request wiring, service interaction, or serialization boundaries
- The failure involves multiple internal components but not the full UI flow
- A unit test would fake too much of the real behavior

## Choose Browser or E2E Reproduction When

- The bug depends on real UI state, timing, navigation, form behavior, or client-server interaction
- The defect only appears with actual rendering, browser APIs, or network choreography
- Lower layers pass but the user-visible flow still fails

## Choose One-Off Script or Targeted Command When

- There is no practical automated test harness yet
- The issue is environmental, operational, or toolchain-specific
- The fastest honest reproduction is a script, shell command, or minimal runtime probe

This is acceptable only if:
- The script is narrowly scoped
- The command clearly proves the failure
- Follow-up regression coverage is added later when realistic

## Escalation Rules

- If a unit test reproduces the issue, do not jump to E2E first
- If lower layers cannot reproduce the failure, escalate instead of over-mocking
- If the root cause is still unclear after reproducing at one layer, add boundary evidence before escalating blindly

## Output

When reporting the chosen layer, include:
- Why this layer is the cheapest honest reproduction
- What it proves
- What it still does not prove
