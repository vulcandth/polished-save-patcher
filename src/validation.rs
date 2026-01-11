use gb_save_core::{PatchLogSink, SaveBinary, SaveError, SaveResult, SymbolDatabase};

use crate::{
    calculate_backup_save_checksum, calculate_main_save_checksum, read_backup_save_checksum,
    read_main_save_checksum,
};

/// Validates that both the primary and backup checksums match.
///
/// # Errors
/// Returns `SaveError::ChecksumMismatch` if either checksum does not match.
///
/// Also returns an error if checksum reading/calculation fails.
pub fn validate_primary_and_backup_checksums(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
) -> SaveResult<()> {
    let stored_main = read_main_save_checksum(save)?;
    let calc_main = calculate_main_save_checksum(save, symbols)?;
    if stored_main != calc_main {
        return Err(SaveError::ChecksumMismatch {
            which: "main",
            stored: stored_main,
            calculated: calc_main,
        });
    }

    let stored_backup = read_backup_save_checksum(save)?;
    let calc_backup = calculate_backup_save_checksum(save, symbols)?;
    if stored_backup != calc_backup {
        return Err(SaveError::ChecksumMismatch {
            which: "backup",
            stored: stored_backup,
            calculated: calc_backup,
        });
    }

    Ok(())
}

/// Validates that both the primary and backup checksums match and logs errors.
///
/// This mirrors [`validate_primary_and_backup_checksums`], but additionally records a log entry
/// before returning a mismatch error.
///
/// # Errors
/// Returns `SaveError::ChecksumMismatch` if either checksum does not match.
///
/// Also returns an error if checksum reading/calculation fails.
pub fn validate_primary_and_backup_checksums_with_log(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> SaveResult<()> {
    let stored_main = read_main_save_checksum(save)?;
    let calc_main = calculate_main_save_checksum(save, symbols)?;
    if stored_main != calc_main {
        log.error(
            log_source,
            &format!("Checksum mismatch! Expected: {calc_main:04x}, got: {stored_main:04x}",),
        );
        return Err(SaveError::ChecksumMismatch {
            which: "main",
            stored: stored_main,
            calculated: calc_main,
        });
    }

    let stored_backup = read_backup_save_checksum(save)?;
    let calc_backup = calculate_backup_save_checksum(save, symbols)?;
    if stored_backup != calc_backup {
        log.error(
            log_source,
            &format!(
                "Backup checksum mismatch! Expected: {calc_backup:04x}, got: {stored_backup:04x}",
            ),
        );
        return Err(SaveError::ChecksumMismatch {
            which: "backup",
            stored: stored_backup,
            calculated: calc_backup,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use gb_save_core::{PatchLogLevel, VecPatchLogSink};

    use crate::{
        symbols_for_version, write_backup_save_checksum, write_main_save_checksum,
        SupportedSaveVersion, MIN_SAVE_SIZE,
    };

    #[test]
    fn validate_checksums_ok_when_stored_matches_calculated() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let main = calculate_main_save_checksum(&save, &symbols).unwrap();
        let backup = calculate_backup_save_checksum(&save, &symbols).unwrap();
        write_main_save_checksum(&mut save, main).unwrap();
        write_backup_save_checksum(&mut save, backup).unwrap();

        validate_primary_and_backup_checksums(&save, &symbols).unwrap();

        let mut log = VecPatchLogSink::new();
        validate_primary_and_backup_checksums_with_log(&save, &symbols, &mut log, "test").unwrap();
        assert!(log.into_entries().is_empty());
    }

    #[test]
    fn validate_checksums_reports_main_mismatch_and_logs() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let correct_main = calculate_main_save_checksum(&save, &symbols).unwrap();
        let correct_backup = calculate_backup_save_checksum(&save, &symbols).unwrap();
        write_main_save_checksum(&mut save, correct_main ^ 0x0001).unwrap();
        write_backup_save_checksum(&mut save, correct_backup).unwrap();

        let err = validate_primary_and_backup_checksums(&save, &symbols).unwrap_err();
        match err {
            SaveError::ChecksumMismatch { which, .. } => assert_eq!(which, "main"),
            other => panic!("unexpected error: {other:?}"),
        }

        let mut log = VecPatchLogSink::new();
        let err = validate_primary_and_backup_checksums_with_log(&save, &symbols, &mut log, "test")
            .unwrap_err();
        match err {
            SaveError::ChecksumMismatch { which, .. } => assert_eq!(which, "main"),
            other => panic!("unexpected error: {other:?}"),
        }
        let entries = log.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, PatchLogLevel::Error);
        assert_eq!(entries[0].source, "test");
        assert!(entries[0].message.starts_with("Checksum mismatch!"));
    }

    #[test]
    fn validate_checksums_reports_backup_mismatch_and_logs() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let correct_main = calculate_main_save_checksum(&save, &symbols).unwrap();
        let correct_backup = calculate_backup_save_checksum(&save, &symbols).unwrap();
        write_main_save_checksum(&mut save, correct_main).unwrap();
        write_backup_save_checksum(&mut save, correct_backup ^ 0x0001).unwrap();

        let err = validate_primary_and_backup_checksums(&save, &symbols).unwrap_err();
        match err {
            SaveError::ChecksumMismatch { which, .. } => assert_eq!(which, "backup"),
            other => panic!("unexpected error: {other:?}"),
        }

        let mut log = VecPatchLogSink::new();
        let err = validate_primary_and_backup_checksums_with_log(&save, &symbols, &mut log, "test")
            .unwrap_err();
        match err {
            SaveError::ChecksumMismatch { which, .. } => assert_eq!(which, "backup"),
            other => panic!("unexpected error: {other:?}"),
        }
        let entries = log.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, PatchLogLevel::Error);
        assert_eq!(entries[0].source, "test");
        assert!(entries[0].message.starts_with("Backup checksum mismatch!"));
    }
}
