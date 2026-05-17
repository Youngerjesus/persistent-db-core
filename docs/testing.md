# Testing

`./scripts/verify` is the baseline verification command for this repo.

`./scripts/verify_differential_property` runs the task-specific SQLite
differential/property harness:

```text
cargo test --test differential_property -- --nocapture
```

The harness uses deterministic seeds by default. A single seed can be replayed
with `PDB_DIFF_SEED=<seed>`, and a shortest failing prefix can be replayed with
`PDB_DIFF_PREFIX=<operation_count>`:

```text
PDB_DIFF_SEED=1 PDB_DIFF_PREFIX=17 cargo test --test differential_property -- --nocapture
```

When the harness detects a mismatch, it prints the seed, failing operation
index, minimal replay prefix, expected SQLite rows, actual `db` rows, artifact
path, and rerun command. Local generated failure evidence is written under
`target/differential_property/failures/<seed>.json`; those files are not durable
product documentation or SSOT.
