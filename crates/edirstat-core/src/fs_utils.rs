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

    /// Probe filesystem case sensitivity: on case-insensitive filesystems
    /// (default APFS, vfat, default NTFS mounts) a file is reachable under any
    /// casing, which changes which spelling `find_mft_file` can return.
    fn fs_is_case_insensitive(dir: &Path) -> Result<bool, crate::EdirstatError> {
        fs::write(dir.join("case_probe"), b"")?;
        let insensitive = dir.join("CASE_PROBE").exists();
        fs::remove_file(dir.join("case_probe"))?;
        Ok(insensitive)
    }

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

        // When both casings exist, `$MFT` wins — only meaningful on
        // case-sensitive filesystems, where both spellings can coexist
        // (on case-insensitive ones the write above already IS `$MFT`).
        if !fs_is_case_insensitive(&dir)? {
            fs::write(dir.join("$mft"), b"mft")?;
            assert_eq!(find_mft_file(&dir), Some(dir.join("$MFT")));
        }

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
        // On case-insensitive filesystems `$MFT` resolves to this same file,
        // so either casing is a correct discovery of the lowercase-only file.
        let found = find_mft_file(&dir);
        let name = found
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|n| n.to_str());
        assert!(name.is_some_and(|n| n.eq_ignore_ascii_case("$mft")));
        assert!(found.is_some_and(|p| p.is_file()));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
