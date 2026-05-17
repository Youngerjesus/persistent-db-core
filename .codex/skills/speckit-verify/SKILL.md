---
name: speckit-verify
description: Compatibility alias for the canonical `verify` skill. Use when older callers still invoke `speckit.verify` or `speckit-verify`.
---

# Speckit Verify Alias

This skill name is retained only for backward compatibility.

- Canonical skill: `verify`
- Canonical path: `.codex/skills/verify/SKILL.md`
- Canonical behavior: read-only implementation verification gate

When this alias is invoked, execute the `verify` skill behavior exactly.

Compatibility rules:
- Do not introduce behavior that differs from `verify`.
- Do not keep separate contracts, prompts, or report shapes here.
- If this alias and `verify` ever diverge, `verify` is the source of truth.
