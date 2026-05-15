# AGENTS (persistent-db-core)

This repository is the managed product repo for the V1 persistent database core. Product code and repo-local context live here. Autopilot orchestration, scheduling, reports, and runtime state live in the sibling `autopilot/` control-plane repository.

## Product Direction

Build a small, deterministic Rust CLI database binary named `db`.

V1 must prove durable single-process database behavior through:

- CLI contract and smoke behavior.
- Disk page storage and record format.
- SQL parser, schema catalog, and basic execution.
- Primary B-tree index.
- Secondary index range scans.
- Transactions, WAL, and recovery.
- Deterministic crash matrix.
- SQLite differential/property tests.
- `db check` invariant validation.
- Benchmark and acceptance documentation.

## Engineering Rules

- Keep changes scoped to the current CAO gap and its spec package.
- Prefer deterministic tests and explicit on-disk fixtures over implicit state.
- Treat file format, WAL, and recovery behavior as compatibility-sensitive once introduced.
- Do not add network services, background daemons, or remote dependencies for V1.
- Do not store Autopilot runtime state in this repository.

## Verification Baseline

- `cargo test`
- `cargo run --bin db -- --help`

## Repo-Local Skills

`.codex/skills/spec-reviewer/SKILL.md` and `.codex/skills/spec-creator/SKILL.md` are repo-local product context contracts for V1 DB work. They are not the generic Autopilot runtime.
