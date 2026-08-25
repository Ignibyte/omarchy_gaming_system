use std::{
    fs::{self, File, Metadata},
    io::Read,
    path::Path,
};

use crate::error::{CartridgeError, Result};

/// Read one regular, non-symlink file through the same handle whose metadata
/// was checked, stopping before a caller-controlled byte limit can be crossed.
pub(crate) fn read_bounded_regular_file(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let path_metadata = fs::symlink_metadata(path)?;
    if !path_metadata.file_type().is_file() || path_metadata.file_type().is_symlink() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let mut file = File::open(path)?;
    let handle_metadata = file.metadata()?;
    if !handle_metadata.is_file() || !same_file(&path_metadata, &handle_metadata) {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    if handle_metadata.len() > maximum {
        return Err(CartridgeError::LimitExceeded);
    }
    let mut bytes = Vec::with_capacity(handle_metadata.len() as usize);
    file.by_ref()
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(CartridgeError::LimitExceeded);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_left: &Metadata, _right: &Metadata) -> bool {
    true
}
