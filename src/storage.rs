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

        Ok(Self { path })
    }

    pub fn append_record(&mut self, payload: &[u8]) -> Result<(), StorageError> {
        if payload.len() > max_record_payload_len() {
            return Err(StorageError::RecordTooLarge);
        }

        let record_count_before = total_record_count(&self.path)?;
        append_wal_frame(&wal_path(&self.path), record_count_before, payload)?;
        append_record_to_file(&self.path, payload)?;
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

pub fn read_records_for_check(
    path: impl AsRef<Path>,
) -> Result<CheckStorageSnapshot, StorageError> {
    let path = path.as_ref();
    validate_file(path)?;

    let mut file = OpenOptions::new().read(true).open(path)?;
    let page_count = read_page_count(&mut file)?;
    let mut records = Vec::new();

    for page_index in 1..page_count {
        let page = read_page(&mut file, page_index)?;
        read_data_page_records(&page, &mut records)?;
    }

    let record_count = u64::try_from(records.len()).map_err(|_| StorageError::Io)?;
    Ok(CheckStorageSnapshot {
        records,
        record_count,
    })
}

pub fn validate_wal_for_check(
    path: impl AsRef<Path>,
    durable_record_count: u64,
) -> Result<(), StorageError> {
    read_committed_wal_records_for_check(path, durable_record_count).map(|_| ())
}

pub fn read_committed_wal_records_for_check(
    path: impl AsRef<Path>,
    durable_record_count: u64,
) -> Result<Vec<Vec<u8>>, StorageError> {
    let wal_path = wal_path(path.as_ref());
    if !wal_path.exists() {
        return Ok(Vec::new());
    }

    let bytes = std::fs::read(&wal_path)?;
    let mut offset = 0usize;
    let mut virtual_record_count = durable_record_count;
    let mut committed_records = Vec::new();
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < WAL_HEADER_LEN {
            return Ok(committed_records);
        }

        let header = &bytes[offset..offset + WAL_HEADER_LEN];
        if &header[0..8] != WAL_MAGIC {
            return Err(StorageError::InvalidMagic);
        }

        let version = u16::from_le_bytes([header[8], header[9]]);
        if version != WAL_VERSION {
            return Err(StorageError::UnsupportedVersion);
        }

        let record_count_before = u64::from_le_bytes([
            header[18], header[19], header[20], header[21], header[22], header[23], header[24],
            header[25],
        ]);
        let state = header[26];
        let payload_kind = header[27];
        let payload_len =
            u32::from_le_bytes([header[28], header[29], header[30], header[31]]) as usize;
        let frame_len = WAL_HEADER_LEN
            .checked_add(payload_len)
            .ok_or(StorageError::CorruptRecordLength)?;
        if remaining < frame_len {
            return Ok(committed_records);
        }

        let frame = &bytes[offset..offset + frame_len];
        let expected_checksum =
            u32::from_le_bytes([header[32], header[33], header[34], header[35]]);
        if expected_checksum != wal_checksum(frame) {
            return Err(StorageError::CorruptRecordLength);
        }

        if state == WAL_STATE_COMMITTED && payload_kind == WAL_PAYLOAD_KIND_PAGE_APPEND {
            match virtual_record_count.cmp(&record_count_before) {
                std::cmp::Ordering::Equal => {
                    let payload = &frame[WAL_HEADER_LEN..];
                    validate_wal_page_append_payload(payload)?;
                    committed_records.push(payload.to_vec());
                    virtual_record_count = virtual_record_count
                        .checked_add(1)
                        .ok_or(StorageError::Io)?;
                }
                std::cmp::Ordering::Less => return Err(StorageError::CorruptRecordLength),
                std::cmp::Ordering::Greater => {}
            }
        } else if state != WAL_STATE_ROLLED_BACK {
            return Err(StorageError::InvalidMagic);
        }

        offset += frame_len;
    }

    Ok(committed_records)
}

fn validate_wal_page_append_payload(payload: &[u8]) -> Result<(), StorageError> {
    if payload.len() > max_record_payload_len() {
        return Err(StorageError::RecordTooLarge);
    }
    Ok(())
}

