---
name: speckit-analyze
description: Perform cross-artifact consistency and quality analysis across `spec.md`, `plan.md`, `design.md`, and `tasks.md`. Use when the user says `speckit.analyze` or wants a read-only Speckit artifact audit before implementation.
---

# Speckit Analyze

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

If the user input indicates `autopilot`, `non-interactive`, `sdd-autopilot`, or an equivalent instruction to avoid user interaction, switch to **Autopilot Mode**:
- Stay read-only.
- Return findings and remediation priorities without asking the user whether to continue.
- Assume the caller will decide and apply the edits after reading the report.

## Goal

Identify inconsistencies, duplications, ambiguities, and underspecified items across the core artifacts (`spec.md`, `plan.md`, `design.md`, `tasks.md`) before implementation. This command MUST run only after `/speckit:tasks` has successfully produced a complete `tasks.md`.

## Operating Constraints

**STRICTLY READ-ONLY**: Do **not** modify any files. Output a structured analysis report. Offer a remediation plan, but do not apply edits directly.

**Constitution Authority**: The project constitution (`.specify/memory/constitution.md`) is authoritative only when it contains real, project-specific principles. If the file is still the stock placeholder template (e.g. `[PRINCIPLE_1_NAME]`, `[SECTION_2_NAME]`), report that as a repo-governance warning and do not fail a feature artifact solely on that basis. Concrete constitution conflicts remain automatically CRITICAL and require adjustment of the spec, plan, or tasks.

## Execution Steps

