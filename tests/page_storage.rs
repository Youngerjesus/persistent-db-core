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