fn append_record_to_file(path: &Path, payload: &[u8]) -> Result<(), StorageError> {
    validate_file(path)?;

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut page_count = read_page_count(&mut file)?;

    if page_count == 1 {
        append_empty_data_page(&mut file)?;
        page_count = 2;
        write_page_count(&mut file, page_count)?;
    }

    let record_size = RECORD_LENGTH_SIZE + payload.len();
    let mut page_index = page_count - 1;
    let mut page = read_page(&mut file, page_index)?;
    let mut used = data_page_used(&page)? as usize;

    if used + record_size > PAGE_SIZE {
        append_empty_data_page(&mut file)?;
        page_count += 1;
        page_index = page_count - 1;
        write_page_count(&mut file, page_count)?;
        page = empty_data_page();
        used = DATA_PAGE_HEADER_SIZE;
    }

    if used + record_size > PAGE_SIZE {
        return Err(StorageError::RecordTooLarge);
    }

    page[used..used + RECORD_LENGTH_SIZE].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    let payload_start = used + RECORD_LENGTH_SIZE;
    page[payload_start..payload_start + payload.len()].copy_from_slice(payload);

    let new_used = used + record_size;
    let new_count = data_page_record_count(&page)?
        .checked_add(1)
        .ok_or(StorageError::Io)?;
    page[DATA_PAGE_USED_OFFSET..DATA_PAGE_USED_OFFSET + 2]
        .copy_from_slice(&(new_used as u16).to_le_bytes());
    page[DATA_PAGE_RECORD_COUNT_OFFSET..DATA_PAGE_RECORD_COUNT_OFFSET + 2]
        .copy_from_slice(&new_count.to_le_bytes());

    write_page(&mut file, page_index, &page)?;
    file.flush()?;
    Ok(())
}

fn validate_file(path: &Path) -> Result<(), StorageError> {
    let metadata = std::fs::metadata(path)?;
    let len = metadata.len();
    if len < PAGE_SIZE as u64 {
        return Err(StorageError::TruncatedFile);
    }
    if len % PAGE_SIZE as u64 != 0 {
        return Err(StorageError::TruncatedPage);
    }

    let mut file = OpenOptions::new().read(true).open(path)?;
    let header = read_page(&mut file, 0)?;
    validate_file_header(&header)?;

    let page_count = page_count_from_header(&header)?;
    if page_count == 0 {
        return Err(StorageError::TruncatedFile);
    }

    let actual_page_count = len / PAGE_SIZE as u64;
    if actual_page_count < page_count {
        return Err(StorageError::TruncatedPage);
    }
    if actual_page_count > page_count {
        return Err(StorageError::TruncatedPage);
    }

    for page_index in 1..page_count {
        let page = read_page(&mut file, page_index)?;
        validate_data_page(&page)?;
    }

    Ok(())
}

fn replay_wal(path: &Path) -> Result<(), StorageError> {
    let wal_path = wal_path(path);
    if !wal_path.exists() {
        return Ok(());
    }

    let bytes = std::fs::read(&wal_path)?;
    let mut offset = 0usize;
    let mut truncate_to = None;
    let mut replay_applies = 0u64;
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        if remaining < WAL_HEADER_LEN {
            truncate_to = Some(offset);
            break;
        }

        let header = &bytes[offset..offset + WAL_HEADER_LEN];
        if &header[0..8] != WAL_MAGIC {
            return Err(StorageError::InvalidMagic);
        }

        let version = u16::from_le_bytes([header[8], header[9]]);
        if version != WAL_VERSION {
            return Err(StorageError::UnsupportedVersion);
        }

        let record_count_before = u64::from_le_bytes([
            header[18], header[19], header[20], header[21], header[22], header[23], header[24],
            header[25],
        ]);
        let state = header[26];
        let payload_kind = header[27];
        let payload_len =
            u32::from_le_bytes([header[28], header[29], header[30], header[31]]) as usize;
        let frame_len = WAL_HEADER_LEN
            .checked_add(payload_len)
            .ok_or(StorageError::CorruptRecordLength)?;
        if remaining < frame_len {
            truncate_to = Some(offset);
            break;
        }

        let frame = &bytes[offset..offset + frame_len];
        let expected_checksum =
            u32::from_le_bytes([header[32], header[33], header[34], header[35]]);
        if expected_checksum != wal_checksum(frame) {
            return Err(StorageError::CorruptRecordLength);
        }

        if state == WAL_STATE_COMMITTED && payload_kind == WAL_PAYLOAD_KIND_PAGE_APPEND {
            let current_record_count = total_record_count(path)?;
            match current_record_count.cmp(&record_count_before) {
                std::cmp::Ordering::Equal => {
                    append_record_to_file(path, &frame[WAL_HEADER_LEN..])?;
                    replay_applies = replay_applies.checked_add(1).ok_or(StorageError::Io)?;
                    maybe_interrupt_after_wal_replay_apply(replay_applies);
                }
                std::cmp::Ordering::Less => return Err(StorageError::CorruptRecordLength),
                std::cmp::Ordering::Greater => {}
            }
        } else if state != WAL_STATE_ROLLED_BACK {
            return Err(StorageError::InvalidMagic);
        }

        offset += frame_len;
    }

    if let Some(len) = truncate_to {
        OpenOptions::new()
            .write(true)
            .open(&wal_path)?
            .set_len(len as u64)?;
    }

    Ok(())
}

