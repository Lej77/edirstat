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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_find_mft_file_prefers_uppercase() -> Result<(), crate::EdirstatError> {
        let dir = std::env::current_dir()?
            .join("target")
            .join("fs_utils_test_mft_upper");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        assert_eq!(find_mft_file(&dir), None);

        fs::write(dir.join("$MFT"), b"mft")?;
        assert_eq!(find_mft_file(&dir), Some(dir.join("$MFT")));

        // When both casings exist, `$MFT` wins.
        fs::write(dir.join("$mft"), b"mft")?;
        assert_eq!(find_mft_file(&dir), Some(dir.join("$MFT")));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_find_mft_file_lowercase_only_and_empty() -> Result<(), crate::EdirstatError> {
        let dir = std::env::current_dir()?
            .join("target")
            .join("fs_utils_test_mft_lower");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        assert_eq!(find_mft_file(&dir), None);

        fs::write(dir.join("$mft"), b"mft")?;
        assert_eq!(find_mft_file(&dir), Some(dir.join("$mft")));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
