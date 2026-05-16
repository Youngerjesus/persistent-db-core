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

        Ok(Self { path })
    }

    pub fn append_record(&mut self, payload: &[u8]) -> Result<(), StorageError> {
        if payload.len() > max_record_payload_len() {
            return Err(StorageError::RecordTooLarge);
        }

        validate_file(&self.path)?;

        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
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

        page[used..used + RECORD_LENGTH_SIZE]
            .copy_from_slice(&(payload.len() as u32).to_le_bytes());
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