fn maybe_interrupt_after_wal_replay_apply(replay_applies: u64) {
    let Ok(limit) = std::env::var("PDB_CRASH_AFTER_WAL_REPLAY_APPLIES") else {
        return;
    };
    let Ok(limit) = limit.parse::<u64>() else {
        return;
    };
    if limit == replay_applies {
        std::process::exit(101);
    }
}

fn append_wal_frame(
    wal_path: &Path,
    record_count_before: u64,
    payload: &[u8],
) -> Result<(), StorageError> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| StorageError::RecordTooLarge)?;
    let frame_id = next_wal_frame_id(wal_path)?;
    let mut frame = Vec::with_capacity(WAL_HEADER_LEN + payload.len());
    frame.extend_from_slice(WAL_MAGIC);
    frame.extend_from_slice(&WAL_VERSION.to_le_bytes());
    frame.extend_from_slice(&frame_id.to_le_bytes());
    frame.extend_from_slice(&record_count_before.to_le_bytes());
    frame.push(WAL_STATE_COMMITTED);
    frame.push(WAL_PAYLOAD_KIND_PAGE_APPEND);
    frame.extend_from_slice(&payload_len.to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(payload);

    let checksum = wal_checksum(&frame);
    frame[WAL_CHECKSUM_OFFSET..WAL_CHECKSUM_END].copy_from_slice(&checksum.to_le_bytes());

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(wal_path)?;
    file.write_all(&frame)?;
    file.flush()?;
    Ok(())
}

fn next_wal_frame_id(wal_path: &Path) -> Result<u64, StorageError> {
    if !wal_path.exists() {
        return Ok(1);
    }

    let bytes = std::fs::read(wal_path)?;
    let mut offset = 0usize;
    let mut frames = 0u64;
    while offset + WAL_HEADER_LEN <= bytes.len() {
        let payload_len = u32::from_le_bytes([
            bytes[offset + 28],
            bytes[offset + 29],
            bytes[offset + 30],
            bytes[offset + 31],
        ]) as usize;
        let Some(frame_len) = WAL_HEADER_LEN.checked_add(payload_len) else {
            break;
        };
        if offset + frame_len > bytes.len() {
            break;
        }
        frames = frames.checked_add(1).ok_or(StorageError::Io)?;
        offset += frame_len;
    }
    Ok(frames + 1)
}

fn wal_checksum(frame: &[u8]) -> u32 {
    frame
        .iter()
        .enumerate()
        .filter(|(index, _)| !(WAL_CHECKSUM_OFFSET..WAL_CHECKSUM_END).contains(index))
        .fold(0u32, |sum, (_, byte)| sum.wrapping_add(*byte as u32))
}

fn total_record_count(path: &Path) -> Result<u64, StorageError> {
    validate_file(path)?;
    let mut file = OpenOptions::new().read(true).open(path)?;
    let page_count = read_page_count(&mut file)?;
    let mut total = 0u64;
    for page_index in 1..page_count {
        let page = read_page(&mut file, page_index)?;
        total = total
            .checked_add(data_page_record_count(&page)? as u64)
            .ok_or(StorageError::Io)?;
    }
    Ok(total)
}

fn wal_path(path: &Path) -> PathBuf {
    let mut sidecar = path.as_os_str().to_os_string();
    sidecar.push(".wal");
    PathBuf::from(sidecar)
}

fn validate_file_header(page: &[u8]) -> Result<(), StorageError> {
    if &page[0..8] != FILE_MAGIC {
        return Err(StorageError::InvalidMagic);
    }

    let version = u16::from_le_bytes([page[8], page[9]]);
    if version != FORMAT_VERSION {
        return Err(StorageError::UnsupportedVersion);
    }

    let page_size_u16 = u16::from_le_bytes([page[10], page[11]]);
    let page_size_u32 = u32::from_le_bytes([page[12], page[13], page[14], page[15]]);
    if page_size_u16 as usize != PAGE_SIZE || page_size_u32 as usize != PAGE_SIZE {
        return Err(StorageError::InvalidMagic);
    }

    Ok(())
}

