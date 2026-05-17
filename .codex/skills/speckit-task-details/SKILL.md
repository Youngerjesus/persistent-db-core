---
name: speckit-task-details
description: Enrich `tasks.md` with precise technical details and targeted clarification questions. Use when the user says `speckit.task-details` or wants the Speckit task-detailing step in Codex.
---

# Speckit Task Details

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

If the user input indicates `autopilot`, `non-interactive`, `sdd-autopilot`, or an equivalent instruction to avoid user interaction, switch to **Autopilot Mode**:
- Do not emit user-facing clarification questions.
- Resolve technical ambiguities through `architect` consultation when defaults would be risky.
- Apply the resolved decision directly to `tasks.md` rather than waiting for a reply.

## Context

1. **Setup**: Run `.specify/scripts/bash/check-prerequisites.sh --json` from repo root and parse FEATURE_DIR.
2. **Load documents**: Read `spec.md`, `research.md`, `tasks.md`, `design.md`, and `plan.md` from the FEATURE_DIR. If they don't exist, ERROR.

## Objective

Your goal is to take the existing `tasks.md` and inject **explicit, precise technical details** into each task. AI agents perform poorly with vague instructions. You must transform high-level commands into rigorous technical specifications. If any requirements remain ambiguous or are missing from the design documents, you must formulate targeted questions for the user.

**CRITICAL CONSTRAINT:** You MUST NOT alter the existing Task ID structure, the parallel `[P]` markers, the Story `[US#]` labels, or the primary description line of the original tasks. You will ONLY append indented detailed descriptions beneath each task.

## The Detail Injection Rules

Scan every task in `tasks.md` and apply the following strategies:

**Strategy 1: Add Precision and Exactness (구현의 정확성 요구)**
Transform general instructions into concrete technical requirements.
- *Bad*: "Implement authentication system"
- *Good*: "Implement JWT authentication system using bcrypt-12 password hashing, RS256 signature, and 7-day refresh token rotation."
Extract these exact details from `design.md`, `plan.md`, or `spec.md`. If not explicitly defined, propose a sensible and robust default based on industry best practices, but mark it clearly.

**Strategy 2: Identify and Expose Ambiguities (모호함 제거 및 질문)**
If a task is too abstract and context is insufficient to make a safe technical assumption, you MUST surface the ambiguity explicitly. In interactive mode, ask the user. In Autopilot Mode, consult the `architect` agent instead of guessing blindly. Avoid "fill-in-the-blank" situations for the next AI agent.

### Formatting Example

**Original Task (in tasks.md):**
`- [ ] T001 [US1] Implement authentication middleware in src/middleware/auth.py`

**Your Modified Output:**
```markdown
- [ ] T001 [US1] Implement authentication middleware in src/middleware/auth.py
  - **Details**: Implement JWT validation using `PyJWT`. Token secret must be loaded from `AUTH_SECRET_KEY` env var. Validate token expiration strictly. If expired, return 401 Unauthorized with a specific error code `TOKEN_EXPIRED`.
  - **Questions**: Should we implement rate limiting on the authentication endpoint (e.g., 5 attempts / min) in this phase?
```

## Workflow

1. Read the current `tasks.md`.
2. Cross-reference the tasks with ALL loaded documents (`spec.md`, `research.md`, `plan.md`, `design.md`) to extract specific technical constraints, libraries, and architectural decisions.
3. Rewrite `tasks.md` in place using the file writing tool. For every task, insert indented `- **Details**: ...` sub-bullets. If there are uncertainties, add a `- **Questions**: ...` sub-bullet only in interactive mode; in Autopilot Mode, resolve them through `architect` and encode the chosen answer into the details directly.
4. **Report**: Verify the file was updated. Output a summary listing the key technical constraints added. In interactive mode, explicitly list all generated questions. In Autopilot Mode, explicitly list which ambiguities were resolved via `architect`.

## Success Criteria

- **Coverage**: 모든 task에 `- **Details**` 서브불릿이 추가됨 (단순 setup task 제외)
- **Questions Gate**: `- **Questions**`가 존재하면 사용자에게 응답을 요청하고, 응답을 반영한 후 해당 task의 Details를 업데이트. Autopilot Mode에서는 질문을 남기지 않고 `architect` 결정으로 닫는다.
- **Precision Check**: Details에 "적절한", "필요에 따라" 같은 모호한 표현이 없어야 함. 구체적 수치/이름/설정값을 포함해야 함

Adapt it to Codex:
- preserve the original task line and append details beneath it
- remove vague phrasing and inject concrete technical constraints
- ask targeted questions when a safe default would be risky, unless Autopilot Mode delegates that decision to `architect`
- report every open question clearly to the user in interactive mode, or every agent-resolved ambiguity in Autopilot Mode
