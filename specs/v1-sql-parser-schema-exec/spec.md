# 최소 SQL schema/execute 경로 구현

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-17-19-38-21
- Task ID: task-2026-05-17-19-38-21-v1-sql-parser-schema-exec
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 최소 SQL schema/execute 경로 구현
- Artifact: v1-sql-parser-schema-exec

## 목표
- V1은 CLI와 durable page storage 기반은 갖췄지만, 아직 사용자가 SQL로 schema를 만들고 row를 넣고 조회하는 observable query behavior가 없습니다. 이 작업은 후속 index, WAL, crash recovery, differential/property test가 참조할 최소 실행 의미론을 만듭니다.

## 지금 해야 하는 이유
- Progress Projection에서 CLI와 disk page storage gate는 `projected_complete`이고, `gate-v1-sql-schema-exec`는 `req-v1-sql-exec-examples`가 open입니다. current objective의 sequencing도 storage 다음 SQL/schema execution을 요구하므로 오늘의 가장 작은 downstream unblocker입니다.

## 기대 산출물 변화
- Managed repo에 `db exec <path> <sql>` CLI smoke path, 최소 SQL parser/schema catalog/executor, deterministic tests, durable docs를 추가합니다.
- `CREATE TABLE`, `INSERT`, `SELECT`의 지원 grammar, 출력 row ordering, stderr 형식, exit code, persistence behavior를 문서와 테스트로 고정합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- src/sql.rs
- tests/sql_exec.rs
- tests/cli_contract.rs
- docs/cli_contract.md
- docs/sql_subset.md
- docs/file_format.md
- flow:create-table-insert-select

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 8aea6208d2a42d51a78306ccd57dbbc5e7aad6a4
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, src/storage.rs, tests/cli_contract.rs, docs/cli_contract.md, AGENTS.md, work_queue/progress.md, docs/history_archives/history.md, .codex/agents/decision-brake-readiness-reviewer.toml, .codex/agents/project-reviewer.toml, .codex/agents/task-master.toml, .codex/agents/task-reviewer.toml

## Risk flags
- storage_format_compatibility_must_be_preserved
- unsupported_sql_error_contract_required
- no_protected_area_change

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Current Artifact: `gate-v1-sql-schema-exec`는 open이고 `req-v1-sql-exec-examples`가 missing requirement입니다.
- Current Plan: `gap-v1-sql-parser-schema-exec`의 next candidate hint는 최소 `CREATE TABLE`, `INSERT`, `SELECT` path를 storage 위에 구현하는 것입니다.
- Root Progress Projection: `gate-v1-cli-smoke`와 `gate-v1-disk-page-storage`는 `projected_complete`라 오늘 후보에서 제외됩니다.
- Root Progress Projection: `gate-v1-sql-schema-exec`는 `status=open`, `missing_requirement_ids=[req-v1-sql-exec-examples]`입니다.
- Queue Snapshot: active 또는 reserved task가 없어 동일 feature 중복 실행 근거가 없습니다.
- Managed Repo Snapshot: repo git status가 clean이고, V1 Rust CLI database boundary가 active managed repo로 고정되어 있습니다.
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/src/lib.rs
- persistent-db-core_worktree/main/src/storage.rs
- persistent-db-core_worktree/main/tests/cli_contract.rs
- persistent-db-core_worktree/main/tests/page_storage.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/file_format.md
- persistent-db-core_worktree/main/docs/v1_spec.md
- persistent-db-core_worktree/main/work_queue/progress.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- HELP
- main
- PageStore
- PageStore::open
- PageStore::append_record
- PageStore::read_records
- StorageError
- autopilot/project_manager/tasks/tasks.json#task-2026-05-15-16-06-54-v1-bootstrap-cli-contract
- autopilot/project_manager/tasks/tasks.json#task-2026-05-16-13-58-47-v1-page-storage-record-format
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md

