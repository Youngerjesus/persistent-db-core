---
name: speckit-design
description: Create a detailed technical design document with architecture, flows, invariants, and component boundaries. Use when the user says `speckit.design` or wants the Speckit design phase in Codex.
---

# Speckit Design

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Spec(WHAT)과 Plan(기술 스택/구조)을 바탕으로 **구체적인 기술 설계 문서(TDD)**를 작성하는 단계.
구현 단계에서 "어떻게 만들지?"에 대한 답을 제공.

- **대상 독자**: 구현 AI Agent와 개발자가 코드 작성 시 참조하는 설계 청사진
- **산출물**: `FEATURE_DIR/design.md` — 아키텍처, 컴포넌트 상호작용, 상세 로직 설계
- **역할**: Spec의 WHAT/WHY를 **HOW**로 변환하는 **"설계 번역기"**

You are a **Senior System Architect**. Your goal is to translate a functional specification and rough plan into a concrete **Technical Design Document (TDD)**. Focus on the high-level system interactions and detailed logic flow that are out of scope for the execution-focused specific plan.

## Operating Constraints

**Spec Scope 보호 (절대 금지 사항)**:
- ❌ Spec의 functional requirements를 임의 변경/삭제/축소하지 않음
- ❌ Design 과정에서 새로운 비즈니스 요구사항을 추가하지 않음 (기술적 요구사항만 추가 가능)
- ❌ "기술적으로 불가능"이라는 이유로 요구사항을 사용자 승인 없이 제거하지 않음

**Spec Impact 처리 규칙**:
- 기술적 제약으로 요구사항 조정이 필요한 경우:
  1. `⚠️ Spec Impact` 마커로 표시
  2. 영향받는 spec 요구사항 명시
  3. 대안 제시
  4. **사용자 승인 없이 진행하지 않음**

**허용 범위**:
- ✅ Spec의 WHAT을 구체적인 HOW로 변환
- ✅ 기술적 요구사항 (성능, 보안, 확장성) 관련 설계 결정
- ✅ Plan/Research에서 결정된 기술 스택 기반의 구체적 설계
- ✅ Spec에서 정의하지 않은 기술적 edge case 추가 (error handling, retry 등)

## Execution Steps

### 1. Setup

Run `.specify/scripts/bash/check-prerequisites.sh --json --paths-only` from repo root **once** and parse JSON for FEATURE_DIR and FEATURE_SPEC.
- All file paths must be absolute.
- If JSON parsing fails, abort and instruct user to run `/speckit:specify` first.
- For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

### 2. Load Context

- **Required**: `spec.md` (requirements), `plan.md` (tech stack, data model, contracts)
- **Required**: `research.md` (technical decisions and rationale)
- **Optional**: `.specify/memory/constitution.md` (project principles)
- If `plan.md` or `research.md` is missing, WARN and suggest running `/speckit:plan` or `/speckit:research` first.

### 3. Design Execution

Create or update `FEATURE_DIR/design.md` with the following sections. **Focus on *HOW* it works, not *WHAT* it does.**

#### A. System Architecture (High-Level)
- **Diagram**: Use Mermaid `graph TD` or `C4Context` to show how this feature fits into the existing system.
- **Components**: List modified or new components (Frontend, Backend, DB, External APIs).
- **Dependencies**: Explicitly list libraries, services, or internal modules required.

#### B. Component Interaction (Flow)
- **Sequence Diagrams**: Use Mermaid `sequenceDiagram` to visualize the flow of data and control between components.
    - Example: `User clicks button` -> `Frontend calls API` -> `Backend validates` -> `Backend queries DB`.
- **Data Flow**: Describe how data moves through the layers (Interface -> Application -> Domain -> Infrastructure).

#### C. Detailed Design (Low-Level)
- **Class/Interface Definitions**: Define key classes, interfaces, and public methods.
- **Algorithm Logic**: Use pseudo-code or flowcharts for complex business logic.
- **State Management**: If applicable (e.g., frontend state, backend state machines).
- **Domain-Specific Constraints**: Explicitly define critical domain specific logic (e.g., for AI/LLM: prompt templates & context size limits; for FinTech: precision & transaction boundaries).

#### D. Edges & Errors
- **Error Handling**: How are failures handled specific to this design?
- **Edge Cases**: List specific scenarios (race conditions, empty states, concurrent access).

#### E. Spec Impact (if any)
- 기술적 제약으로 요구사항 조정이 필요한 경우만 기록
- 각 항목에 영향받는 spec 요구사항, 대안, **사용자 승인 필요** 표시

### 4. Self-Review

작성 완료 후 다음 기준으로 자체 검증:

| 기준 | 검증 질문 | PASS/FAIL |
|------|-----------|-----------|
| **Spec Coverage** | Spec의 모든 functional requirement가 design에서 HOW로 매핑되는가? | |
| **Separation of Concerns** | Layered Architecture (Interface/App/Domain/Infra) 분리가 유지되는가? | |
| **Testability** | 모든 외부 의존성을 mock할 수 있는가? | |
| **Security** | AuthN/AuthZ 체크가 필요한 모든 경로에 설계되었는가? | |
| **Spec Integrity** | Spec의 요구사항을 임의로 변경/삭제하지 않았는가? | |

- 하나라도 FAIL이면: 해당 항목을 수정하거나 Spec Impact에 기록
- Coverage gap이 있으면: 누락된 requirement 목록을 리포트에 명시

### 5. Output & Report

- Confirm the design file path (`FEATURE_DIR/design.md`).
- Write the content to the design file.
- Report completion with:
  - Self-Review 결과 (PASS/FAIL per criterion)
  - Spec Coverage: N/M requirements mapped
  - Spec Impact 존재 여부 (있으면 사용자 승인 필요 안내)
  - Design decisions 요약

Adapt it to Codex:
- produce a design artifact that is concrete enough for implementation and tests
- preserve the repository's layered architecture rules
- include important states, interactions, invariants, and edge cases
- cross-check against existing `spec.md`, `research.md`, and `plan.md`
