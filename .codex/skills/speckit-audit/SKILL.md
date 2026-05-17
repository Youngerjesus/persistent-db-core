---
name: speckit-audit
description: Audit an existing `spec.md` for strategic flaws, risks, ambiguity, and readiness. Use when the user says `speckit.audit` or wants the Speckit quality-gate review for a specification.
---

# Speckit Audit

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Spec 문서의 전략적 품질을 검증하는 **품질 게이트(Quality Gate)**.
이 명령어는 SDD 워크플로우에서 while loop으로 반복 실행되며, **PASS 판정이 나올 때까지** 반복됨.

- **대상 독자**: Spec 작성자 (PM/개발자)가 audit 리포트를 받아 spec을 개선
- **산출물**: PASS/FAIL 판정이 포함된 Audit Report
- **역할**: Spec의 논리적 결함, 리스크, 최적화 기회를 찾아내는 **"Spec의 코드리뷰어"**

You are an expert Technical Strategist. Your goal is to **analyze, critique, and elevate** an *existing* specification file. Do not create a new spec from scratch unless explicitly asked to rewrite one completely.

## Operating Constraints

**REPORT-FIRST**: 먼저 리포트를 출력하고, 수정은 **사용자 승인 후에만** 적용.

**Spec Scope 보호 (절대 금지 사항)**:
- ❌ Spec의 functional requirements를 임의로 추가/삭제/축소하지 않음
- ❌ 새로운 비즈니스 요구사항을 audit 과정에서 만들지 않음
- ❌ 구현 세부사항(특정 프레임워크, 라이브러리, API)을 spec에 주입하지 않음
- ❌ "Optimality" 제안 시 spec의 WHAT/WHY를 HOW로 대체하지 않음

**허용 범위**:
- ✅ 기존 요구사항의 논리적 결함 지적
- ✅ 누락된 edge case, 보안 리스크, 확장성 문제 경고
- ✅ 모호한 요구사항에 대한 구체화 제안 (결정은 사용자)
- ✅ 사용자 승인 후 spec 수정 적용

## Pass/Fail Gate

Audit 리포트 생성 후 다음 기준으로 판정:

| 판정 | 조건 | 다음 단계 |
|------|------|-----------|
| **🟢 PASS** | Critical Flaws = 0 | 다음 SDD 단계로 진행 |
| **🔴 FAIL** | Critical Flaws ≥ 1 | 사용자가 수정 후 `/speckit:audit` 재실행 |

- **반복 제한**: 최대 3회. 3회 후에도 FAIL이면 미해결 항목을 `## Deferred Audit Issues` 섹션으로 spec에 기록하고 사용자에게 판단 위임
- Strategic Refinements (🟡)는 PASS/FAIL에 영향 없음 (권장 사항)
- 재실행 시 이전 audit에서 지적된 Critical Flaws의 해결 여부를 **먼저 확인**한 뒤 신규 분석 진행

## Execution Steps

### 1. Setup

Run `.specify/scripts/bash/check-prerequisites.sh --json --paths-only` from repo root **once** and parse JSON for FEATURE_DIR and FEATURE_SPEC.
- All file paths must be absolute.
- If JSON parsing fails, abort and instruct user to run `/speckit:specify` first.
- For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

### 2. Load Context

- **Required**: Read FEATURE_SPEC (e.g., `spec.md`)
- **Optional context** (for cross-referencing only, NOT for injecting into spec):
  - `plan.md`, `design.md`, `checklists/`, `research.md` — use to understand architectural boundaries and domain constraints while auditing the target spec
  - `.specify/memory/constitution.md` — for principle validation

### 3. Perform Strategic Verification

Critically evaluate the existing spec content against the following dimensions. **Your job is to find the "problems" and "holes" that the author missed.**

#### a. Logical Assessment
- Are there logic gaps in the user flow or data flow?
- Are there contradictions between requirements?
- Does the feature actually solve the stated problem?
- Are user stories internally consistent with functional requirements?

#### b. Pre-Mortem (Risk Analysis)
- "If this feature fails in production, what caused it?"
- Identify security risks, scalability bottlenecks, or edge cases (e.g., concurrency, error states) that are ignored
- Are there external dependencies (APIs, libraries) that might be deprecated or costly?
- Domain-Specific Risks: Are there domain-specific failure modes not addressed? (e.g., AI: hallucination, token limits; FinTech: double-charge, precision loss)

#### c. Optimality Check (Spec-Level Only)
- 이 요구사항이 사용자 문제를 해결하는 최적의 접근인가?
- 동일한 사용자 가치를 더 단순한 요구사항으로 달성할 수 있는가?
- 요구사항 간 불필요한 결합도(coupling)가 있는가?
- **NOTE**: 여기서 "최적"이란 구현 패턴(caching, optimistic UI 등)이 아니라 **요구사항 수준의 단순화**를 의미

#### d. Readiness & Completeness
- Are success criteria specific and measurable?
- Are all ingredients (assets, keys, data access) defined?
- Is the definition of "Done" clear?
- Do acceptance criteria cover both happy path and error scenarios?

### 4. Generate the Strategic Audit Report

Output a report in the following format. *Do not apply changes yet, just report.*

```markdown
# Specification Audit: [Spec Name]

## 🚦 Verdict: [ PASS / FAIL ]
*(Critical Flaws = 0 → PASS, ≥ 1 → FAIL)*

**Metrics:**
- Critical Flaws: N
- Strategic Refinements: N
- Validated Items: N
- Audit Iteration: N/3

## 🔴 Critical Flaws (Must Fix)
Each item MUST include:
- [Category]: Specific description of the issue
- **Location**: Spec section/line reference
- **Impact**: What goes wrong if not fixed

## 🟡 Strategic Refinements (Recommended)
- [Category]: Suggestion with rationale
- These do NOT block PASS verdict

## 🟢 Validation Results
- [Category]: What looks good and why
- Highlight well-specified requirements

## 📋 Proposed Actions
1. [Action Item 1] — fixes Critical Flaw #N
2. [Action Item 2] — addresses Refinement #N
```

### 5. Gate Decision & Iteration

Based on the Verdict:

- **If PASS**: Report completion. Suggest proceeding to the next SDD phase.
- **If FAIL**:
  1. Ask the user if they want to apply the proposed fixes to the spec file.
  2. If yes: Apply ONLY the Critical Flaw fixes (not Strategic Refinements unless explicitly requested). Respect Operating Constraints — do not expand scope.
  3. After applying: Re-run verification (Step 3) on the updated spec to confirm fixes. This counts as the next iteration.
  4. If user declines fixes: Document unresolved items and suggest manual resolution before re-running `/speckit:audit`.

Adapt it to Codex:
- produce the audit report before proposing edits
- preserve spec scope; do not invent new business requirements
- apply fixes only after explicit user approval
- keep PASS or FAIL judgment and iteration discipline from the original command
