# 트랜잭션 WAL 복구 최소 증거 추가

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-17-23-45-17
- Task ID: task-2026-05-17-23-45-17-v1-transaction-wal-recovery
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 트랜잭션 WAL 복구 최소 증거 추가
- Artifact: v1-transaction-wal-recovery

## 목표
- 현재 `persistent-db-core`는 durable storage, SQL 실행, index 증거는 갖췄지만, committed mutation만 재시작 후 살아남고 rollback 또는 partial mutation은 사라진다는 복구 계약 증거가 없습니다.

## 지금 해야 하는 이유
- Root Progress Projection에서 CLI, storage, SQL, indexes gate는 `projected_complete`이고, 남은 recovery 계열 중 crash matrix는 WAL 복구 의미가 먼저 고정되어야 검증 가능합니다. 따라서 WAL replay의 최소 commit/rollback slice가 다음 의존성 병목입니다.

## 기대 산출물 변화
- Rust CLI `db exec` 경로에서 committed mutation이 재시작 후 조회되는 최소 WAL replay 증거를 추가합니다.
- 공개 CLI에 transaction rollback 명령이 없으면 rollback 또는 incomplete WAL entry 부재 증거는 storage-level deterministic fixture test로 고정합니다.
- WAL 파일 위치, record framing, replay 의미, rollback/incomplete 처리, 기존 database file 호환 동작을 `docs/file_format.md`에 compatibility note로 문서화합니다.

## 의도한 변경 대상
- src/main.rs
- src/lib.rs
- src/storage.rs
- tests/
- docs/file_format.md
- docs/cli_contract.md
- route:db-exec
- flow:wal-replay-recovery

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 8adb253bac616445f6e389e29048e4296f0ce85b
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, src/lib.rs, src/storage.rs, tests/cli_contract.rs, tests/page_storage.rs, tests/primary_index.rs, tests/sql_exec.rs, docs/file_format.md, docs/cli_contract.md

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: `gate-v1-transactions-wal-recovery` 상태가 `open`이고 missing requirement가 `req-v1-wal-recovery-proof`입니다.
- Current Plan: `gap-v1-transaction-wal-recovery`는 high priority이며 next candidate hint가 commit/rollback WAL replay입니다.
- Current Artifact: `gate-v1-transactions-wal-recovery`는 WAL replay tests, commit/rollback tests, recovery transcript, file-state evidence를 요구합니다.
- Managed repo progress: SQL schema/execute path와 primary-key indexed lookup/ordered scan proof가 이미 존재해 recovery layer를 검증할 실행 baseline이 있습니다.
- Queue Snapshot: 현재 active 또는 reserved task가 없습니다.
- persistent-db-core_worktree/main/src/sql.rs
- persistent-db-core_worktree/main/src/storage.rs
- persistent-db-core_worktree/main/src/main.rs
- persistent-db-core_worktree/main/tests/sql_exec.rs
- persistent-db-core_worktree/main/tests/primary_index.rs
- persistent-db-core_worktree/main/docs/cli_contract.md
- persistent-db-core_worktree/main/docs/sql_subset.md
- persistent-db-core_worktree/main/docs/file_format.md
- persistent-db-core_worktree/main/work_queue/progress.md
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- sql::execute
- Database::from_records
- Statement::CreateTable
- Statement::Insert
- Statement::SelectAll
- Statement::SelectPrimaryKey
- PageStore::open
- PageStore::append_record
- PageStore::read_records
- PrimaryIndex
- autopilot/project_manager/tasks/tasks.json#task-2026-05-15-16-06-54-v1-bootstrap-cli-contract:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-16-13-58-47-v1-page-storage-record-format:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-19-38-21-v1-sql-parser-schema-exec:SUCCESS
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-22-43-31-v1-primary-btree-index:SUCCESS
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/spec.md
- autopilot/project_manager/specs/v1-bootstrap-cli-contract/contracts.md
- autopilot/project_manager/specs/v1-page-storage-record-format/spec.md
- autopilot/project_manager/specs/v1-page-storage-record-format/contracts.md
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/spec.md
- autopilot/project_manager/specs/v1-sql-parser-schema-exec/contracts.md
- autopilot/project_manager/specs/v1-primary-btree-index/spec.md
- autopilot/project_manager/specs/v1-primary-btree-index/contracts.md

