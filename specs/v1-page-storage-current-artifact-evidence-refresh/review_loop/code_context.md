# Code Context Evidence

- available: true
- repo_root: <repo-root>
- head_sha: 02632eed38ac83e4091f23dca8f2419efc076d3f
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-20T08:22:52.260370+00:00
- selected_files: tests/page_storage.rs, docs/file_format.md, docs/v1_acceptance.md, scripts/verify, src/storage.rs, specs/v1-page-storage-record-format/spec.md, specs/v1-page-storage-record-format/contracts.md, specs/v1-page-storage-record-format/final_review.md, AGENTS.md, work_queue/progress.md

## Omitted Reasons
- .codex/agents/code-reviewer.toml: context_char_limit
- docs/history_archives/history.md: context_char_limit
- open/stale: not_git_tracked
- project_manager/tasks/task-2026-05-16-13-58-47-v1-page-storage-record-format/evidence/recovery_20260517_184449_page_storage_evidence/final-verification.json: not_git_tracked
- project_manager/tasks/task_status_events.jsonl: not_git_tracked
- project_manager/tasks/tasks.json: not_git_tracked
- scripts/verify_page_storage_acceptance: not_git_tracked
- test/evidence/doc: not_git_tracked

## File Excerpts

### tests/page_storage.rs
- excerpt_chars: 4000
- clipped: true

```text
use persistent_db_core::storage::{PageStore, StorageError};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const PAGE_SIZE: usize = 4096;
const FILE_MAGIC: &[u8; 8] = b"PDBV1\0\0\0";
const PAGE_MAGIC: &[u8; 4] = b"PDPG";
const FORMAT_VERSION_OFFSET: usize = 8;
const PAGE_COUNT_OFFSET: usize = 16;
const DATA_PAGE_HEADER_SIZE: usize = 16;
const FIRST_RECORD_LENGTH_OFFSET: u64 = (PAGE_SIZE + DATA_PAGE_HEADER_SIZE) as u64;

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_page_storage_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn assert_storage_error<T: std::fmt::Debug>(
    result: Result<T, StorageError>,
    expected: StorageError,
) {
    let actual = result.expect_err("operation should return a deterministic storage error");
    assert_eq!(expected, actual);
}

fn write_header(path: &PathBuf, version: u16, page_count: u64) {
    let mut page = vec![0u8; PAGE_SIZE];
    page[0..8].copy_from_slice(FILE_MAGIC);
    page[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&version.to_le_bytes());
    page[10..12].copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
    page[12..16].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    page[PAGE_COUNT_OFFSET..PAGE_COUNT_OFFSET + 8].copy_from_slice(&page_count.to_le_bytes());
    fs::write(path, page).expect("header fixture should be written");
}

fn write_header_and_data_page(path: &PathBuf) {
    write_header(path, 1, 2);

    let mut page = vec![0u8; PAGE_SIZE];
    page[0..4].copy_from_slice(PAGE_MAGIC);
    page[4..6].copy_from_slice(&1u16.to_le_bytes());
    page[6..8].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[8..10].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[10..12].copy_from_slice(&0u16.to_le_bytes());

    OpenOptions::new()
        .append(true)
        .open(path)
        .expect("fixture should open")
        .write_all(&page)
        .expect("data page fixture should be written");
}

#[test]
fn append_read_preserves_order_and_bytes() {
    let path = temp_db_path("append_read_preserves_order_and_bytes");

    let result = (|| {
        let mut store = PageStore::open(&path)?;
        store.append_record(b"alpha")?;
        store.append_record(b"beta")?;

        let records = store.read_records()?;
        assert_eq!(vec![b"alpha".to_vec(), b"beta".to_vec()], records);

        let bytes = fs::read(&path).expect("page file should be readable");
        assert_eq!(
            0,
            bytes.len() % PAGE_SIZE,
            "file length must be page aligned"
        );
        assert_eq!(FILE_MAGIC, &bytes[0..8]);
        assert_eq!(
            &1u16.to_le_bytes(),
            &bytes[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2]
        );
        assert_eq!(PAGE_MAGIC, &bytes[PAGE_SIZE..PAGE_SIZE + 4]);
        assert_eq!(
            &5u32.to_le_bytes(),
            &bytes[FIRST_RECORD_LENGTH_OFFSET as usize..FIRST_RECORD_LENGTH_OFFSET as usize + 4]
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("append/read should succeed");
}

#[test]
fn reopen_reads_previously_appended_records() {
    let path = temp_db_path("reopen_reads_previously_appended_records");

    let result = (|| {
        {
            let mut store = PageStore::open(&path)?;
            store.append_record(b"alpha")?;
            store.append_record(b"beta")?;
        }
```

