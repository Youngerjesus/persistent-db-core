use persistent_db_core::storage::{PageFileWrite, PageStore, StorageError};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const PAGE_SIZE: usize = 4096;
const FILE_MAGIC: &[u8; 8] = b"PDBV1\0\0\0";
const PAGE_MAGIC: &[u8; 4] = b"PDPG";
const FORMAT_VERSION_OFFSET: usize = 8;
const PAGE_COUNT_OFFSET: usize = 16;
const DATA_PAGE_HEADER_SIZE: usize = 16;
const DATA_PAGE_USED_OFFSET: usize = PAGE_SIZE + 8;
const DATA_PAGE_RECORD_COUNT_OFFSET: usize = PAGE_SIZE + 10;
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

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
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

        let mut reopened = PageStore::open(&path)?;
        assert_eq!(
            vec![b"alpha".to_vec(), b"beta".to_vec()],
            reopened.read_records()?
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("reopen read should preserve stored records");
}

#[test]
fn record_encoding_supports_empty_ascii_and_binary_payloads() {
    let path = temp_db_path("record_encoding_supports_empty_ascii_and_binary_payloads");

    let result = (|| {
        let expected = vec![
            b"".to_vec(),
            b"alpha".to_vec(),
            b"beta".to_vec(),
            vec![0x00, 0xff, 0x10],
        ];
        let mut store = PageStore::open(&path)?;
        for payload in &expected {
            store.append_record(payload)?;
        }

        assert_eq!(expected, store.read_records()?);

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("record encoding should round-trip opaque bytes");
}

#[test]
fn truncated_file_returns_error() {
    let path = temp_db_path("truncated_file_returns_error");
    fs::write(&path, b"short").expect("fixture should be written");

    let result = PageStore::open(&path);

    cleanup(&path);
    assert_storage_error(result, StorageError::TruncatedFile);
}

#[test]
fn truncated_page_returns_error() {
    let path = temp_db_path("truncated_page_returns_error");
    write_header(&path, 1, 2);
    OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("fixture should open")
        .write_all(&[0u8; 32])
        .expect("partial page should be written");

    let result = PageStore::open(&path);

    cleanup(&path);
    assert_storage_error(result, StorageError::TruncatedPage);
}

#[test]
fn invalid_magic_returns_error() {
    let path = temp_db_path("invalid_magic_returns_error");
    write_header(&path, 1, 1);
    let mut file = OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("fixture should open");
    file.seek(SeekFrom::Start(0)).expect("seek should succeed");
    file.write_all(b"NOTPDB!!")
        .expect("invalid magic should be written");

    let result = PageStore::open(&path);

    cleanup(&path);
    assert_storage_error(result, StorageError::InvalidMagic);
}

#[test]
fn invalid_data_page_magic_returns_error() {
    let path = temp_db_path("invalid_data_page_magic_returns_error");
    write_header_and_data_page(&path);
    let mut file = OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("fixture should open");
    file.seek(SeekFrom::Start(PAGE_SIZE as u64))
        .expect("seek should succeed");
    file.write_all(b"BAD!")
        .expect("invalid page magic should be written");

    let result = PageStore::open(&path);

    cleanup(&path);
    assert_storage_error(result, StorageError::InvalidMagic);
}

#[test]
fn unsupported_format_version_returns_error() {
    let path = temp_db_path("unsupported_format_version_returns_error");
    write_header(&path, 2, 1);

    let result = PageStore::open(&path);

    cleanup(&path);
    assert_storage_error(result, StorageError::UnsupportedVersion);
}

#[test]
fn page_overflow_record_returns_error() {
    let path = temp_db_path("page_overflow_record_returns_error");

    let result = (|| {
        let mut store = PageStore::open(&path)?;
        store.append_record(&vec![0u8; PAGE_SIZE])
    })();

    cleanup(&path);
    assert_storage_error(result, StorageError::RecordTooLarge);
}

#[test]
fn corrupt_record_length_returns_error() {
    let path = temp_db_path("corrupt_record_length_returns_error");

    let result = (|| {
        {
            let mut store = PageStore::open(&path)?;
            store.append_record(b"alpha")?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("page file should open for corruption fixture");
        file.seek(SeekFrom::Start(FIRST_RECORD_LENGTH_OFFSET))
            .expect("seek should succeed");
        file.write_all(&(PAGE_SIZE as u32).to_le_bytes())
            .expect("corrupt length should be written");

        PageStore::open(&path)
    })();

    cleanup(&path);
    assert_storage_error(result, StorageError::CorruptRecordLength);
}

#[test]
fn qa_scaffold_req_6_store_data_in_disk_current_artifact_layout_evidence() {
    // REQ-6-store-data-in-a-disk-ad3ffc4e
    let path = temp_db_path("qa_req_6_layout_current_artifact_evidence");

    let result = (|| {
        let payload = b"current-artifact-layout";
        let mut store = PageStore::open(&path)?;
        store.append_record(payload)?;

        let bytes = fs::read(&path).expect("page file should be readable after append");
        assert_eq!(PAGE_SIZE * 2, bytes.len());
        assert_eq!(0, bytes.len() % PAGE_SIZE);
        assert_eq!(FILE_MAGIC, &bytes[0..8]);
        assert_eq!(1, read_u16(&bytes, FORMAT_VERSION_OFFSET));
        assert_eq!(PAGE_SIZE as u16, read_u16(&bytes, 10));
        assert_eq!(PAGE_SIZE as u32, read_u32(&bytes, 12));
        assert_eq!(2, read_u64(&bytes, PAGE_COUNT_OFFSET));

        assert_eq!(PAGE_MAGIC, &bytes[PAGE_SIZE..PAGE_SIZE + 4]);
        assert_eq!(1, read_u16(&bytes, PAGE_SIZE + 4));
        assert_eq!(
            DATA_PAGE_HEADER_SIZE as u16,
            read_u16(&bytes, PAGE_SIZE + 6)
        );
        assert_eq!(
            (DATA_PAGE_HEADER_SIZE + 4 + payload.len()) as u16,
            read_u16(&bytes, DATA_PAGE_USED_OFFSET)
        );
        assert_eq!(1, read_u16(&bytes, DATA_PAGE_RECORD_COUNT_OFFSET));
        assert_eq!(
            payload.len() as u32,
            read_u32(&bytes, FIRST_RECORD_LENGTH_OFFSET as usize)
        );
        let payload_start = FIRST_RECORD_LENGTH_OFFSET as usize + 4;
        assert_eq!(
            payload,
            &bytes[payload_start..payload_start + payload.len()]
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("current-artifact page layout evidence should pass");
}

#[test]
fn qa_scaffold_req_6_restart_durability_current_artifact_evidence() {
    // REQ-6-data-must-survive-process-restart-0471a233
    let path = temp_db_path("qa_req_6_restart_current_artifact_evidence");

    let result = (|| {
        let expected = vec![
            b"restart-alpha".to_vec(),
            b"restart-beta".to_vec(),
            vec![0x00, 0x10, 0xff],
        ];
        {
            let mut store = PageStore::open(&path)?;
            for payload in &expected {
                store.append_record(payload)?;
            }
        }
        let bytes_after_drop = fs::read(&path).expect("page file should exist after drop");

        let mut reopened = PageStore::open(&path)?;
        assert_eq!(expected, reopened.read_records()?);
        assert_eq!(
            bytes_after_drop,
            fs::read(&path).expect("reopen/read should not duplicate or rewrite records")
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("current-artifact restart durability evidence should pass");
}

#[test]
fn qa_scaffold_fail_6_reject_memory_only_dump_current_artifact_evidence() {
    // FAIL-6-reject-memory-only-dump-at-fd82a296
    let path = temp_db_path("qa_fail_6_memory_only_dump_current_artifact_evidence");

    let result = (|| {
        let payload = b"visible-before-drop";
        let mut store = PageStore::open(&path)?;
        store.append_record(payload)?;

        let bytes_while_live =
            fs::read(&path).expect("page file should be readable while PageStore is live");
        let payload_start = FIRST_RECORD_LENGTH_OFFSET as usize + 4;
        assert_eq!(PAGE_MAGIC, &bytes_while_live[PAGE_SIZE..PAGE_SIZE + 4]);
        assert_eq!(
            payload.len() as u32,
            read_u32(&bytes_while_live, FIRST_RECORD_LENGTH_OFFSET as usize)
        );
        assert_eq!(
            payload,
            &bytes_while_live[payload_start..payload_start + payload.len()]
        );
        assert_eq!(vec![payload.to_vec()], store.read_records()?);

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("current-artifact memory-only rejection evidence should pass");
}

#[test]
fn qa_scaffold_fail_6_reject_whole_file_rewrite_current_artifact_evidence() {
    // FAIL-6-reject-whole-database-file-rewrite-bebf73bb
    let path = temp_db_path("qa_fail_6_full_file_rewrite_current_artifact_evidence");

    let result = (|| {
        let first = b"stable-first-record";
        let second = b"bounded-active-page-append";
        let mut store = PageStore::open(&path)?;
        store.append_record(first)?;
        let before = fs::read(&path).expect("page file should be readable before second append");
        let before_used = read_u16(&before, DATA_PAGE_USED_OFFSET) as usize;
        assert_eq!(PAGE_SIZE * 2, before.len());
        assert_eq!(2, read_u64(&before, PAGE_COUNT_OFFSET));

        let audit = store.append_record_with_write_audit_for_test(second)?;
        let after = fs::read(&path).expect("page file should be readable after second append");
        let after_used = read_u16(&after, DATA_PAGE_USED_OFFSET) as usize;
        let expected_after_used = before_used + 4 + second.len();

        assert_eq!(
            vec![PageFileWrite {
                offset: PAGE_SIZE as u64,
                len: PAGE_SIZE,
            }],
            audit.page_file_writes,
            "same-page append should write exactly the active data page and not rewrite page 0 or the whole database file"
        );
        assert_eq!(before.len(), after.len());
        assert_eq!(2, read_u64(&after, PAGE_COUNT_OFFSET));
        assert_eq!(&before[0..PAGE_SIZE], &after[0..PAGE_SIZE]);
        assert_eq!(
            &before[PAGE_SIZE..PAGE_SIZE + 8],
            &after[PAGE_SIZE..PAGE_SIZE + 8]
        );
        assert_eq!(
            &before[PAGE_SIZE + 12..PAGE_SIZE + before_used],
            &after[PAGE_SIZE + 12..PAGE_SIZE + before_used]
        );
        assert_eq!(expected_after_used, after_used);
        assert_eq!(1, read_u16(&before, DATA_PAGE_RECORD_COUNT_OFFSET));
        assert_eq!(2, read_u16(&after, DATA_PAGE_RECORD_COUNT_OFFSET));
        assert_eq!(
            second.len() as u32,
            read_u32(&after, PAGE_SIZE + before_used)
        );
        assert_eq!(
            second,
            &after[PAGE_SIZE + before_used + 4..PAGE_SIZE + expected_after_used]
        );
        assert_eq!(
            &before[PAGE_SIZE + before_used..PAGE_SIZE + before_used + 4],
            &[0, 0, 0, 0],
            "pre-append active region should be empty before the bounded append"
        );
        assert_eq!(
            &before[PAGE_SIZE + expected_after_used..],
            &after[PAGE_SIZE + expected_after_used..],
            "suffix after the appended record should remain the same zero-filled capacity"
        );

        Ok::<(), StorageError>(())
    })();

    cleanup(&path);
    result.expect("current-artifact full-file rewrite rejection evidence should pass");
}
