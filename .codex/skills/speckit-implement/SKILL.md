---
name: speckit-implement
description: Execute the implementation plan from `tasks.md` in the Speckit workflow. Use when the user says `speckit.implement` or wants Codex to implement the next or specified Speckit task.
---

# Speckit Implement

## User Input

{{args}}

You **MUST** consider the user input before proceeding (if not empty).

## Outline

1. Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` from repo root and parse FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\\''m Groot' (or double-quote if possible: "I'm Groot").

2. **Check Phase 1 Contracts (Pre-flight Readiness)**:
   - Check if `spec_phase1_contracts.md` (or equivalent `contracts.md` tracking Phase 1 readiness) exists in `FEATURE_DIR`.
   - If it exists, scan all checklist items under `SDD Phases` (or similar readiness sections).
   - If ANY Phase 1 task (e.g., `research`, `readiness`, `tasks`, `spec-audit`) is incomplete `[ ]`:
     - **STOP IMMEDIATELY**.
     - Display a message indicating that Phase 1 specifications are incomplete and the user must finish the missing SDD phases before starting implementation.
   - If ALL items are complete `[x]`, proceed to the next step.

3. **Check checklists status** (if FEATURE_DIR/checklists/ exists):
   - Scan all checklist files in the checklists/ directory
   - For each checklist, count:
     - Total items: All lines matching `- [ ]` or `- [X]` or `- [x]`
     - Completed items: Lines matching `- [X]` or `- [x]`
     - Incomplete items: Lines matching `- [ ]`
   - Create a status table:

     ```text
     | Checklist | Total | Completed | Incomplete | Status |
     |-----------|-------|-----------|------------|--------|
     | ux.md     | 12    | 12        | 0          | ✓ PASS |
     | test.md   | 8     | 5         | 3          | ✗ FAIL |
     | security.md | 6   | 6         | 0          | ✓ PASS |
     ```

   - Calculate overall status:
     - **PASS**: All checklists have 0 incomplete items
     - **FAIL**: One or more checklists have incomplete items

   - **If any checklist is incomplete**:
     - Display the table with incomplete item counts
     - **STOP** and ask: "Some checklists are incomplete. Do you want to proceed with implementation anyway? (yes/no)"
     - Wait for user response before continuing
     - If user says "no" or "wait" or "stop", halt execution
     - If user says "yes" or "proceed" or "continue", proceed to step 4

   - **If all checklists are complete**:
     - Display the table showing all checklists passed
     - Automatically proceed to step 4

4. Load and analyze the implementation context:
   - **REQUIRED**: Read spec.md for the complete spec list and execution plan
   - **REQUIRED**: Read tasks.md for the complete task list and execution plan
   - **REQUIRED**: Read plan.md for tech stack, architecture, and file structure
   - **REQUIRED**: Read checklists/ for checklist status
   - **IF EXISTS**: Read data-model.md for entities and relationships
   - **IF EXISTS**: Read contracts/ for API specifications and test requirements
   - **IF EXISTS**: Read research.md for technical decisions and constraints
   - **IF EXISTS**: Read quickstart.md for integration scenarios
   - **IF EXISTS**: Read design.md for system architecture and component interactions

5. **Project Setup Verification**:
   - **REQUIRED**: Create/verify ignore files based on actual project setup:

   **Detection & Creation Logic**:
   - Check if the following command succeeds to determine if the repository is a git repo (create/verify .gitignore if so):

     ```sh
     git rev-parse --git-dir 2>/dev/null
     ```

   - Check if Dockerfile* exists or Docker in plan.md → create/verify .dockerignore
   - Check if .eslintrc* exists → create/verify .eslintignore
   - Check if eslint.config.* exists → ensure the config's `ignores` entries cover required patterns
   - Check if .prettierrc* exists → create/verify .prettierignore
   - Check if .npmrc or package.json exists → create/verify .npmignore (if publishing)
   - Check if terraform files (*.tf) exist → create/verify .terraformignore
   - Check if .helmignore needed (helm charts present) → create/verify .helmignore

   **If ignore file already exists**: Verify it contains essential patterns, append missing critical patterns only
   **If ignore file missing**: Create with full pattern set for detected technology

   **Common Patterns by Technology** (from plan.md tech stack):
   - **Node.js/JavaScript/TypeScript**: `node_modules/`, `dist/`, `build/`, `*.log`, `.env*`
   - **Python**: `__pycache__/`, `*.pyc`, `.venv/`, `venv/`, `dist/`, `*.egg-info/`
   - **Java**: `target/`, `*.class`, `*.jar`, `.gradle/`, `build/`
   - **C#/.NET**: `bin/`, `obj/`, `*.user`, `*.suo`, `packages/`
   - **Go**: `*.exe`, `*.test`, `vendor/`, `*.out`
   - **Ruby**: `.bundle/`, `log/`, `tmp/`, `*.gem`, `vendor/bundle/`
   - **PHP**: `vendor/`, `*.log`, `*.cache`, `*.env`
   - **Rust**: `target/`, `debug/`, `release/`, `*.rs.bk`, `*.rlib`, `*.prof*`, `.idea/`, `*.log`, `.env*`
   - **Kotlin**: `build/`, `out/`, `.gradle/`, `.idea/`, `*.class`, `*.jar`, `*.iml`, `*.log`, `.env*`
   - **C++**: `build/`, `bin/`, `obj/`, `out/`, `*.o`, `*.so`, `*.a`, `*.exe`, `*.dll`, `.idea/`, `*.log`, `.env*`
   - **C**: `build/`, `bin/`, `obj/`, `out/`, `*.o`, `*.a`, `*.so`, `*.exe`, `Makefile`, `config.log`, `.idea/`, `*.log`, `.env*`
   - **Swift**: `.build/`, `DerivedData/`, `*.swiftpm/`, `Packages/`
   - **R**: `.Rproj.user/`, `.Rhistory`, `.RData`, `.Ruserdata`, `*.Rproj`, `packrat/`, `renv/`
   - **Universal**: `.DS_Store`, `Thumbs.db`, `*.tmp`, `*.swp`, `.vscode/`, `.idea/`

   **Tool-Specific Patterns**:
   - **Docker**: `node_modules/`, `.git/`, `Dockerfile*`, `.dockerignore`, `*.log*`, `.env*`, `coverage/`
   - **ESLint**: `node_modules/`, `dist/`, `build/`, `coverage/`, `*.min.js`
   - **Prettier**: `node_modules/`, `dist/`, `build/`, `coverage/`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`
   - **Terraform**: `.terraform/`, `*.tfstate*`, `*.tfvars`, `.terraform.lock.hcl`
   - **Kubernetes/k8s**: `*.secret.yaml`, `secrets/`, `.kube/`, `kubeconfig*`, `*.key`, `*.crt`

