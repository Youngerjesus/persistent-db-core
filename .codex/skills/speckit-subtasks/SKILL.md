---
name: speckit-subtasks
description: Expand complex tasks in `tasks.md` into actionable subtasks without breaking the existing task structure. Use when the user says `speckit.subtasks` or wants the Speckit subtask decomposition step in Codex.
---

# Speckit Subtasks

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Context

1. **Setup**: Run `.specify/scripts/bash/check-prerequisites.sh --json` from repo root and parse FEATURE_DIR.
2. **Load documents**: Read `spec.md`, `research.md`, `tasks.md`, `design.md`, and `plan.md` from the FEATURE_DIR. If they don't exist, ERROR.

## Objective

Your goal is to take the existing `tasks.md` and identify **"Fat Tasks"** (monolithic tasks that hide complex domain logic, AI agent interactions, complex state/memory management, or intricate rule engines) and break them down into granular, actionable subtasks.

**CRITICAL CONSTRAINT:** You MUST NOT alter the existing Task ID structure, the parallel `[P]` markers, the Story `[US#]` labels, or the primary description line of the original tasks. You will ONLY append indented subtasks beneath the complex tasks you identify.

## The Subtask Injection Rule

Scan every task in `tasks.md` and apply the following two breakdown strategies where appropriate:

**Strategy 1: Conditional Unwrapping (유연성 부여)**
If a task uses generic groupings or raw counts (e.g., "Implement 3 sub-agents", "Create the database tables") and the entities have **distinct, complex domain logic**, you MUST unwrap them into specific explicitly named subtasks based on the design documents.
**However**, if the entities share the exact same structure and can be implemented elegantly via a single Factory pattern, a loop, or generic shared functions, **DO NOT unwrap them**. Instead, write a subtask to implement the abstract/factory pattern that handles the group.

**Strategy 2: 3-Tier Architectural Breakdown**
For any task (or unwrapped specific entity) that involves:
- Complex rule engines, algorithmic calculations, or data pipelines
- Tricky state, context, session, or memory management
- External API integrations with side effects (like payments, LLMs, or third-party services)
- High-risk security or compliance boundaries

You MUST inject nested subtasks following this strict 3-tier architectural breakdown:
1.  **`config:`** Define the exact static rules, algorithms, policies, templates, or system configurations.
2.  **`state:`** Build the logic for preparing inputs, managing memory/state/session, context tracking, or handling boundaries.
3.  **`execution:`** The actual execution logic, API call, or engine triggering that integrates the `config` with the `state`.

### Formatting Example 1: Unwrapping Grouped Entities

**Original Task (in tasks.md):**
`- [ ] T008 [US2] Implement 3 Sub-Agents for 오행 진단`

**Your Modified Output:**
```markdown
- [ ] T008 [US2] Implement 3 Sub-Agents for 오행 진단
  - [ ] Implement '과다 오행' (Excess Element) Sub-Agent
  - [ ] Implement '부족 오행' (Deficient Element) Sub-Agent
  - [ ] Implement '기구구 오행' (Harmful Element) Sub-Agent
```

### Formatting Example 2: 3-Tier Breakdown

**Original Task (in tasks.md):**
`- [ ] T016 [US2] Implement Checkout and Payment Gateway Integration...`

**Your Modified Output:**
```markdown
- [ ] T016 [US2] Implement Checkout and Payment Gateway Integration...
  - [ ] config: 결제 수단별 허용 정책(Refund rules, Rate limits) 및 결제 PG사 API 스펙(Credentials, Endpoints) 정의
  - [ ] state: Redis를 활용한 결제 진행 중 상태(Pending) Lock 관리 및 멱등성(Idempotency) 키 생성 로직 구현
  - [ ] execution: PG사 결제 승인 API 실제 호출 및 예외(Timeout, Network Error) 발생 시 자동 환불(Fallback) 처리 연동
```

## Workflow

1.  Read the current `tasks.md`.
2.  Cross-reference the tasks with ALL loaded documents (`spec.md`, `research.md`, `plan.md`, `design.md`) to spot where the real technical complexity lies (e.g., hidden business rules, complex state machines, performance bottlenecks).
3.  Rewrite `tasks.md` in place using the file writing tool. Leave simple tasks (like "Create User model" or "Setup route") completely untouched. ONLY expand the complex tasks by inserting indented `- [ ]` subtasks below them.
4.  **Self-Validation**: Before reporting, verify each expanded task:
    - 3-Tier tasks (config/state/execution): 세 tier가 모두 존재하는지 확인. 하나라도 누락 시 보완.
    - Unwrapped tasks: 각 subtask가 독립적으로 구현 가능한지 확인 (다른 subtask에 대한 암묵적 의존성 없어야 함).
    - 단순 task가 잘못 확장되지 않았는지 확인 (Factory 패턴으로 해결 가능한 것은 합치기).

5.  **Report**: Verify the file was updated. Output a summary listing which Task IDs (e.g., T015, T016) were expanded, and briefly explain why they were flagged as complex.

Adapt it to Codex:
- preserve task IDs, story labels, and parallel markers
- add only indented subtasks under complex tasks
- use the config, state, execution split when the original rule calls for it
- summarize which tasks were expanded and why