### docs/file_format.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 Page File Format

## Page Size And Numbering

V1 page files use fixed 4096-byte pages. Page `0` is the file header page. Data pages start at page `1` and continue in append order. The file length must always be an exact multiple of 4096 bytes.

## File Header Page

All multi-byte integer fields are little-endian.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 8 | File magic `PDBV1\0\0\0` |
| 8 | 2 | Format version, currently `1` |
| 10 | 2 | Page size as `u16`, currently `4096` |
| 12 | 4 | Page size as `u32`, currently `4096` |
| 16 | 8 | Total page count, including the file header page |
| 24 | 4072 | Reserved, zero-filled in new files |

## Data Page Layout

Each data page stores opaque byte records in append order.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | Data page magic `PDPG` |
| 4 | 2 | Page format version, currently `1` |
| 6 | 2 | Data page header size, currently `16` |
| 8 | 2 | Used byte offset from the start of the page |
| 10 | 2 | Record count in this page |
| 12 | 4 | Reserved, zero-filled in new pages |
| 16 | variable | Record stream |

## Record Encoding

Records are encoded as `u32 little-endian length` followed by exactly that many payload bytes. Payloads are opaque bytes; empty payloads, UTF-8 text, and arbitrary binary bytes are all valid. A single record must fit in one data page after the 16-byte page header and 4-byte length prefix. Overflow pages are not part of V1.

## SQL Logical Records

The SQL executor does not change the page header, data page header, or opaque
record framing. SQL catalog and row data live above `PageStore` as opaque record
payloads documented in `docs/sql_subset.md`.

SQL payloads are UTF-8 compatible and start with the prefix `PDBSQL1\0`.
The byte after the prefix is the SQL logical record kind: `C` for catalog, `R`
for row, `E` for secondary-index backfill entry, `X` for committed
secondary-index metadata, and `I` for one atomic post-index row plus its
embedded secondary-index entries. `U` updates one existing row slot and `D`
tombstones one existing row slot. Catalog records include table name and ordered
column metadata. Row records include table name and ordered typed values.
Arbitrary records without the SQL prefix are valid page-storage payloads, but
they are not valid SQL database records and are rejected by `db exec` with the
documented invalid SQL storage record error.

Catalog records may include an optional primary-key extension after the ordered
column metadata: byte tag `P` followed by a little-endian `u16` zero-based
column index. The referenced column must be `INT`. Catalog records without this
extension are valid row-only SQL catalogs and load as tables without a primary
key.

Primary indexes are not persisted as separate page records, sidecar files, or
background metadata. `db exec` rebuilds the in-memory primary index from durable
row records when the database is opened. A primary-key table with duplicate
persisted key values is treated as corrupt SQL logical data and fails with the
existing invalid SQL storage record error. Because no separate index metadata is
stored, missing index metadata is not a V1 failure mode.

Secondary indexes are persisted as append-only SQL logical records above the
same page framing. Existing no-index databases containing only `C` and `R`
records remain compatible: they reopen normally, and a later `CREATE INDEX`
backfills existing rows.

`CREATE INDEX` writes all backfill `E` records first, then writes the final `X`
metadata record as the commit marker. The `build_id` in `E` and `X` is the
durable SQL logical-record count before that `CREATE INDEX` appends anything.
An `E` record without a matching committed `X(build_id, index_name)` is an
orphan interrupted build entry; `db exec` and `db check` ignore it. Retrying the
same index name after an interrupted build writes a fresh build id and fresh
`E` records before the final `X`.

Committed secondary-index metadata record:

```text
PDBSQL1\0
X
u64 buil
```

