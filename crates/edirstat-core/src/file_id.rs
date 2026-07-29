use std::fs;
use std::path::Path;
use std::borrow::Cow;

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
pub fn get_file_id<'a>(meta: &fs::Metadata, _path: impl FnOnce() -> Cow<'a, Path>) -> (u64, u64) {
    #[cfg(not(feature = "stable"))]
    {
        use std::os::windows::fs::MetadataExt as _;

        (
            meta.volume_serial_number().unwrap_or(0) as u64,
            meta.file_index().unwrap_or(0),
        )
    }
    #[cfg(feature = "stable")]
    {
        ::file_id::get_file_id(_path()).unwrap_or((0, 0))
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
