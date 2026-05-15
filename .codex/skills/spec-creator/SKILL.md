# V1 Persistent DB Spec Creator

Use this repo-local contract when creating specs for `persistent-db-core`.

## Spec Creation Rules

- Select the smallest coherent slice that advances one current V1 gap.
- Preserve the `db` binary contract and keep the implementation in Rust.
- Include scope, non-goals, expected behavior, edge cases, and concrete acceptance criteria.
- Include deterministic verification commands, starting with `cargo test` and a relevant `cargo run --bin db -- ...` smoke check.
- For storage and recovery features, name the on-disk artifacts and corruption or crash cases that must be tested.
- For SQL and index features, define input statements, expected rows, and ordering guarantees.

## Non-Goals For V1 Specs

- Networked database server behavior.
- Multi-process concurrency.
- Distributed storage.
- Query optimization beyond the V1 acceptance gates.