## 범위
- In scope: `db exec <path> <sql>` 실행 경로, 최소 SQL subset parser, schema catalog, row insert/select executor, storage persistence, CLI/docs/tests 갱신입니다.
- Out of scope: SQL projection, `WHERE`, `ORDER BY`, `JOIN`, `UPDATE`, `DELETE`, transactions, WAL, indexes, query optimizer, multi-process concurrency, network server, background daemon, benchmark tooling입니다.
- Out of scope: 브라우저 기반 검증 산출물입니다. 이 작업은 CLI-only Rust database 작업입니다.

## CLI 계약
- 새 지원 명령은 `db exec <path> <sql>`입니다.
- `<path>`는 database file 경로입니다. 파일이 없으면 기존 `PageStore::open` 동작에 따라 생성합니다.
- `<sql>`은 하나의 CLI argument로 전달되는 UTF-8 SQL string입니다. stdin 입력, interactive shell, 여러 `<sql>` argument 조합은 지원하지 않습니다.
- `db --help`와 `db help`의 기존 exit `0`, empty stderr, help stdout 계약은 유지하되 reserved future commands 목록에서 `exec <path> <sql>`를 supported command로 승격해 문서와 테스트를 갱신합니다.
- 성공한 `db exec`는 exit `0`, empty stderr를 반환합니다. `SELECT`가 포함된 경우에만 stdout에 row output을 씁니다.

## 최소 SQL subset
- Statement delimiter는 semicolon입니다. 모든 statement는 `;`로 끝나야 하며, 하나의 `db exec` 호출 안에 여러 statement를 순서대로 넣을 수 있습니다.
- Keywords는 ASCII case-insensitive입니다. Identifier는 `[A-Za-z_][A-Za-z0-9_]*`만 허용하며 출력과 catalog에는 입력 spelling을 그대로 저장합니다.
- Table name과 column name equality는 입력 spelling 보존과 별개로 ASCII case-insensitive입니다. Catalog lookup, duplicate table 판정, duplicate column 판정, missing table 판정은 모두 ASCII case-insensitive 비교를 사용해야 합니다.
- Error message와 SELECT header는 catalog 또는 입력에서 이미 정한 spelling을 유지합니다. Duplicate table과 duplicate column 오류는 충돌을 일으킨 새 입력 spelling을 error target으로 출력하고, missing table 오류는 lookup에 실패한 입력 spelling을 출력합니다.
- Type은 `INT`와 `TEXT`만 허용합니다. `NULL`, primary key, constraint, default value, quoted identifier는 지원하지 않습니다.
- `TEXT` literal은 single quote로 감싼 UTF-8 string만 허용합니다. Escape sequence와 embedded single quote는 이번 범위에서 지원하지 않습니다.
- `INT` literal은 signed 64-bit decimal integer만 허용합니다.

### 지원 grammar
```text
CREATE TABLE <table_name> (<column_name> INT|TEXT[, <column_name> INT|TEXT]*);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
SELECT * FROM <table_name>;
```

### 금지 grammar 예시
```text
CREATE TABLE users id INT);
CREATE TABLE users (id INTEGER);
INSERT INTO users (id, name) VALUES (1, 'ada');
SELECT id FROM users;
SELECT * FROM users WHERE id = 1;
DROP TABLE users;
```

## Deterministic output 계약
- `SELECT * FROM <table_name>;`는 catalog에 저장된 column order로 header line을 출력합니다.
- Row는 successful `INSERT` append order 그대로 출력합니다.
- Field delimiter는 `|`이고, 각 output line은 `\n`으로 끝납니다.
- 하나의 successful `db exec` 안에 여러 `SELECT * FROM ...;`가 있으면 각 SELECT는 독립 result set으로 즉시 이어 붙인 stdout을 만듭니다. 각 result set은 header line을 반복하고, result set 사이에 blank line, separator, count line을 넣지 않습니다.
- Empty table에 대한 `SELECT * FROM <table_name>;`는 header line만 출력하고 row line은 출력하지 않습니다.
- 하나의 `db exec` 호출 중 어떤 statement라도 실패하면 command stdout은 empty string이어야 합니다. 이전 statement가 SELECT output을 만들었더라도 실패 command는 partial stdout을 출력하지 않습니다.
- 현재 subset은 delimiter escaping을 지원하지 않으므로 `TEXT` 값 안의 `|`, newline, carriage return, single quote는 malformed SQL로 처리합니다.

