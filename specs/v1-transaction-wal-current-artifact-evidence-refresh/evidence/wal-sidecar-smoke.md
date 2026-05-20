# WAL Sidecar Reopen Smoke

Current repo SHA: `bc45346ff9eb92e3f2585f14f6a694e10bce0918`

The smoke used a fresh temporary database path. The machine-specific temporary
parent is redacted, while the product evidence identity remains `$DB_PATH` and
`$DB_PATH.wal`.

## Create And Insert

command: cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"
exit_code: 0
stdout: ""
stderr: ""
sidecar_path: "$DB_PATH.wal"
exists_after_create_insert: true
bytes_after_create_insert: 202

## Separate Reopen Select

command: cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"
exit_code: 0
stdout: "id|name\n1|ada\n2|bea\n"
stderr: ""
sidecar_path: "$DB_PATH.wal"
exists_after_reopen_select: true
bytes_after_reopen_select: 202

observed_result: A separate build-coupled `cargo run --bin db -- exec` process
reopened the same database path, read rows committed by the earlier mutation
process, and retained the WAL sidecar with positive byte length.
