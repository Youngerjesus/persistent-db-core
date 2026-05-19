---
name: code-review
description: Strict pre-merge multi-agent review protocol for correctness, regressions, completeness, security, performance, maintainability, and additive bias. Use when the user asks for a code review before merging or committing changes.
license: Complete terms in LICENSE.txt
---

# Code Review Skill

This skill is a merge-gate review protocol. It decides review scope from the diff, routes the right specialist reviewers, and compiles one decision-ready report.

## Review Standard

Use this skill for pre-merge or pre-commit review. The goal is not general advice. The goal is to catch real merge blockers, structural regressions, and missing completeness work before code lands.

Core rules:
- Review the actual diff first: `git diff` and `git diff --staged`.
- Read full changed files when a finding depends on surrounding logic. Do not review only hunk fragments.
- Flag only real problems. Do not pad the review with style nits or speculative cleanup.
- If the diff introduces a new enum value, status string, type discriminator, route shape, or schema field, trace consumer completeness outside the diff before concluding it is safe.
- Findings must include file references and a concrete fix direction when possible.

## Workflow

When requested to review code, follow this sequence:

1. **Establish scope**
   - Inspect `git diff` and `git diff --staged`.
   - If the user names files or a PR scope, constrain review to that scope.
   - Classify the diff:
     - Backend or domain logic
     - API contract or runtime boundary
     - Database or migration
     - Frontend UI or interaction
     - Auth, payments, or other trust-boundary code

2. **Run Pass 1: Merge blockers first**
   - Start with `code-reviewer` for correctness, regressions, missing tests, spec mismatch, and completeness.
   - Always run `testing-reviewer` for test gaps, edge cases, flaky patterns, and merge confidence.
   - Always run `security-reviewer` when the diff touches input boundaries, auth, external calls, secrets, uploads, payments, or sensitive data.
   - Always run `maintainability-reviewer`.
   - Always run `red-team-reviewer`.

3. **Run Pass 2: Scope-based specialists**
   - Run `performance-reviewer` for backend/frontend code with query, rendering, async, or bundle risk.
   - Run `api-reviewer` for endpoint, DTO, schema, contract, transport, or error-shape changes.
   - Run `database-reviewer` for SQL, migrations, schema, indexes, transactions, queue workers, or ORM query changes.
   - Run `ui-ux-reviewer` for component, layout, interaction, accessibility, motion, or visualization changes.

4. **Deduplicate and merge**
   - Remove duplicate findings across specialists.
   - If multiple reviewers hit the same issue, keep the strongest owner and fold supporting context into one finding.
   - Order findings by merge risk, not by reviewer order.

5. **Return one unified report**
   - Findings first.
   - Summary second.
   - Verdict last: `PASS`, `FAIL`, or `BLOCK`.

## Specialist Routing

Use these local agent contracts:
- `.codex/agents/code-reviewer.toml`
- `.codex/agents/testing-reviewer.toml`
- `.codex/agents/security-reviewer.toml`
- `.codex/agents/maintainability-reviewer.toml`
- `.codex/agents/red-team-reviewer.toml`
- `.codex/agents/performance-reviewer.toml`
- `.codex/agents/api-reviewer.toml`
- `.codex/agents/database-reviewer.toml`
- `.codex/agents/ui-ux-reviewer.toml`

Recommended routing defaults:
- `code-reviewer`: always
- `testing-reviewer`: always
- `maintainability-reviewer`: always
- `red-team-reviewer`: always
- `security-reviewer`: auth, runtime boundary, input handling, external network/file/process, payment, secret, upload, webhook, admin flow
- `performance-reviewer`: loops over data, queries, async code, rendering logic, charting, state management, large list views
- `api-reviewer`: route handlers, request/response schemas, status codes, error envelope, contract changes
- `database-reviewer`: migrations, SQL, indexes, transactions, RLS, pagination, ORM query shape
- `ui-ux-reviewer`: components, CSS, layout, motion, a11y, mobile, chart interactions

## Severity Policy

- `BLOCK`: correctness bug, regression risk, trust-boundary security issue, missing completeness on new enum/status/schema, broken contract, destructive migration risk
- `WARNING`: maintainability, performance, UX, or test gap that should be fixed before merge but is not clearly catastrophic
- `INFO`: non-blocking follow-up worth tracking

Final verdict mapping:
- `PASS`: no open merge findings remain.
- `FAIL`: one or more open `WARNING` or `INFO` findings must be addressed or explicitly accepted before merge.
- `BLOCK`: one or more open `BLOCK` findings or human/blocking decisions prevent merge.

Tag each finding with one action class:
- `NEEDS JUDGMENT`: not mechanical; human or implementer decision required
- `AUTO-FIXABLE`: concrete and mechanical enough to fix directly in a follow-up implementation step
- `FOLLOW-UP TEST GAP`: the code might be right, but merge confidence is weak because coverage is missing

## Suppressions

Do not flag:
- Harmless redundancy that improves readability
- Consistency-only rewrites with no concrete risk
- Hypothetical performance issues without a visible hot path
- Security issues without an actual trust boundary in the diff
- Problems already fixed in the same diff
- Generic "add comments" advice

## Ownership Rules

- `code-reviewer` owns correctness, regressions, completeness, and spec mismatch.
- `testing-reviewer` owns generic coverage gaps, negative-path omissions, edge cases, flaky patterns, and isolation issues.
- `api-reviewer` owns wire contracts, backward compatibility, versioning, and docs drift.
- `database-reviewer` owns migration safety, lock risk, rollout order, and deploy compatibility.
- `red-team-reviewer` owns additive bias, silent failure suspicion, and cross-category integration gaps.
- If multiple reviewers identify the same root issue, keep one finding under the most direct owner and merge the supporting rationale.

## Unified Output Format

```markdown
# Comprehensive Code Review Report

## Findings
- [BLOCK][NEEDS JUDGMENT][Correctness] `path/to/file.ts:42`
  Problem: ...
  Fix: ...

- [WARNING][FOLLOW-UP TEST GAP][Testing] `path/to/test.ts:1`
  Problem: ...
  Fix: ...

## Category Summary
| Category | Status | Notes |
|---|---|---|
| Correctness & Testing | BLOCK | 2 blockers |
| Testing | WARNING | 1 coverage gap |
| Security | PASS | 0 findings |
| API / Database | WARNING | 1 contract risk |
| Maintainability | WARNING | 1 issue |
| Performance | PASS | 0 findings |
| UI / UX | PASS | Not triggered |
| Red Team | WARNING | 1 simplification required |

## Residual Risks
- ...

**Verdict:** BLOCK
```

Use `WARNING` and `INFO` only as finding or category status labels, not as the final report verdict. If there are no findings, say that explicitly and still mention any unreviewed areas or missing runtime verification.
