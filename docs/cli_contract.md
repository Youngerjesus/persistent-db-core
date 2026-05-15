# V1 `db` CLI Contract

This slice defines only the deterministic command-line contract and smoke baseline for the `db` binary.

## Supported Commands

The supported command surface is intentionally small:

```text
db --help
db help
```

Both commands exit with code `0`, write no stderr, and write identical help text to stdout.

## Help Stdout

The help output must contain these core lines in this order:

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
Supported commands:
  help        Print this help text.
Reserved future commands:
  open <path>
  exec <path> <sql>
  check <path>
  bench <path>
V1 bootstrap scope:
  This build only defines the CLI contract and smoke baseline.
  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

## Exit Codes

- `0`: `db --help` or `db help` printed the help contract successfully.
- `2`: the first argument was unsupported, or no supported command was provided.

## Unsupported Input

Unsupported arguments and subcommands exit with code `2`, write no stdout, and write this stderr format:

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

`<token>` is the first unsupported token supplied by the user. For example, `db --unknown` reports `--unknown`, and `db open demo.db` reports `open`.

## Reserved Future Commands

The following names are reserved for later V1 work but are not executable in this slice:

```text
open <path>
exec <path> <sql>
check <path>
bench <path>
```

Invoking any reserved command currently follows the unsupported input behavior.

## Non-Goals

This slice does not implement storage pages, SQL execution, indexes, transactions, WAL, recovery, networking, multi-process concurrency, or distributed storage.
