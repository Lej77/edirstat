use std::borrow::Cow;
use std::fs;
use std::path::Path;

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(unix)]
#[must_use]
pub fn get_file_id<'a>(meta: &fs::Metadata, _path: impl FnOnce() -> Cow<'a, Path>) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt as _;

    (meta.dev(), meta.ino())
}

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(windows)]
#[must_use]
pub fn get_file_id<'a>(_meta: &fs::Metadata, _path: impl FnOnce() -> Cow<'a, Path>) -> (u64, u64) {
    #[cfg(not(feature = "stable"))]
    {
        use std::os::windows::fs::MetadataExt as _;

        (
            _meta.volume_serial_number().unwrap_or(0) as u64,
            _meta.file_index().unwrap_or(0),
        )
    }
    #[cfg(feature = "stable")]
    {
        use ::file_id::FileId;

        match ::file_id::get_file_id(_path()) {
            Err(_) => (0, 0),
            Ok(id) => match id {
                FileId::Inode {
                    device_id,
                    inode_number,
                } => (device_id, inode_number),
                FileId::LowRes {
                    volume_serial_number,
                    file_index,
                } => (u64::from(volume_serial_number), file_index),
                // ReFS file system includes 128-bit file identifiers, we just
                // truncate the id but we might mistakenly find duplicates
                FileId::HighRes {
                    volume_serial_number,
                    file_id,
                } => (volume_serial_number, file_id as u64),
            },
        }
    }
}

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(not(any(unix, windows)))]
#[must_use]
pub fn get_file_id<'a>(meta: &fs::Metadata, _path: impl FnOnce() -> Cow<'a, Path>) -> (u64, u64) {
    (0, 0)
}
