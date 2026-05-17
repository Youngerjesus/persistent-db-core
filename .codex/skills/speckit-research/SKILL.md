---
name: speckit-research
description: Resolve technical ambiguities and design choices before planning. Use when the user says `speckit.research` or wants the Speckit research phase in Codex.
---

# Speckit Research

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Spec에서 발견된 **기술적 미지(unknowns)를 조사하고 의사결정**하여 `research.md`에 기록하는 단계.
Plan/Design 단계가 기술적 근거 없이 진행되는 것을 방지.

- **대상 독자**: `/speckit:plan` 및 `/speckit:design` 단계의 AI Agent가 기술적 의사결정의 근거 문서로 사용
- **산출물**: `research.md` — 모든 기술적 의사결정과 그 근거, 대안 비교를 담은 문서
- **역할**: Spec(WHAT/WHY)과 Plan/Design(HOW) 사이의 **기술적 다리(bridge)** 역할

## Operating Constraints

**Spec Scope 보호 (절대 금지 사항)**:
- ❌ Spec에서 정의한 기능 범위(WHAT/WHY)를 변경하지 않음
- ❌ "기술적으로 어렵다"는 이유로 요구사항을 임의 축소/제거하지 않음
- ❌ 근거 없는 기술 선택 금지 (감이나 선호도로 결정하지 않음)
- ❌ 하나의 옵션만 조사하고 "이것이 최선"이라고 결론내지 않음

**허용 범위**:
- ✅ 기술적 제약으로 요구사항 조정이 필요한 경우: `## Spec Impact` 섹션에 기록하고 **사용자 승인 필요** 명시
- ✅ 각 의사결정에 최소 2개 대안 비교 (단, 명백한 업계 표준은 1개로 충분 — 근거 명시)
- ✅ 외부 도구/API/라이브러리에 대한 구체적 조사 및 추천
- ✅ constitution.md의 원칙과 충돌하는 기술 선택지 필터링

## Execution Steps

### 1. Setup

Run `.specify/scripts/bash/check-prerequisites.sh --json --paths-only` from repo root **once** and parse JSON for FEATURE_DIR and FEATURE_SPEC.
- All file paths must be absolute.
- If JSON parsing fails, abort and instruct user to run `/speckit:specify` first.
- For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

### 2. Load Context

- **Required**: Read FEATURE_SPEC (`spec.md`)
- **Optional**: `.specify/memory/constitution.md` (project principles for filtering decisions)
- **Optional**: `checklists/` (if exist, check for flagged technical gaps)

### 3. Extract Research Topics

Analyze the feature specification systematically:

1. **Technical Unknowns**: 기술 스택, 라이브러리, 프레임워크 선택이 필요한 영역
2. **External Dependencies**: API 연동, 서드파티 서비스, 인증 방식
3. **Domain-Specific Constraints**: 해당 도메인의 기술적 제약 (e.g., AI: 모델 선택/토큰 제한/프롬프트 전략; FinTech: 정밀도/트랜잭션 경계)
4. **"NEEDS CLARIFICATION" 기술 항목**: Spec에서 기술적 결정이 필요하다고 표시된 항목
5. **Architecture Patterns**: 확장성, 성능, 보안 요구사항을 만족하는 아키텍처 패턴

각 topic에 대해 **연구 과제(Research Task)**를 정의.

### 4. Conduct Research

각 Research Task에 대해:

1. **Options Survey**: 최소 2개 대안 조사 (명백한 업계 표준 제외)
2. **Comparison Criteria**: 다음 기준으로 비교
   - Performance / Scalability
   - Security / Compliance
   - Developer Experience / Learning Curve
   - Cost (운영 비용, 라이센스)
   - Community / Maintenance status
3. **Decision**: 최적 옵션 선택 + 근거
4. **Risk Note**: 선택에 따르는 리스크 (있을 경우)

### 5. Consolidate & Write

`FEATURE_DIR/research.md`에 다음 구조로 작성:

```markdown
# Technical Research: [Feature Name]

**Purpose**: Plan/Design 단계에서 기술적 의사결정의 근거 문서로 사용
**Created**: [DATE]
**Status**: [Complete / Partial — N개 미해결]

## Research Decisions

### RD-001: [Topic]
- **Question**: [What needed to be decided]
- **Decision**: [What was chosen]
- **Rationale**: [Why, based on comparison criteria]
- **Alternatives Considered**:
  | Option | Pros | Cons | Verdict |
  |--------|------|------|---------|
  | A      | ...  | ...  | Selected |
  | B      | ...  | ...  | Rejected — [reason] |
- **Risk**: [Any risk from this decision]

## Spec Impact (if any)
- [Spec 요구사항 변경이 필요한 경우만 기록]
- 각 항목에 **사용자 승인 필요** 표시

## Unresolved Items
- [아직 결정되지 않은 항목 — 0개여야 COMPLETE]
```

### 6. Validate & Report

**Success Criteria** (모두 충족해야 COMPLETE):
- [ ] Spec의 모든 기술적 NEEDS CLARIFICATION 항목이 Decision으로 전환됨
- [ ] 각 Decision에 Rationale + 최소 1개 Alternative 기록 (업계 표준 예외 시 근거 명시)
- [ ] `## Unresolved Items` 섹션이 비어있음
- [ ] Spec Impact 항목이 있으면 사용자에게 승인 요청 안내

**Validation failure**: 위 기준 미충족 시 ERROR. 사용자에게 미해결 항목을 보고하고 추가 리서치 또는 의사결정을 요청.

**Report**: `research.md` 경로, 총 Decision 수, Unresolved 수, Spec Impact 존재 여부 출력.

Adapt it to Codex:
- capture explicit decisions, tradeoffs, and unresolved items in `research.md`
- use local docs and web research only when required by the command and current policy
- prefer primary sources for technical decisions
- keep the research output decision-oriented, not narrative-heavy