### docs/v1_acceptance.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 Acceptance Guide

Evidence id: `evidence-v1-acceptance-docs`

Gate source at task handoff: `autopilot/ssot/current-artifact.md`, specifically the Launch Gate Evidence Contract and Evidence Requirements sections. This guide maps that source to current repo evidence without treating progress projection as proof.

## Gate Evidence Map

| Gate id | Requirement id | Evidence path | Verification command or manual review evidence | Current status |
| --- | --- | --- | --- | --- |
| `gate-v1-cli-smoke` | `req-v1-cli-help-smoke` | `docs/cli_contract.md`; `src/main.rs`; `tests/cli_contract.rs` | `scripts/verify`; `cargo run --bin db -- --help`; `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-cli-smoke` | `req-v1-cli-dispatch-tests` | `tests/cli_contract.rs` | `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-page-storage-restart` | `src/storage.rs`; `tests/page_storage.rs` | `cargo test --test page_storage`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-record-format-doc` | `docs/file_format.md` | Manual review of documented page, SQL logical record, and WAL sidecar compatibility notes | `verified_current_run` |
| `gate-v1-sql-schema-exec` | `req-v1-sql-exec-examples` | `docs/sql_subset.md`; `tests/sql_exec.rs` | `cargo test --test sql_exec`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-primary-index-proof` | `tests/primary_index.rs`; `src/index.rs`; `docs/sql_subset.md` | `cargo test --test primary_index`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-secondary-index-proof` | `tests/secondary_index.rs`; `src/sql.rs`; `src/index.rs`; `docs/cli_contract.md`; `docs/file_format.md` | `cargo test --test secondary_index -- --nocapture`; included in `scripts/verify`; manual review of persisted `E`/`X`/`I` record docs and `db check` invariant coverage | `verified_current_run` |
| `gate-v1-transactions-wal-recovery` | `req-v1-wal-recovery-proof` | `tests/wal_recovery.rs`; `docs/file_format.md` | `cargo test --test wal_recovery`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-crash-testing` | `req-v1-crash-matrix-output` | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md`; `target/crash_matrix/` when generated | `scripts/verify_crash_matrix` when crash-matrix evidence is required; crash tests are also covered by `scripts/verify` if present in the normal test suite | `verified_current_run` |
| `gate-v1-differential-property-tests` | `req-v1-differential-property-proof` | `tests/differential_property.rs`; `scripts/verify_differential_property`; `target/differential_property/` only when a mismatch artifact is generated | `scripts/verify_differential_property`; blocker: no current passing-run deterministic seed-capture artifact is produced by the existing test command | `seed_capture_missing` |
| `gate-v1-db-check-invariants` | `req-v1-db-check-proof` | `docs/cli_contract.md`; `tests/db_check.rs` | `cargo test --test db_check`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-bench-docs-acceptance` | `req-v1-benchmark-lower-bounds` | `docs/benchmarks.md`; `docs/performance_report.md`; `scripts/verify_bench_acceptance`; `target/bench_acceptance/section14-benchmark-acceptance.json` | `db bench`; `scripts/verify_bench_acceptance`; Section 14 requirement IDs `METRIC-14-1`, `METRIC-14-2`, `METRIC-14-3`, `METRIC-14-4`, `FAIL-14-5`; final report evidence id `evidence-v1-benchmark-lower-bounds` | `verified_current_run` |
| `gate-v1-bench-docs-acceptance` | `req-v1-acceptance-docs` | `docs/v1_acceptance.md`; `docs/cli_contract.md`; `docs/bug_diary.md` | Manual review of this guide against `autopilot/ssot/current-artifact.md`; Section 14 docs traceability IDs `EVID-15`, `EVID-16-7`; final report evidence id `evidence-v1-acceptance-docs` | `verified_current_run` |

## Acceptance Boundary

V1 remains a s
```

### scripts/verify
- excerpt_chars: 217
- clipped: false

```text
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin db -- --help
```

### src/storage.rs
- excerpt_chars: 4000
- clipped: true

```text
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const PAGE_SIZE: usize = 4096;
pub const FILE_MAGIC: &[u8; 8] = b"PDBV1\0\0\0";
pub const DATA_PAGE_MAGIC: &[u8; 4] = b"PDPG";