### Happy path smoke
```bash
tmp_db="$(mktemp -t pdb-sql-smoke.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;"
```

Expected stdout:
```text
id|name
1|ada
2|bea
```

Expected stderr는 empty string이고 expected exit code는 `0`입니다.

### Multiple SELECT stdout smoke
```bash
tmp_db="$(mktemp -t pdb-sql-multi-select.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); SELECT * FROM users; INSERT INTO users VALUES (1, 'ada'); SELECT * FROM users;"
```

Expected stdout:
```text
id|name
id|name
1|ada
```

Expected stderr는 empty string이고 expected exit code는 `0`입니다. 첫 번째 SELECT는 empty table result set이므로 header만 출력하고, 두 번째 SELECT는 같은 header를 다시 출력한 뒤 row를 append order로 출력합니다.

### Unsupported SQL negative path
```bash
tmp_db="$(mktemp -t pdb-sql-unsupported.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "SELECT id FROM users;"
```

Expected stdout는 empty string이고 expected exit code는 `2`입니다.

Expected stderr:
```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ...;
```

### Malformed SQL negative path
```bash
tmp_db="$(mktemp -t pdb-sql-malformed.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users id INT);"
```

Expected stdout는 empty string이고 expected exit code는 `2`입니다.

Expected stderr:
```text
error: malformed SQL statement: CREATE TABLE users id INT);
hint: terminate each statement with ';' and use the documented SQL subset.
```

### Semantic failure negative path 계약
- 아래 항목은 supported grammar 안에서 parsing은 성공하지만 schema/catalog/executor 의미론이 실패하는 user SQL error입니다.
- 모든 항목은 expected stdout empty string, expected exit code `2`, panic 없음, `docs/cli_contract.md`와 `docs/sql_subset.md`의 동일 문구 문서화, `cargo test --test sql_exec` assertion을 필수로 요구합니다.

#### Duplicate table 오류
SQL 입력:
```text
CREATE TABLE users (id INT); CREATE TABLE users (name TEXT);
```

예상 stderr:
```text
error: SQL semantic error: table already exists: users
hint: use a new table name for CREATE TABLE in this database.
```

Case variant assertion:
```text
CREATE TABLE users (id INT); CREATE TABLE Users (name TEXT);
```

예상 stderr는 `table already exists: Users`를 사용해야 합니다.

#### Missing table 오류
SQL 입력:
```text
INSERT INTO missing VALUES (1);
```

예상 stderr:
```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT or SELECT.
```

`SELECT * FROM missing;`도 같은 stdout, stderr, exit code 계약을 따라야 합니다.
이미 `CREATE TABLE Users (id INT);`가 존재할 때 `SELECT * FROM users;`는 missing table이 아니라 같은 table lookup으로 성공해야 합니다.

#### Duplicate column 오류
SQL 입력:
```text
CREATE TABLE users (id INT, id TEXT);
```

예상 stderr:
```text
error: SQL semantic error: duplicate column: id
hint: column names in a table must be unique.
```

Case variant assertion:
```text
CREATE TABLE users (id INT, ID TEXT);
```

예상 stderr는 `duplicate column: ID`를 사용해야 합니다.

#### Column count mismatch 오류
SQL 입력:
```text
CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1);
```

예상 stderr:
```text
error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1
hint: INSERT values must match the table schema exactly.
```

#### Type mismatch 오류
SQL 입력:
```text
CREATE TABLE users (id INT); INSERT INTO users VALUES ('ada');
```

