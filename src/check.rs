use crate::sql;
use crate::storage::{self, StorageError};
use std::path::{Path, PathBuf};

pub const SUCCESS_OUTPUT: &str = "ok: db check passed\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    OpenRead { path: PathBuf },
    Invariant { label: &'static str },
}

pub fn check_database(path: impl AsRef<Path>) -> Result<(), CheckError> {
    let path = path.as_ref();
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) | Err(_) => {
            return Err(CheckError::OpenRead {
                path: path.to_path_buf(),
            });
        }
    }

    let snapshot = storage::read_records_for_check(path).map_err(|error| match error {
        StorageError::Io => CheckError::OpenRead {
            path: path.to_path_buf(),
        },
        _ => CheckError::Invariant {
            label: "storage record readability",
        },
    })?;

    let wal_records = storage::read_committed_wal_records_for_check(path, snapshot.record_count)
        .map_err(|error| match error {
            StorageError::Io => CheckError::OpenRead {
                path: path.to_path_buf(),
            },
            _ => CheckError::Invariant {
                label: "wal replay consistency",
            },
        })?;
    let mut records = snapshot.records;
    records.extend(wal_records);

    sql::validate_records_for_check(records).map_err(|label| CheckError::Invariant { label })?;

    storage::validate_wal_for_check(path, snapshot.record_count).map_err(|error| match error {
        StorageError::Io => CheckError::OpenRead {
            path: path.to_path_buf(),
        },
        _ => CheckError::Invariant {
            label: "wal replay consistency",
        },
    })?;

    Ok(())
}
