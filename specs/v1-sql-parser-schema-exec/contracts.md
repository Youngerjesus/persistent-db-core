# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.
- 이 task는 CLI-only Rust database 작업입니다. 브라우저 기반 검증 산출물은 acceptance evidence가 아닙니다.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: `db exec <path> <sql>` CLI path, 최소 SQL parser/schema catalog/executor, SQL logical record docs, CLI contract docs.
- 생성 대상 테스트 또는 verification output: `./scripts/verify`, `cargo test --test sql_exec`, `cargo test --test cli_contract`, CLI smoke command output.
- 생성 대상 리포트 업데이트: run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.
- 구현이 `spec.md`의 최소 SQL subset, semantic failure matrix, stderr 문구, exit code, persistence contract, verification command 중 하나라도 약화하면 미완료입니다.

## CLI Completion Contract
- `db exec <path> <sql>`는 exact command surface입니다. stdin SQL, interactive shell, `db open`, `db check`, `db bench` 구현은 이 task의 completion 조건이 아닙니다.
- 기존 `db --help`와 `db help`는 exit `0`, empty stderr, identical help stdout을 유지해야 합니다.
- 기존 unsupported CLI input은 exit `2`, empty stdout, documented stderr 형식을 유지해야 합니다.
- Help 문서는 `exec <path> <sql>`를 supported command로 반영해야 하며, reserved future command 문구와 테스트가 이에 맞게 갱신되어야 합니다.

## SQL Grammar Contract
- 지원 grammar는 아래 세 가지뿐입니다.

```text
CREATE TABLE <table_name> (<column_name> INT|TEXT[, <column_name> INT|TEXT]*);
INSERT INTO <table_name> VALUES (<value>[, <value>]*);
SELECT * FROM <table_name>;
```

- Identifier는 `[A-Za-z_][A-Za-z0-9_]*`만 허용합니다.
- Table name과 column name equality는 입력 spelling 보존과 별개로 ASCII case-insensitive입니다. Catalog lookup, duplicate table 판정, duplicate column 판정, missing table 판정은 모두 ASCII case-insensitive 비교를 사용해야 합니다.
- Error message와 SELECT header는 catalog 또는 입력에서 이미 정한 spelling을 유지합니다. Duplicate table과 duplicate column 오류는 충돌을 일으킨 새 입력 spelling을 error target으로 출력하고, missing table 오류는 lookup에 실패한 입력 spelling을 출력합니다.
- Type은 `INT`와 `TEXT`만 허용합니다.
- `TEXT` literal은 single quote로 감싼 UTF-8 string만 허용하며 escape sequence, embedded single quote, `|`, newline, carriage return은 지원하지 않습니다.
- `SELECT id FROM users;`, `SELECT * FROM users WHERE id = 1;`, `INSERT INTO users (id) VALUES (1);`, `DROP TABLE users;`는 unsupported SQL입니다.
- `CREATE TABLE users id INT);`처럼 supported statement shape가 깨진 입력은 malformed SQL입니다.

## Output And Error Contract
- Successful `SELECT * FROM users;`는 column header를 먼저 출력하고 row를 successful `INSERT` append order로 출력합니다.
- Field delimiter는 `|`이고 각 line은 `\n`으로 끝납니다.
- 하나의 successful `db exec` 안에 여러 `SELECT * FROM ...;`가 있으면 각 SELECT는 독립 result set으로 header line을 반복하고 row를 이어서 출력합니다. Result set 사이에는 blank line, separator, count line을 넣지 않습니다.
- Empty table SELECT expected stdout은 header line만 포함합니다.
- 하나의 `db exec` 호출 중 어떤 statement라도 실패하면 command stdout은 empty string이어야 합니다. 이전 statement가 SELECT output을 만들었더라도 실패 command는 partial stdout을 출력하지 않습니다.
- Happy path expected stdout은 아래와 같습니다.

```text
id|name
1|ada
2|bea
```

