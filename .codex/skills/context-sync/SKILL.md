---
name: context-sync
description: Keep AGENTS.md, DESIGN.md, docs, specs, and .codex guidance aligned when a requirement, policy, or assumption changes.
---

# Context Sync

Use this skill when a policy, architecture decision, requirement, or terminology change needs to be reflected across multiple documents.

## Workflow

1. Identify the old context and the new context.
2. Search the active doc surface:
   - `AGENTS.md`
   - `DESIGN.md`
   - `README.md`
   - `.codex/**`
   - `docs/**`
   - `specs/**`
   - `repo_name/docs/**`
   - `repo_name/specs/**`
3. Filter out irrelevant matches and archives.
4. Update only the files that actually encode the outdated assumption.
5. Report:
   - `changed`
   - `ignored`
   - `conflicts`
   - `follow-up`

## Constraints

- Do not blindly find-and-replace.
- Do not rewrite archive or history docs unless explicitly asked.
- If the replacement policy is ambiguous, stop and report the conflict.