const FORMAT_VERSION: u16 = 1;
const FILE_HEADER_PAGE_COUNT_OFFSET: usize = 16;
const DATA_PAGE_HEADER_SIZE: usize = 16;
const DATA_PAGE_USED_OFFSET: usize = 8;
const DATA_PAGE_RECORD_COUNT_OFFSET: usize = 10;
const RECORD_LENGTH_SIZE: usize = 4;
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;
const WAL_CHECKSUM_OFFSET: usize = 32;
const WAL_CHECKSUM_END: usize = 36;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    TruncatedFile,
    TruncatedPage,
    InvalidMagic,
    UnsupportedVersion,
    RecordTooLarge,
    CorruptRecordLength,
    Io,
}

impl From<std::io::Error> for StorageError {
    fn from(_: std::io::Error) -> Self {
        StorageError::Io
    }
}

#[derive(Debug)]
pub struct PageStore {
    path: PathBuf,
    next_wal_frame_id: u64,
    durable_record_count: u64,
    page_count: u64,
    last_page_used: usize,
    last_page_record_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckStorageSnapshot {
    pub records: Vec<Vec<u8>>,
    pub record_count: u64,
}

impl PageStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            let mut file = OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)?;
            write_file_header(&mut file, 1)?;
            file.flush()?;
        } else {
            validate_file(&path)?;
        }

        replay_wal(&path)?;

        let next_wal_frame_id = next_wal_frame_id(&wal_path(&path))?;
        let durable_record_count = total_record_count(&path)?;
        let (page_count, last_page_used, last_page_record_count) = append_cursor(&path)?;

        Ok(Self {
            path,
            next_wal_frame_id,
            durable_record_count,
            page_count,
            last_page_used,
            last_page_record_count,
        })
    }

    pub fn append_record(&mut self, payload: &[u8]) -> Result<(), StorageError> {
        if payload.len() > max_record_payload_len() {
            return Err(StorageError::RecordTooLarge);
        }

        append_wal_frame(
            &wal_path(&self.path),
            self.next_wal_frame_id,
            self.durable_record_count,
            payload,
        )?;
        append_record_to_file_with_cursor(
            &self.path,
            &mut self.page_count,
            &mut self.last_page_used,
            &mut self.last_page_record_count,
            payload,
        )?;
        self.next_wal_frame_id = self
            .next_wal_frame_id
            .checked_add(1)
            .ok_or(StorageError::Io)?;
        self.durable_record_count = self
            .durable_record_count
            .checked_add(1)
            .ok_or(StorageError::Io)?;
        Ok(())
    }

    pub fn read_records(&mut self) -> Result<Vec<Vec<u8>>, StorageError> {
        validate_file(&self.path)?;

        let mut file = OpenOptions::new().read(true).open(&self.path)?;
        let page_count = read_page_count(&mut file)?;
        let mut records = Vec::new();

        for page_index in 1..page_count {
            let page = read_page(&mut file, page_index)?;
            read_data_page_records(&page, &mut records)?;
        }

        Ok(records)
    }
}