- Happy path expected stderr는 empty string이고 expected exit code는 `0`입니다.
- Multiple SELECT expected stdout 예시는 아래와 같습니다. SQL은 `CREATE TABLE users (id INT, name TEXT); SELECT * FROM users; INSERT INTO users VALUES (1, 'ada'); SELECT * FROM users;`입니다.

```text
id|name
id|name
1|ada
```

- Unsupported SQL expected stdout는 empty string이고 expected exit code는 `2`입니다.

```text
error: unsupported SQL statement: SELECT id FROM users;
hint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ...;
```

- Malformed SQL expected stdout는 empty string이고 expected exit code는 `2`입니다.

```text
error: malformed SQL statement: CREATE TABLE users id INT);
hint: terminate each statement with ';' and use the documented SQL subset.
```

- Supported grammar 안에서 발생하는 semantic failure는 user SQL error로 처리합니다. 아래 모든 항목은 expected stdout empty string, expected exit code `2`, panic 없음, `docs/cli_contract.md`와 `docs/sql_subset.md`의 동일 문구 문서화, `cargo test --test sql_exec` assertion을 필수로 요구합니다.
- Duplicate table 입력: `CREATE TABLE users (id INT); CREATE TABLE users (name TEXT);`

```text
error: SQL semantic error: table already exists: users
hint: use a new table name for CREATE TABLE in this database.
```

- Duplicate table case variant 입력 `CREATE TABLE users (id INT); CREATE TABLE Users (name TEXT);`는 같은 오류 계약을 따르되 stderr target은 `Users`여야 합니다.
- Missing table 입력: `INSERT INTO missing VALUES (1);`

```text
error: SQL semantic error: table not found: missing
hint: create the table before INSERT or SELECT.
```

- `SELECT * FROM missing;`도 missing table과 같은 stdout, stderr, exit code 계약을 따라야 합니다.
- `CREATE TABLE Users (id INT); SELECT * FROM users;`는 missing table이 아니라 같은 table lookup으로 성공해야 합니다.
- Duplicate column 입력: `CREATE TABLE users (id INT, id TEXT);`

```text
error: SQL semantic error: duplicate column: id
hint: column names in a table must be unique.
```

- Duplicate column case variant 입력 `CREATE TABLE users (id INT, ID TEXT);`는 같은 오류 계약을 따르되 stderr target은 `ID`여야 합니다.
- Column count mismatch 입력: `CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1);`

```text
error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1
hint: INSERT values must match the table schema exactly.
```

- Type mismatch 입력: `CREATE TABLE users (id INT); INSERT INTO users VALUES ('ada');`

```text
error: SQL semantic error: type mismatch for column id: expected INT, got TEXT
hint: INSERT values must match the declared column types.
```

- `NULL`, constraints, projection column resolution, `WHERE`, `ORDER BY`, `JOIN`, `UPDATE`, `DELETE` 관련 semantic behavior는 out of scope입니다. 해당 입력이 문서화된 supported grammar를 벗어나면 unsupported SQL 또는 malformed SQL 계약으로 실패해야 하며 panic이나 storage-level success로 귀결되면 안 됩니다.
- Unknown SQL storage record expected stdout는 empty string이고 expected exit code는 `1`입니다.

```text
error: invalid SQL storage record: unknown record tag
hint: run against a database file created by this SQL contract or restore from a valid backup.
```