## 범위
- In scope: selected candidate only.
- In scope: `db exec <path> <sql>`를 통한 committed WAL replay CLI integration test.
- In scope: rollback 또는 incomplete WAL entry가 재시작/reopen 후 durable row로 노출되지 않는 storage-level deterministic test.
- In scope: WAL compatibility note를 `docs/file_format.md`에 추가합니다.
- Out of scope: unrelated breadth features.
- Out of scope: 공개 transaction SQL 문법, network server, multi-process concurrency, crash matrix 전체 확장.

## Canonical Recovery Scenarios
- Scenario A는 CLI-visible completion proof입니다. Test는 temp database path를 생성하고 별도 `db exec` process로 `CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');`를 실행해야 합니다. 이 command는 exit code `0`, stdout `""`, stderr `""`를 가져야 합니다.
- Scenario A는 같은 temp database path를 새 `db exec` process로 다시 열어 `SELECT * FROM users;`를 실행해야 합니다. 이 reopen command는 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 가져야 합니다.
- Scenario B는 rollback 또는 incomplete WAL entry 부재 proof입니다. Test는 deterministic temp database/WAL fixture를 만들고 committed row `1|ada`와 rollback 또는 incomplete row `9|ghost`를 함께 배치한 뒤 reopen/replay를 수행해야 합니다.
- Scenario B의 post-reopen assertion은 `9|ghost`가 조회 결과나 storage row set에 없어야 하며, CLI로 검증할 수 있는 fixture라면 `SELECT * FROM users;`가 exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n`를 반환해야 합니다. CLI로 표현할 수 없는 fixture라면 storage-level assertion으로 row set이 `[(1, "ada")]`와 동등함을 검증하고, 왜 CLI fixture가 아닌지 test 이름 또는 주석에 남깁니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `cargo test`가 통과합니다.
- `./scripts/verify`가 통과합니다.
- `tests/wal_recovery.rs`가 존재하고 `cargo test --test wal_recovery`로 실행됩니다.
- Scenario A의 committed insert set이 재시작 후 WAL replay를 통해 CLI stdout exactly `id|name\n1|ada\n2|bea\n`로 조회됩니다.
- Scenario B의 rollback된 mutation 또는 incomplete WAL entry가 재시작 후 조회 결과나 storage row set에 나타나지 않습니다.
- `docs/file_format.md`의 WAL compatibility note는 WAL 파일명 또는 위치, record layout 또는 framing, replay 순서, committed/rollback/incomplete entry 처리, 기존 database 파일을 열 때의 기대 동작을 모두 기록합니다.
- `docs/cli_contract.md`는 public CLI output, exit code, stderr contract가 변경될 때만 갱신하고, 변경하지 않는다면 final report에 변경 없음 이유를 남깁니다.
- 테스트는 deterministic fixture 또는 temp database를 사용하고 expected row output과 exit behavior를 명시합니다.

## 검증 계획
- Required verification commands:
  - `cargo test`
  - `cargo test --test wal_recovery`
  - `./scripts/verify`
- Required smoke evidence:
  - `cargo run --bin db -- exec <temp-db> "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`
  - `cargo run --bin db -- exec <temp-db> "SELECT * FROM users;"`
- 기대 증거 파일:
  - `tests/wal_recovery.rs`
  - `docs/file_format.md`
  - `docs/cli_contract.md` when public CLI behavior changes
  - scheduler final report 또는 phase result에 위 verification command output, temp database path redaction 여부, WAL file-state evidence summary가 연결되어야 합니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