fn append_cursor(path: &Path) -> Result<(u64, usize, u16), StorageError> {
    validate_file(path)?;
    let mut file = OpenOptions::new().read(true).open(path)?;
    let page_count = read_page_count(&mut file)?;
    if page_count == 1 {
        return Ok((
```

### specs/v1-page-storage-record-format/spec.md
- excerpt_chars: 4000
- clipped: true

```text
# 고정 크기 페이지 저장소와 레코드 재시작 검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-16-13-58-47
- Task ID: task-2026-05-16-13-58-47-v1-page-storage-record-format
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 고정 크기 페이지 저장소와 레코드 재시작 검증
- Artifact: v1-page-storage-record-format

## 목표
- 현재 persistent-db-core는 CLI skeleton과 CLI 계약 근거는 있으나, records가 disk page에 저장되고 process restart 뒤에도 동일하게 읽힌다는 durable storage 증거가 없습니다. SQL, index, WAL, recovery 작업은 모두 이 저장소 primitive 없이는 검증 가능한 제품 가치로 이어지기 어렵습니다.

## 지금 해야 하는 이유
- Current objective는 CLI 다음 순서로 durable page storage를 요구하고, current plan의 high priority gap 중 CLI 이후 첫 번째 기반 gap입니다. Queue Snapshot은 비어 있고, Root Progress Projection은 gate-v1-disk-page-storage의 두 evidence requirement가 모두 open이라고 보고합니다.

## 기대 산출물 변화
- managed repo에 deterministic page file 생성, fixed record encoding, append/read/reopen 동작, restart test, file/page/record format compatibility note가 추가됩니다.

## 의도한 변경 대상
- `src/storage.rs`: fixed-size page file 생성, record append/read, reopen read를 담당하는 storage primitive.
- `tests/page_storage.rs`: append/read/reopen, record encoding, file format failure mode를 검증하는 deterministic integration tests.
- `docs/file_format.md`: V1 page file, page header 또는 slot layout, record encoding, compatibility note.
- `src/main.rs`: 새 storage 전용 user-facing CLI command를 추가하지 않습니다. 기존 `db --help` CLI contract가 유지되는지만 smoke check로 확인합니다.

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 178aa445c286aee9929ed7e0b8a14bd7e3d6b2e0
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, work_queue/progress.md, AGENTS.md, docs/history_archives/history.md, .codex/skills/spec-creator/SKILL.md, .codex/skills/spec-reviewer/SKILL.md, Cargo.toml, docs/cli_contract.md, docs/v1_spec.md, specs/v1-bootstrap-cli-contract/code_review.md, specs/v1-bootstrap-cli-contract/contracts.md

## Risk flags
- format_compatibility_sensitive
- requires_existing_cli_contract_preservation
- pre_launch_no_existing_user_data_assumed

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- ssot/current-objective.md: metric-v1-durable-storage는 records가 deterministic on-disk page storage를 통해 restart 뒤에도 살아남아야 한다고 정의합니다.
- ssot/current-plan.md: gap-v1-page-storage-record-format은 high priority이며 page file creation, fixed record encoding, restart read verification을 next candidate hint로 둡니다.
- ssot/current-artifact.md: gate-v1-disk-page-storage는 req-v1-page-storage-restart와 req-v1-record-format-doc을 완료 조건으로 둡니다.
- Root Progress Projection: gate-v1-disk-page-storage status=open, missing_requirement_ids=[req-v1-page-storage-restart, req-v1-record-format-doc].
- Active Managed Repo Snapshot: work_queue/progress.md는 page storage와 record format implementation evidence가 아직 없다고 기록합니다.
- Queue Snapshot: []로 현재 중복 active/reserved task가 없습니다.
- Gap Evidence Cache: verified task는 v1-bootstrap-cli-contract 1개뿐이고 page storage verified evidence는 없습니다.
- autopilot/ssot/current-objective.md
- autopilot/ssot/current-plan.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- autopilot/project_manager/specs/v1-page-storage-record-format/review_loop/design.md
- AGENTS.md
- Cargo.toml
- src/main.rs
- tests/cli_contract.rs
- docs/cli_contract.md
- docs/v1_spec.md
- work_queue/progress.md
- metric-v1-durable-storage
- gap-v1-page-storage-record-format
- gate-v1-disk-page-storage
- req-v1-page-storage-restart
- req-v1-record-format-doc
- HELP
- main
- [[bin]] name = "db"
- reserved_future_command_remains_unsupported
- autopilot/project_manager/tasks/tasks.
```

### specs/v1-page-storage-record-format/contracts.md
- excerpt_chars: 3153
- clipped: false

```text
# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: `src/storage.rs`의 page storage primitive와 `docs/file_format.md`의 file/page/record format 문서.
- 생성 대상 테스트 또는 verification output: `tests/page_storage.rs`, `cargo test`, `cargo test --test page_storage`, `cargo run --bin db -- --help` 결과.
- 생성 대상 리포트 업데이트: run report의 acceptance criterion별 proof entry, command output summary, 변경 파일 목록, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- 각 Candidate Acceptance Criteria item은 test output, command output, document path, run report evidence 또는 explicit blocker와 연결되어야 합니다.
- spec hardening 중 candidate acceptance criteria를 generic completion wording으로 약화하거나 병합하거나 대체하면 안 됩니다.
- 새 storage 전용 user-facing CLI command는 이 task의 산출물이 아닙니다. CLI evidence는 기존 `db --help` contract 보존 확인으로 제한합니다.
- 고정 크기 page file 생성과 deterministic binary append/read는 `cargo test --test page_storage`의 append/read test output과 `tests/page_storage.rs`로 증명해야 합니다.
- restart read verification은 같은 database path를 reopen한 뒤 append 순서와 byte 값이 동일함을 검증하는 `cargo test --test page_storage` test output으로 증명해야 합니다.
- record encoding은 empty payload, ASCII payload, binary byte payload를 포함한 `tests/page_storage.rs` assertions로 증명해야 합니다.
- file/page/record format 문서는 `docs/file_format.md`에서 page size, file header 또는 page header, slot layout 또는 record length layout, endian, record encoding, compatibility note 섹션을 확인해 증명해야 합니다.
- compatibility note는 V1 pre-launch에서 기존 user data backward compatibility를 보장하지 않는 전제와, 이후 format 변경 시 문서와 테스트 갱신이 필요하다는 제약을 포함해야 합니다.
- failure-mode behavior는 truncated file/page, invalid magic/header, unsupported format version, page overflow record, corrupt record length가 각각 panic이나 silent success 없이 deterministic error를 반환함을 `cargo test --test page_storage`의 별도 assertions로 증명해야 합니다.
- unsupported format version 증거는 invalid magic/header 증거와 병합할 수 없으며, format version field를 지원하지 않는 값으로 바꾼 독립 test output과 assertion으로 연결해야 합니다.
- 전체 회귀는 `cargo test` 통과로 증명해야 합니다.
- CLI contract 보존은 `cargo run --bin db -- --help` command output과 기존 CLI contract test 통과로 증명해야 합니다.
- 최종 run report는 위 항목별 command output, document path, evidence summary를 acceptance criterion별로 연결해야 합니다.

## Visual Evidence Contract
- 적용 제외입니다. 이 task는 Rust CLI storage/file-format 작업이며 reference bundle, DOM route, viewport, screenshot, UX design review를 요구하지 않습니다.
- Visual evidence 부재는 blocker가 아닙니다. 대신 deterministic test output, command output, `docs/file_format.md`, run report evidence가 필수 proof layer입니다.

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
```

### specs/v1-page-storage-record-format/final_review.md
- excerpt_chars: 2048
- clipped: false

```text
Verdict: PASS

## Scope

- Final execution closure for `task-2026-05-16-13-58-47-v1-page-storage-record-format`.
- Verified approved spec and contract: `specs/v1-page-storage-record-format/spec.md`, `specs/v1-page-storage-record-format/contracts.md`.
- Verified implementation artifacts: `src/lib.rs`, `src/storage.rs`, `tests/page_storage.rs`, `docs/file_format.md`.
- Visual evidence is not applicable for this Rust CLI storage/file-format task; deterministic command and test evidence is the proof layer.

## Closure Checks

- Fixed 4096-byte page file creation exists in `src/storage.rs`.
- Opaque byte record append/read preserves append order and byte values.
- Reopen/restart read verification exists in `tests/page_storage.rs`.
- Record encoding test covers empty payload, ASCII payload, and binary bytes.
- Failure-mode tests separately cover truncated file, truncated page, invalid file/data page magic, unsupported format version, page overflow append, and corrupt record length.
- `docs/file_format.md` documents page size, file header, data page layout, little-endian record encoding, validation errors, and compatibility constraints.
- No storage-specific user-facing CLI command was added.
- Protected `ssot/` and `policies/` areas were not modified.

## Open Items

None.

## Verification Evidence

- `cargo fmt --check`: pass.
- `cargo clippy --all-targets --all-features -- -D warnings`: pass.
- `cargo test --test page_storage`: pass, 10 tests.
- `cargo test`: pass, including 4 CLI contract tests and 10 page storage tests.
- `cargo run --bin db -- --help`: pass, existing CLI help contract preserved.

## Remote State

- Current branch: `task-2026-05-16-13-58-47-v1-page-storage-record-format`.
- Remote: `origin` configured as `https://github.com/Youngerjesus/persistent-db-core.git`.
- Final execution prepared the managed repo artifacts for commit/push and wrote scheduler handoff evidence under the task evidence directory.

## Next Action

Hand off to independent `final_verify`.

## Updated At

2026-05-16T14:45:23+09:00
```

### AGENTS.md
- excerpt_chars: 4000
- clipped: true

```text
# AGENTS (persistent-db-core)

Primary audience: coding agents and maintainers working inside this product repo.

## Product Direction

Build a small, deterministic Rust CLI database binary named `db`. V1 should grow toward durable single-process database behavior while keeping the documented CLI contract stable.

## Engineering Rules

- Keep changes scoped to the active task and its spec package.
- Prefer deterministic behavior, deterministic tests, and explicit persisted-data fixtures over implicit state.
- Treat persisted data compatibility, CLI output, exit codes, and documented error behavior as stable contracts once introduced.
- Make failure modes explicit. Avoid panics for user-facing CLI errors and persisted-data handling unless the invariant violation is unrecoverable programmer error.
- Do not broaden dependencies without a task-level reason. For V1, standard library implementations are preferred unless the spec explicitly calls for a crate.
- Do not add network services, background daemons, or remote-service requirements for V1.
- Keep CLI output stable once documented in `docs/cli_contract.md` or covered by integration tests.

## SDD Workflow

- Start each implementation from the task-provided `spec.md` and `contracts.md`.
- Treat `contracts.md` as the completion contract: every listed behavior, non-goal, required check, and acceptance artifact must be satisfied or explicitly blocked.
- Use repo-local `specs/**` as product history, templates, or local execution artifacts unless the active task explicitly selects one as the current spec.
- Before editing, identify the affected product contract: CLI behavior, persisted data compatibility, documented errors, tests, or durable docs.
- Preserve existing public behavior unless the active spec explicitly changes it.
- Add or update focused tests with behavior changes, including negative and edge cases for persisted data, recovery, indexing, and CLI contract work.
- Update durable docs only when the user-facing or compatibility contract changes.
- If the spec, contract, and repo reality conflict, stop and report the conflict instead of silently changing scope.
- Treat SDD pipeline `result_*.md` files as phase status reports. Their status line or `PM_RESULT:` sentinel expresses next owner and readiness; it is not a substitute for contract evidence.
- Treat latest review/report files as verifier or reviewer SSOT: `qa_prep_review.md`, `impl_review.md`, `impl_brake_review.md`, `code_review.md`, and `final_review.md`.
- Execution and repair phases may read the latest review/report files as input, but must not check off, delete, or overwrite reviewer findings unless the phase explicitly owns that review/report.
- On retry or repair re-entry, read the latest review/report first and repair only the actionable open items such as `Repair Targets`, `Must Fix Now`, `Next Action`, or `Verify Risks`.
- Preserve previous review/report contents in the matching `.history.md` file when the owning verifier or reviewer refreshes the latest SSOT; the latest file should contain only current open findings and verdict context.
- Use `development_state.md` only as high-density implementation state between passes. It supports handoff, but completion still depends on the active contract, latest review/report state, and required verification evidence.
- Task completion is blocked until `scripts/verify` and any contract-required checks pass.

## Architecture Boundary

Current structure is intentionally small:

- `src/main.rs`: CLI entrypoint and current implementation surface.
- `tests/`: integration and behavior tests; prefer black-box CLI behavior here.
- `docs/`: durable product documentation and contract references.
- `docs/history_archives/history.md`: append-only product history.
- `work_queue/progress.md`: current product progress view.
- `specs/`: product history or local execution artifacts from completed work.

No deeper module boundary map exists yet. When meaningful layers or modules are
```

### work_queue/progress.md
- excerpt_chars: 582
- clipped: true

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, disk-backed secondary-index equality/range proof, mutation-maintained secondary-index UPDATE/DELETE proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, `db check` invariant validation for existing database files, SQLite-backed differential/property evidence for the supported SQL subset, and
```