## Persistence Contract
- 기존 page file format은 변경하지 않습니다. `PDBV1\0\0\0`, format version `1`, `PDPG`, fixed 4096-byte pages, opaque record encoding은 기존 계약과 호환되어야 합니다.
- SQL catalog와 row data는 기존 `PageStore::append_record`와 `PageStore::read_records` 위의 opaque payload로 저장해야 합니다.
- SQL logical record는 식별 가능한 version/tag/encoding을 가져야 하며 최소 `PDBSQL1\0` prefix, record kind `catalog` 또는 `row`, table name, column metadata 또는 row values를 문서화해야 합니다.
- 기존 arbitrary PageStore payload는 valid SQL record로 silently accepted되면 안 됩니다. Unknown SQL record tag는 panic 없이 deterministic error로 실패해야 합니다.
- Restart 후 동일 database file에서 `SELECT * FROM users;`가 같은 header와 row를 출력해야 합니다.
- Page-level corruption은 기존 `StorageError` behavior를 유지해야 하며 SQL layer가 이를 성공으로 숨기면 안 됩니다.
- 이 task는 transactions를 지원하지 않으므로 command-level atomicity를 제공하지 않습니다. 여러 statement를 한 `db exec`에서 실행하다가 중간 statement가 실패하면, 실패 전까지 성공한 CREATE TABLE 또는 INSERT statement는 durable하게 남아야 합니다.
- 실패한 statement는 partial SQL logical record를 append하면 안 되며, 실패 statement 이후 statement는 실행하면 안 됩니다.
- Mid-command failure restart 관찰 계약: `CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES ('bad', 'type'); INSERT INTO users VALUES (2, 'bea');`는 exit `2`, empty stdout, type mismatch stderr로 실패해야 하고, 이후 새 process에서 `SELECT * FROM users;`는 `id|name\n1|ada\n`만 출력해야 합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `db exec <path> <sql>`가 문서화된 최소 SQL subset으로 `CREATE TABLE`, `INSERT`, `SELECT * FROM`을 실행하며 기존 CLI help/exit contract를 깨지 않는다는 증거가 필요합니다.
- 지원 SQL happy path는 command, expected stdout, expected stderr, expected exit code, row ordering을 테스트로 검증해야 합니다.
- Identifier equality는 ASCII case-insensitive lookup과 duplicate 판정으로 검증해야 하며, `Users`/`users` table lookup, `users`/`Users` duplicate table, `id`/`ID` duplicate column case variant assertion이 필요합니다.
- Multiple SELECT output은 header 반복, separator 없음, empty table header-only 출력, failure command의 empty stdout을 exact stdout으로 검증해야 합니다.
- schema/catalog와 row data는 기존 page storage primitive를 사용하며 process restart 후 `SELECT * FROM` 결과가 유지된다는 증거가 필요합니다.
- 중간 statement 실패 시 실패 전 성공 statement는 durable하게 남고 실패 이후 statement는 실행되지 않음을 restart 관찰 테스트로 증명해야 합니다.
- Unsupported SQL, malformed statement, unknown SQL storage record는 panic 없이 문서화된 non-zero exit와 stderr 형식으로 처리되어야 합니다.
- Duplicate table, missing table, duplicate column, column count mismatch, type mismatch는 `tests/sql_exec.rs`에서 expected stdout, expected stderr, expected exit code로 각각 검증되어야 합니다.
- `docs/cli_contract.md`, `docs/sql_subset.md`, `docs/file_format.md` 중 해당 계약을 바꾼 문서는 구현과 같은 PR delta에 포함되어야 합니다.

## Required Verification Commands
- 모든 command는 managed repo root에서 실행해야 합니다.
- `./scripts/verify`
  - 증명 항목: baseline fmt, clippy, full test suite, help smoke contract 유지.
  - 기대 evidence: scheduler run report 또는 final report의 command output.
- `cargo test --test sql_exec`
  - 증명 항목: SQL happy path, identifier case variant assertion, multi-SELECT stdout, empty table SELECT, unsupported SQL, malformed SQL, semantic failure matrix, restart persistence, mid-command failure persistence, unknown SQL storage record negative path.
  - 기대 evidence: `tests/sql_exec.rs` assertion output와 command success.
- `cargo test --test cli_contract`
  - 증명 항목: 기존 help/unsupported CLI contract와 새 `exec <path> <sql>` command surface의 문서 일치.
  - 기대 evidence: `tests/cli_contract.rs` assertion output와 command success.
- `tmp_db="$(mktemp -t pdb-sql-smoke.XXXXXX)" && cargo run --quiet --bin db -- exec "$tmp_db" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;"`
  - 증명 항목: CLI smoke stdout `id|name\n1|ada\n2|bea\n`, empty stderr, exit `0`.
  - 기대 evidence: final report의 stdout/stderr/exit code 기록.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Required Verification Commands가 통과하거나 명시적 blocker가 기록되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