### 1. Initialize Analysis Context

Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` once from repo root and parse JSON for FEATURE_DIR and AVAILABLE_DOCS. The helper must support either a standard `001-feature-name` branch, a `task-001-feature-name` worktree branch, or an explicit `SPECIFY_FEATURE` / `SPECIFY_FEATURE_DIR` override. If the helper is unavailable, resolve FEATURE_DIR from the current worktree manually before continuing. Derive absolute paths:

- SPEC = FEATURE_DIR/spec.md
- PLAN = FEATURE_DIR/plan.md
- DESIGN = FEATURE_DIR/design.md
- TASKS = FEATURE_DIR/tasks.md

Abort with an error message if any required file is missing (instruct the user to run missing prerequisite command).
For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

### 2. Load Artifacts (Progressive Disclosure)

Load only the minimal necessary context from each artifact:

**From spec.md:**

- Overview/Context
- Functional Requirements
- Non-Functional Requirements
- User Stories
- Edge Cases (if present)

**From plan.md:**

- Architecture/stack choices
- Data Model references
- Phases
- Technical constraints

**From design.md:**

- System Architecture (C4 Diagrams, Layer Dependency Rules)
- Component Interaction Flows (Sequence Diagrams: Payment, Analysis Pipeline, SSE, Chat)
- Detailed Design (Classes, Interfaces, Invariants)
- State Machines (Order, Job, Chat Session) & Algorithms (Weighting, Grade)
- Security Architecture (Auth, Encryption) & Error Handling Strategies

**From tasks.md:**

- Task IDs
- Descriptions
- Phase grouping
- Parallel markers [P]
- Referenced file paths

**From constitution:**

- Load `.specify/memory/constitution.md` for principle validation

### 3. Build Semantic Models

Create internal representations (do not include raw artifacts in output):

- **Requirements inventory**: Each functional + non-functional requirement with a stable key (derive slug based on imperative phrase; e.g., "User can upload file" → `user-can-upload-file`)
- **User story/action inventory**: Discrete user actions with acceptance criteria
- **Design inventory**: UI components, styling constants, and interaction behaviors
- **Architecture inventory**: Key components (Services, Agents), Interfaces (LLMClient), and Data Flows
- **State transition logic**: Valid status transitions for key entities (Order, Job, Session)
- **Task coverage mapping**: Map each task to requirements, stories, or design elements
- **Constitution rule set**: Extract principle names and MUST/SHOULD normative statements

### 4. Detection Passes (Token-Efficient Analysis)

Focus on high-signal findings. Limit to 50 findings total; aggregate remainder in overflow summary.

#### A. Duplication Detection

- Identify near-duplicate requirements
- Mark lower-quality phrasing for consolidation

#### B. Ambiguity Detection

- Flag vague adjectives (fast, scalable, secure, intuitive, robust) lacking measurable criteria
- Flag unresolved placeholders (TODO, TKTK, ???, `<placeholder>`, etc.)

#### C. Underspecification

- Requirements with verbs but missing object or measurable outcome
- User stories missing acceptance criteria alignment
- Design elements missing visual states (e.g., loading, error)
- Design components (e.g., Orchestrator, Workers, Models) missing implementation tasks
- Domain-specific Complexities (e.g., AI context limits, FinTech transaction boundaries) missing explicit constraints or designs
- Error handling strategies (e.g., retries, fallbacks, circuit breakers) missing in tasks
- Tasks referencing files or components not defined in spec/plan/design

#### D. Constitution Alignment

- Any requirement or plan element conflicting with a MUST principle
- Missing mandated sections or quality gates from a concrete constitution
- Placeholder constitution template detected in repo governance files

#### E. Coverage Gaps

- Requirements with zero associated tasks
- Tasks with no mapped requirement/story
- Non-functional requirements not reflected in tasks (e.g., performance, security)

#### F. Inconsistency

- Terminology drift (same concept named differently across files)
- Data entities referenced in plan but absent in spec (or vice versa)
- Design components referenced in tasks absent in design.md
- Visual requirements in spec.md missing concrete design definitions
- Design flows (Sequence Diagrams) contradict Task order (e.g., implementing integration before atomic units)
- Architecture layers (Interface/App/Domain/Infra) violated in Plan/Tasks (e.g., Domain depends on Infra)
- API endpoints in Design differ from contracts/api.md
- Task ordering contradictions (e.g., integration tasks before foundational setup tasks without dependency note)
- Conflicting requirements (e.g., one requires Next.js while other specifies Vue)

### 5. Severity Assignment

Use this heuristic to prioritize findings:

- **CRITICAL**: Violates a concrete constitution MUST, missing core spec artifact, or requirement with zero coverage that blocks baseline functionality
- **HIGH**: Duplicate or conflicting requirement, ambiguous security/performance attribute, untestable acceptance criterion
- **MEDIUM**: Terminology drift, missing non-functional task coverage, underspecified edge case
- **LOW**: Style/wording improvements, minor redundancy not affecting execution order, or placeholder repo-governance files that do not yet define enforceable feature rules

### 6. Produce Compact Analysis Report

Output a Markdown report (no file writes) with the following structure:

## Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| A1 | Duplication | HIGH | spec.md:L120-134 | Two similar requirements ... | Merge phrasing; keep clearer version |

(Add one row per finding; generate stable IDs prefixed by category initial.)

**Coverage Summary Table:**

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|

**Constitution Alignment Issues:** (if any)

**Unmapped Tasks:** (if any)

**Metrics:**

- Total Requirements
- Total Tasks
- Coverage % (requirements with >=1 task)
- Ambiguity Count
- Duplication Count
- Critical Issues Count

### 7. Provide Next Actions

At end of report, output a concise Next Actions block:

- If CRITICAL issues exist: Recommend resolving before `sdd-implement-loop`
- If only LOW/MEDIUM: User may proceed, but provide improvement suggestions
- Provide explicit command suggestions: e.g., "Run /speckit:specify with refinement", "Run /speckit:plan to adjust architecture", "Manually edit tasks.md to add coverage for 'performance-metrics'", or "Start `sdd-implement-loop` once the critical issues are closed"

### 8. Offer Remediation

In interactive mode, you may ask the user whether they want concrete remediation suggestions. In Autopilot Mode, skip the question and simply include the remediation priorities in the report. Do NOT apply them automatically.

## Operating Principles

### Context Efficiency

- **Minimal high-signal tokens**: Focus on actionable findings, not exhaustive documentation
- **Progressive disclosure**: Load artifacts incrementally; don't dump all content into analysis
- **Token-efficient output**: Limit findings table to 50 rows; summarize overflow
- **Deterministic results**: Rerunning without changes should produce consistent IDs and counts

### Analysis Guidelines

- **NEVER modify files** (this is read-only analysis)
- **NEVER hallucinate missing sections** (if absent, report them accurately)
- **Prioritize concrete constitution violations** (these are always CRITICAL)
- **Use examples over exhaustive rules** (cite specific instances, not generic patterns)
- **Report zero issues gracefully** (emit success report with coverage statistics)

Adapt it to Codex:
- stay strictly read-only; a later caller may apply remediation work after reading your report
- use Codex shell/tools to load only the needed artifact sections
- report findings, severity, coverage, and next actions in Korean by default
- treat `.specify/memory/constitution.md` as a hard constraint only when it is not a placeholder template