6. Parse tasks.md structure and extract:
   - **Task phases**: Setup, Tests, Core, Integration, Polish
   - **Task dependencies**: Sequential vs parallel execution rules
   - **Task details**: ID, description, file paths, parallel markers [P]
   - **Execution flow**: Order and dependency requirements

7. Execute implementation following the task plan (Focus on Target):
   - **Target Identification**: Identify the task specified in `{{args}}` or the next incomplete task in tasks.md that has existing tests written by the QA Engineer.
   - **Respect Dependencies**: Always check `tasks.md` for execution flow. Run sequential tasks in order, and parallel tasks [P] together. Do not skip preceding incomplete tasks.
   - **Test-Driven Run**: First, run the existing tests (e.g., `pytest` or `npx playwright test`) related to the target task.
   - **Fix Import Errors**: If tests fail with `ImportError` or `ModuleNotFoundError`, intentionally create the minimal skeleton (empty classes/functions) matching the EXACT names imported in the test files. This ensures you conform to the agreed interface.
   - **Implement Core Logic**: Write the actual application code to pass the failing tests.

8. Implementation execution rules (CRITICAL Constraints):
   - **Setup First**: Initialize project structure, dependencies, configuration if needed.
   - **Follow QA's Lead**: You MUST use the exact class names, method names, payload structures, and return types defined by the `qa-prep` workflow in its generated tests and `qa_mapping.md`. Do not invent your own interface if the test demands a specific one.
   - **Minimum Code + Compliance**: Only write the code necessary to make the tests pass (Green phase of TDD), BUT you must rigorously ensure that the implementation completely satisfies all requirements defined in `spec.md` and `design.md`.
   - **Do not write new tests**: Your job is to implement the core logic to make existing tests pass. Do not write or modify test code unless absolutely necessary (e.g., to fix syntactical errors preventing compilation).
   - **Domain Constraints**: Strictly implement domain-specific complexities (e.g., AI prompt templates, context management, strict financial rules) exactly as defined in `design.md`.
   - **Integration Work**: When the task demands it, implement necessary Database connections, middleware, logging, and external services cleanly.

9. Progress tracking and error handling:
   - Execute the tests iteratively as you develop the application code.
   - Keep refining your implementation until all tests for the target task pass.
   - Provide clear error messages with context if tests repeatedly fail despite your implementation.
   - **IMPORTANT**: **ONLY** when the tests pass successfully, make sure to mark the target task off as `[X]` in the `tasks.md` file.

10. Completion validation:
    - Validate that tests pass and the behavior satisfies both the task and the spec.
    - Confirm the implementation adheres to the technical boundaries set by `plan.md`.
    - Report final status to the user and suggest running the next `/speckit:test <TaskID>` if there are remaining tasks.

Note: This command relies on an explicit Test-First workflow. If no tests exist for a task (i.e. you cannot find failing tests to pass), you must HALT execution and instruct the user to run `/speckit:test <TaskID>` (or rely on the `sdd-implement-loop` upfront test generation) first.

Adapt it to Codex:
- follow TDD and implement only the targeted task scope
- update task progress only when code and relevant verification pass
- use subagents when helpful, but preserve the plan and design boundaries
- avoid speculative interfaces that contradict existing artifacts
