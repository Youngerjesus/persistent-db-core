---
name: speckit-plan
description: Generate the implementation planning artifact for a feature using the approved spec and research context. Use when the user says `speckit.plan` or wants the Speckit planning phase in Codex.
---

# Speckit Plan

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Spec과 Research를 바탕으로 **기술 구현 계획과 설계 산출물**(data-model, contracts, quickstart)을 생성하는 단계.

- **대상 독자**: `/speckit:design`, `/speckit:tasks` 단계의 AI Agent와 개발자
- **산출물**: `plan.md`, `data-model.md`, `contracts/`, `quickstart.md`
- **역할**: Spec(WHAT) + Research(기술 결정)을 **구현 가능한 설계 산출물**로 변환
- **권한 분리**: `plan.md`는 canonical implementation index이며, 세부 데이터 구조는 `data-model.md`, 세부 인터페이스 계약은 `contracts/`, 실행 검증 시나리오는 `quickstart.md`가 authoritative source이다.

## Operating Constraints

**Spec Scope 보호**:
- ❌ Spec의 functional requirements를 임의 변경/삭제하지 않음
- ❌ 구현 편의를 위해 요구사항을 축소하지 않음
- ✅ 기술적 제약 발견 시 `## Spec Impact` 섹션에 기록하고 사용자 안내

## Outline

1. **Setup**: Run `.specify/scripts/bash/setup-plan.sh --json` from repo root and parse JSON for FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, BRANCH. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

2. **Load context**: Read FEATURE_SPEC and `.specify/memory/constitution.md`. Load IMPL_PLAN template (already copied).

3. **Execute plan workflow**: Follow the structure in IMPL_PLAN template to:
   - Fill Technical Context based on `research.md` (all unknowns should be resolved)
   - Fill Constitution Check section from constitution
   - Evaluate gates (ERROR if violations unjustified)
   - Phase 1: Generate data-model.md, contracts/, quickstart.md
   - Phase 2: Strategic Reinforcement (Critique & Iterate on all artifacts)
   - Re-evaluate Constitution Check post-design

4. **Stop and report**: Command ends after Phase 2. Report branch, IMPL_PLAN path, and generated artifacts.

## Phases

### Sub-phase: Load Research Context

**Prerequisites:** `research.md` must be complete and available.
1. Read `research.md` to understand all technical decisions, architecture choices, and resolved clarifications.
2. Incorporate the findings from `research.md` directly into the Technical Context of your plan.

### Phase 1: Design & Contracts

**Prerequisites:** `research.md` context loaded

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

**Output**: plan.md, data-model.md, /contracts/*, quickstart.md

### Phase 2: Strategic Reinforcement & Validation

**Goal**: Ensure the generated design is valid, robust, and aligned with user goals before finalizing.

1.  **Critique the artifacts** (`research.md`, `data-model.md`, `contracts`, `quickstart.md`):
    -   **Validity Check**: "Is this direction technically sound and user-centric?"
    -   **Gap Analysis**: "What is missing? What assumptions are risky?"
    -   **Use Case Walkthrough**: Mental walkthrough of the user flow using the new contracts.

2.  **Iterate if necessary**:
    -   If the direction is **invalid** or **suboptimal**:
        -   Explain *why* it is invalid.
        -   Propose a better direction.
        -   **REWRITE** the affected plan artifacts (`data-model.md`, `contracts/`, `quickstart.md`) immediately.
        -   **LIMIT**: `research.md`의 결정사항은 수정하지 않음 (research 재실행 필요 시 사용자에게 안내). Spec 요구사항은 절대 수정하지 않음.

**Output**: plan.md, data-model.md, /contracts/*, quickstart.md

## Key rules

- Use absolute paths
- ERROR on gate failures or unresolved clarifications

## Success Criteria

- Spec의 모든 functional requirement가 plan 산출물(data-model, contracts)에 매핑됨
- Research의 모든 기술 결정이 Technical Context에 반영됨
- Constitution 위반 없음 (또는 정당한 사유 문서화)
- Coverage gap이 있으면 리포트에 명시
- `plan.md`가 구조 결정, delivery order, 하위 authoritative artifacts 링크를 모두 포함함

Adapt it to Codex:
- create the plan artifact directly in the feature directory
- keep the plan focused on architecture, constraints, data boundaries, and delivery order
- do not skip prerequisite checks from the original workflow
- call out risks and reasons not to execute the plan before finalizing it