fn validate_data_page(page: &[u8]) -> Result<(), StorageError> {
    if &page[0..4] != DATA_PAGE_MAGIC {
        return Err(StorageError::InvalidMagic);
    }

    let version = u16::from_le_bytes([page[4], page[5]]);
    if version != FORMAT_VERSION {
        return Err(StorageError::UnsupportedVersion);
    }

    let header_size = u16::from_le_bytes([page[6], page[7]]) as usize;
    if header_size != DATA_PAGE_HEADER_SIZE {
        return Err(StorageError::InvalidMagic);
    }

    let used = data_page_used(page)? as usize;
    if !(DATA_PAGE_HEADER_SIZE..=PAGE_SIZE).contains(&used) {
        return Err(StorageError::CorruptRecordLength);
    }

    let record_count = data_page_record_count(page)? as usize;
    let mut offset = DATA_PAGE_HEADER_SIZE;
    for _ in 0..record_count {
        if offset + RECORD_LENGTH_SIZE > used {
            return Err(StorageError::CorruptRecordLength);
        }
        let len = u32::from_le_bytes([
            page[offset],
            page[offset + 1],
            page[offset + 2],
            page[offset + 3],
        ]) as usize;
        offset += RECORD_LENGTH_SIZE;
        if offset + len > used {
            return Err(StorageError::CorruptRecordLength);
        }
        offset += len;
    }

    if offset != used {
        return Err(StorageError::CorruptRecordLength);
    }

    Ok(())
}

fn read_data_page_records(page: &[u8], records: &mut Vec<Vec<u8>>) -> Result<(), StorageError> {
    validate_data_page(page)?;

    let record_count = data_page_record_count(page)? as usize;
    let mut offset = DATA_PAGE_HEADER_SIZE;
    for _ in 0..record_count {
        let len = u32::from_le_bytes([
            page[offset],
            page[offset + 1],
            page[offset + 2],
            page[offset + 3],
        ]) as usize;
        offset += RECORD_LENGTH_SIZE;
        records.push(page[offset..offset + len].to_vec());
        offset += len;
    }

    Ok(())
}

fn write_file_header(file: &mut File, page_count: u64) -> Result<(), StorageError> {
    let mut page = vec![0u8; PAGE_SIZE];
    page[0..8].copy_from_slice(FILE_MAGIC);
    page[8..10].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    page[10..12].copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
    page[12..16].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    page[FILE_HEADER_PAGE_COUNT_OFFSET..FILE_HEADER_PAGE_COUNT_OFFSET + 8]
        .copy_from_slice(&page_count.to_le_bytes());
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&page)?;
    Ok(())
}

fn read_page_count(file: &mut File) -> Result<u64, StorageError> {
    let page = read_page(file, 0)?;
    validate_file_header(&page)?;
    page_count_from_header(&page)
}

fn page_count_from_header(page: &[u8]) -> Result<u64, StorageError> {
    Ok(u64::from_le_bytes([
        page[FILE_HEADER_PAGE_COUNT_OFFSET],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 1],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 2],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 3],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 4],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 5],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 6],
        page[FILE_HEADER_PAGE_COUNT_OFFSET + 7],
    ]))
}

fn write_page_count(file: &mut File, page_count: u64) -> Result<(), StorageError> {
    file.seek(SeekFrom::Start(FILE_HEADER_PAGE_COUNT_OFFSET as u64))?;
    file.write_all(&page_count.to_le_bytes())?;
    Ok(())
}

fn append_empty_data_page(file: &mut File) -> Result<(), StorageError> {
    file.seek(SeekFrom::End(0))?;
    file.write_all(&empty_data_page())?;
    Ok(())
}

fn empty_data_page() -> Vec<u8> {
    let mut page = vec![0u8; PAGE_SIZE];
    page[0..4].copy_from_slice(DATA_PAGE_MAGIC);
    page[4..6].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    page[6..8].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[8..10].copy_from_slice(&(DATA_PAGE_HEADER_SIZE as u16).to_le_bytes());
    page[10..12].copy_from_slice(&0u16.to_le_bytes());
    page
}

fn read_page(file: &mut File, page_index: u64) -> Result<Vec<u8>, StorageError> {
    let mut page = vec![0u8; PAGE_SIZE];
    file.seek(SeekFrom::Start(page_index * PAGE_SIZE as u64))?;
    file.read_exact(&mut page).map_err(|_| {
        if page_index == 0 {
            StorageError::TruncatedFile
        } else {
            StorageError::TruncatedPage
        }
    })?;
    Ok(page)
}

fn write_page(file: &mut File, page_index: u64, page: &[u8]) -> Result<(), StorageError> {
    file.seek(SeekFrom::Start(page_index * PAGE_SIZE as u64))?;
    file.write_all(page)?;
    Ok(())
}

fn data_page_used(page: &[u8]) -> Result<u16, StorageError> {
    Ok(u16::from_le_bytes([
        page[DATA_PAGE_USED_OFFSET],
        page[DATA_PAGE_USED_OFFSET + 1],
    ]))
}

fn data_page_record_count(page: &[u8]) -> Result<u16, StorageError> {
    Ok(u16::from_le_bytes([
        page[DATA_PAGE_RECORD_COUNT_OFFSET],
        page[DATA_PAGE_RECORD_COUNT_OFFSET + 1],
    ]))
}

fn max_record_payload_len() -> usize {
    PAGE_SIZE - DATA_PAGE_HEADER_SIZE - RECORD_LENGTH_SIZE
}
