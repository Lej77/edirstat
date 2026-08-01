use std::path::{Path, PathBuf};

/// Probes a directory path for the existence of an NTFS Master File Table (`$MFT` or `$mft`) file.
#[must_use]
pub fn find_mft_file(dir: &Path) -> Option<PathBuf> {
    let upper = dir.join("$MFT");
    if upper.is_file() {
        return Some(upper);
    }
    let lower = dir.join("$mft");
    if lower.is_file() {
        return Some(lower);
    }
    None
}