예상 stderr:
```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

- 이번 task의 explicit semantic failure matrix는 duplicate table, missing table, duplicate column, column count mismatch, type mismatch로 한정합니다.
- `NULL`, constraints, projection column resolution, `WHERE`, `ORDER BY`, `JOIN`, `UPDATE`, `DELETE` 관련 semantic behavior는 out of scope입니다. 해당 입력이 문서화된 supported grammar를 벗어나면 unsupported SQL 또는 malformed SQL 계약으로 실패해야 하며 panic이나 storage-level success로 귀결되면 안 됩니다.

## Persistence 계약
- 기존 page file format은 변경하지 않습니다. File magic `PDBV1\0\0\0`, file format version `1`, data page magic `PDPG`, opaque record encoding은 기존 `docs/file_format.md`와 `tests/page_storage.rs` 계약을 유지해야 합니다.
- SQL catalog와 row data는 기존 `PageStore::append_record`와 `PageStore::read_records` 위의 opaque payload로 저장해야 하며, page header나 page record framing을 바꾸면 안 됩니다.
- 새 SQL payload는 UTF-8 compatible byte payload로 식별 가능한 version/tag/encoding을 가져야 합니다. 최소 문서화 형식은 `PDBSQL1\0` prefix, record kind `catalog` 또는 `row`, table name, column metadata 또는 row values를 포함해야 합니다.
- 정확한 SQL payload encoding은 `docs/file_format.md`의 compatibility note 또는 `docs/sql_subset.md`에 문서화해야 합니다. 문서는 기존 page storage record와 SQL logical record의 경계를 구분해야 합니다.
- SQL executor는 pre-SQL arbitrary PageStore payload를 valid SQL database로 해석하면 안 됩니다. SQL prefix/tag를 알 수 없는 record는 panic 없이 exit `1`, empty stdout, deterministic stderr로 실패해야 합니다.
- Corrupt page-level record는 기존 `StorageError` 계약에 따라 panic 없이 실패해야 하며, SQL layer가 이를 성공으로 삼으면 안 됩니다.
- 이 task는 transactions를 지원하지 않으므로 command-level atomicity를 제공하지 않습니다. 여러 statement를 한 `db exec`에서 실행하다가 중간 statement가 실패하면, 실패 전까지 성공한 CREATE TABLE 또는 INSERT statement는 durable하게 남아야 합니다.
- 실패한 statement는 partial SQL logical record를 append하면 안 되며, 실패 statement 이후 statement는 실행하면 안 됩니다. 실패 command의 stdout은 empty string이고 exit code와 stderr는 해당 실패 계약을 따릅니다.

### Restart persistence smoke
```bash
tmp_db="$(mktemp -t pdb-sql-restart.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada');"
cargo run --quiet --bin db -- exec "$tmp_db" "SELECT * FROM users;"
```

두 번째 command의 expected stdout:
```text
id|name
1|ada
```

두 번째 command의 expected stderr는 empty string이고 expected exit code는 `0`입니다.

### Mid-command failure persistence smoke
```bash
tmp_db="$(mktemp -t pdb-sql-mid-fail.XXXXXX)"
cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES ('bad', 'type'); INSERT INTO users VALUES (2, 'bea');"
cargo run --quiet --bin db -- exec "$tmp_db" "SELECT * FROM users;"
```

첫 번째 command의 expected stdout은 empty string이고 expected exit code는 `2`입니다.

첫 번째 command의 expected stderr:
```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

두 번째 command의 expected stdout:
```text
id|name
1|ada
```

두 번째 command의 expected stderr는 empty string이고 expected exit code는 `0`입니다. 이 smoke는 실패 전 성공 statement가 restart 후에도 durable하고, 실패 이후 statement가 실행되지 않았음을 증명합니다.

