# Adoption Preflight

## Verdict
PASS

## Scope
This preflight adopts the approved package in `specs/v1-db-check-invariants/` without rewriting canonical inputs.

## Checks
| Check | Result | Evidence |
|---|---|---|
| `spec.md` exists | pass | `specs/v1-db-check-invariants/spec.md` |
| `contracts.md` exists | pass | `specs/v1-db-check-invariants/contracts.md` |
| approved lifecycle status present | pass | `spec.md` contains `Status: APPROVED` |
| unresolved TODO/TBD placeholders in canonical inputs | pass | none found during read |
| external dependency blocker | pass | no network/service/browser dependency required |
| implementation-time product ambiguity | pass | invariant matrix and evidence contract define CLI outputs, fixtures, docs, and verification |

## Repo Reality Rechecked
- Latest HEAD: `881905933361ae5957a43c350efb1b6005d759f0`
- `git status --short --branch`: branch `task-2026-05-18-03-29-23-v1-db-check-invariants`; `specs/v1-db-check-invariants/` is untracked planning input/output.
- Relevant files exist: `src/main.rs`, `src/lib.rs`, `src/storage.rs`, `src/sql.rs`, `src/index.rs`, `tests/cli_contract.rs`, `tests/page_storage.rs`, `tests/wal_recovery.rs`, `tests/primary_index.rs`, `docs/cli_contract.md`, `docs/file_format.md`.
- `README.md` is absent; repo-level guidance came from `AGENTS.md`.

## Boundary Notes
- `review_loop/code_context.md` is supporting evidence only. Implementation must re-read live code before editing.
- The task metadata asks for visual and UX design-review evidence, but canonical `spec.md` and `contracts.md` explicitly reject UI/DOM/screenshot/UX evidence for this CLI-only task. The implementation plan follows the canonical contract.