### Unknown SQL storage record negative path
- Test fixture는 기존 `PageStore`로 `b"legacy"` 같은 SQL prefix 없는 payload를 append한 뒤 `db exec <path> "SELECT * FROM users;"`를 실행해야 합니다.
- Expected stdout는 empty string이고 expected exit code는 `1`입니다.
- Expected stderr:
```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## Candidate Acceptance Criteria
- `db exec <path> <sql>`가 문서화된 최소 SQL subset으로 `CREATE TABLE`, `INSERT`, `SELECT * FROM`을 실행하며 기존 `db --help`, `db help`, unsupported CLI exit contract를 깨지 않습니다.
- 지원 SQL happy path는 입력 statement, expected stdout, empty stderr, exit `0`, insertion-order row output을 `tests/sql_exec.rs`에서 검증합니다.
- Identifier equality는 ASCII case-insensitive lookup과 duplicate 판정으로 검증해야 하며, `Users`/`users` table lookup, `users`/`Users` duplicate table, `id`/`ID` duplicate column case variant assertion을 `tests/sql_exec.rs`에 포함해야 합니다.
- Multiple SELECT output은 header 반복, separator 없음, empty table header-only 출력, failure command의 empty stdout을 `tests/sql_exec.rs`에서 exact stdout으로 검증해야 합니다.
- Unsupported SQL과 malformed statement는 panic 없이 empty stdout, deterministic stderr, exit `2`를 반환하고 `docs/cli_contract.md`와 `docs/sql_subset.md`에 같은 문구로 문서화됩니다.
- Duplicate table, missing table, duplicate column, column count mismatch, type mismatch는 panic 없이 empty stdout, deterministic stderr, exit `2`를 반환하고 `docs/cli_contract.md`, `docs/sql_subset.md`, `tests/sql_exec.rs`에 같은 semantic failure matrix로 추적됩니다.
- schema/catalog와 row data는 기존 page storage primitive를 사용하며 process restart 후 동일 `SELECT * FROM` 결과가 유지됩니다.
- 중간 statement 실패 시 실패 전 성공 statement는 durable하게 남고 실패 이후 statement는 실행되지 않음을 restart 관찰 테스트로 검증해야 합니다.
- Unknown SQL storage record fixture 또는 corrupt page-level fixture는 panic 없이 deterministic failure를 반환하며, 이번 task가 선택한 처리 방식이 테스트와 문서에 반영됩니다.
- `docs/file_format.md` 또는 `docs/sql_subset.md`는 page format compatibility와 SQL logical record version/tag/encoding을 명시합니다.

## 검증 계획
- Commands는 managed repo root에서 실행해야 합니다.
- `./scripts/verify`: 기존 baseline인 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo run --bin db -- --help`가 계속 통과함을 증명합니다.
- `cargo test --test sql_exec`: SQL happy path, identifier case variant assertion, multi-SELECT stdout, empty table SELECT, unsupported SQL, malformed SQL, semantic failure matrix, restart persistence, mid-command failure persistence, unknown SQL storage record negative path를 증명합니다.
- `cargo test --test cli_contract`: 기존 help/unsupported CLI contract와 새 `exec <path> <sql>` command surface가 충돌하지 않음을 증명합니다.
- `tmp_db="$(mktemp -t pdb-sql-smoke.XXXXXX)" && cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;"`: CLI smoke stdout이 `id|name\n1|ada\n2|bea\n`, stderr empty, exit `0`임을 증명합니다.
- 기대 증거는 scheduler run report의 command output, `tests/sql_exec.rs` assertions, `tests/cli_contract.rs` assertions, semantic failure별 stdout/stderr/exit code assertion, 최종 phase report의 verification section입니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: 기존 `PageStore` opaque record API만으로 SQL logical record evolution을 표현해야 하므로, payload encoding을 지나치게 일반화하면 후속 호환성 리스크가 생깁니다.
- SQL syntax 또는 storage compatibility 요구가 현재 repo reality와 충돌하면 worker는 scope를 임의 변경하지 말고 conflict를 report해야 합니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
