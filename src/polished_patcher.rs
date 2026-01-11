use gb_save_core::{
    resolve_migration_plan, PatchLogEntry, PatchLogSink, SaveBinary, SaveError, SaveResult,
    VecPatchLogSink,
};

use crate::{
    get_save_version, polished_fix_patches, polished_migrations, supported_version_from_u16,
    symbols_for_version,
};

const PATCHER_LOG_SOURCE: &str = "polished.patcher";

/// Result of patching that also includes patch-internal logs.
#[derive(Debug)]
pub struct PatchSaveOutcome {
    /// Patched save bytes, if patching succeeded.
    pub bytes: Option<Vec<u8>>,
    /// Structured logs emitted while patching.
    pub logs: Vec<PatchLogEntry>,
    /// Human-readable error string, if patching failed.
    pub error: Option<String>,
}

/// Applies either a fix patch (`dev_type != 0`) or a version migration (`dev_type == 0`).
///
/// For fix patches, `target_version` must match the save's current version.
///
/// # Errors
/// Returns an error if the save cannot be parsed, the requested patch is unknown, or if patching
/// fails.
pub fn patch_save_bytes(bytes: Vec<u8>, target_version: u16, dev_type: u8) -> SaveResult<Vec<u8>> {
    let mut save = SaveBinary::new(bytes);
    let current_version = get_save_version(&save)?;

    if dev_type != 0 {
        if target_version != current_version {
            return Err(SaveError::InvalidSaveState {
                reason:
                    "fix patches do not migrate; target_version must match current save version"
                        .to_string(),
            });
        }

        let fix = polished_fix_patches()
            .iter()
            .find(|p| p.dev_type == dev_type)
            .ok_or(SaveError::UnknownFixPatch { dev_type })?;

        let symbols = symbols_for_version(supported_version_from_u16(current_version)?)?;
        fix.patch.apply(&mut save, &symbols)?;
        return Ok(save.into_bytes());
    }

    if current_version == target_version {
        return Ok(save.into_bytes());
    }

    let migrations = polished_migrations();
    let plan = resolve_migration_plan(&migrations, current_version, target_version)?;

    for patch in plan {
        let meta = patch.metadata();
        let from = meta
            .from_version
            .expect("migration patches must have from_version");
        let symbols = symbols_for_version(supported_version_from_u16(from)?)?;
        patch.apply(&mut save, &symbols)?;
    }

    Ok(save.into_bytes())
}

/// Like [`patch_save_bytes`], but captures structured logs and returns a non-error outcome type.
///
/// This is intended for UIs that want to show warnings/errors without relying on typed Rust
/// errors.
#[must_use]
pub fn patch_save_bytes_with_log(
    bytes: Vec<u8>,
    target_version: u16,
    dev_type: u8,
) -> PatchSaveOutcome {
    let mut log = VecPatchLogSink::new();

    let mut save = SaveBinary::new(bytes);
    let current_version = match get_save_version(&save) {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            log.error(PATCHER_LOG_SOURCE, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    };

    if dev_type != 0 {
        if target_version != current_version {
            let msg = "fix patches do not migrate; target_version must match current save version";
            log.error(PATCHER_LOG_SOURCE, msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg.to_string()),
            };
        }

        let fix = match polished_fix_patches()
            .iter()
            .find(|p| p.dev_type == dev_type)
        {
            Some(p) => p,
            None => {
                let msg = SaveError::UnknownFixPatch { dev_type }.to_string();
                log.error(PATCHER_LOG_SOURCE, &msg);
                return PatchSaveOutcome {
                    bytes: None,
                    logs: log.into_entries(),
                    error: Some(msg),
                };
            }
        };

        log.info(
            PATCHER_LOG_SOURCE,
            &format!(
                "applying fix patch dev_type={dev_type} id={}",
                fix.patch.metadata().id
            ),
        );

        let symbols =
            match supported_version_from_u16(current_version).and_then(symbols_for_version) {
                Ok(s) => s,
                Err(e) => {
                    let msg = e.to_string();
                    log.error(PATCHER_LOG_SOURCE, &msg);
                    return PatchSaveOutcome {
                        bytes: None,
                        logs: log.into_entries(),
                        error: Some(msg),
                    };
                }
            };

        if let Err(e) = fix.patch.apply_with_log(&mut save, &symbols, &mut log) {
            let msg = e.to_string();
            log.error(fix.patch.metadata().id, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }

        return PatchSaveOutcome {
            bytes: Some(save.into_bytes()),
            logs: log.into_entries(),
            error: None,
        };
    }

    if current_version == target_version {
        return PatchSaveOutcome {
            bytes: Some(save.into_bytes()),
            logs: log.into_entries(),
            error: None,
        };
    }

    let migrations = polished_migrations();
    let plan = match resolve_migration_plan(&migrations, current_version, target_version) {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            log.error(PATCHER_LOG_SOURCE, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    };

    let plan_ids = plan
        .iter()
        .map(|p| p.metadata().id)
        .collect::<Vec<_>>()
        .join(" -> ");
    log.info(
        PATCHER_LOG_SOURCE,
        &format!("migration plan {current_version} -> {target_version}: {plan_ids}"),
    );

    for patch in plan {
        let meta = patch.metadata();
        let from = meta
            .from_version
            .expect("migration patches must have from_version");

        let symbols = match supported_version_from_u16(from).and_then(symbols_for_version) {
            Ok(s) => s,
            Err(e) => {
                let msg = e.to_string();
                log.error(PATCHER_LOG_SOURCE, &msg);
                return PatchSaveOutcome {
                    bytes: None,
                    logs: log.into_entries(),
                    error: Some(msg),
                };
            }
        };

        if let Err(e) = patch.apply_with_log(&mut save, &symbols, &mut log) {
            let msg = e.to_string();
            log.error(meta.id, &msg);
            return PatchSaveOutcome {
                bytes: None,
                logs: log.into_entries(),
                error: Some(msg),
            };
        }
    }

    PatchSaveOutcome {
        bytes: Some(save.into_bytes()),
        logs: log.into_entries(),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{MIN_SAVE_SIZE, SAVE_VERSION_ABS_ADDRESS};

    #[test]
    fn unknown_fix_patch_returns_typed_error() {
        let mut bytes = vec![0u8; MIN_SAVE_SIZE];
        bytes[SAVE_VERSION_ABS_ADDRESS as usize] = 0x00;
        bytes[SAVE_VERSION_ABS_ADDRESS as usize + 1] = 0x09;

        let err = patch_save_bytes(bytes, 9, 1).unwrap_err();
        match err {
            SaveError::UnknownFixPatch { dev_type } => assert_eq!(dev_type, 1),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn patch_save_bytes_returns_bytes_when_no_migration_needed() {
        let bytes = bytes_with_version(9);
        let out = patch_save_bytes(bytes.clone(), 9, 0).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn patch_save_bytes_fix_patches_do_not_migrate() {
        let bytes = bytes_with_version(9);
        let err = patch_save_bytes(bytes, 10, 1).unwrap_err();
        match err {
            SaveError::InvalidSaveState { reason } => assert!(reason.contains("do not migrate")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    fn bytes_with_version(version: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; MIN_SAVE_SIZE];
        bytes[SAVE_VERSION_ABS_ADDRESS as usize] = (version >> 8) as u8;
        bytes[SAVE_VERSION_ABS_ADDRESS as usize + 1] = (version & 0xFF) as u8;
        bytes
    }

    #[test]
    fn with_log_returns_bytes_when_no_migration_needed() {
        let bytes = bytes_with_version(9);
        let out = patch_save_bytes_with_log(bytes.clone(), 9, 0);
        assert_eq!(out.error, None);
        assert_eq!(out.bytes, Some(bytes));
        assert!(out.logs.is_empty());
    }

    #[test]
    fn with_log_reports_error_on_too_small_input() {
        let out = patch_save_bytes_with_log(vec![0u8; 8], 9, 0);
        assert!(out.bytes.is_none());
        assert!(out.error.is_some());
        assert_eq!(out.logs.len(), 1);
        assert_eq!(out.logs[0].level, gb_save_core::PatchLogLevel::Error);
        assert_eq!(out.logs[0].source, PATCHER_LOG_SOURCE);
    }

    #[test]
    fn with_log_fix_patches_do_not_migrate() {
        let bytes = bytes_with_version(9);
        let out = patch_save_bytes_with_log(bytes, 10, 1);
        assert!(out.bytes.is_none());
        assert!(out.error.is_some());
        assert_eq!(out.logs.len(), 1);
        assert_eq!(out.logs[0].source, PATCHER_LOG_SOURCE);
        assert_eq!(
            out.logs[0].message,
            "fix patches do not migrate; target_version must match current save version"
        );
    }

    #[test]
    fn with_log_unknown_fix_patch_is_logged() {
        let dev_type = 0xFE;
        let bytes = bytes_with_version(9);
        let out = patch_save_bytes_with_log(bytes, 9, dev_type);
        assert!(out.bytes.is_none());
        assert_eq!(out.logs.len(), 1);
        assert_eq!(out.logs[0].source, PATCHER_LOG_SOURCE);
        assert_eq!(out.error, Some(out.logs[0].message.clone()));
        assert_eq!(
            out.logs[0].message,
            SaveError::UnknownFixPatch { dev_type }.to_string()
        );
    }

    #[test]
    fn with_log_migration_error_is_attributed_to_patch_id() {
        let bytes = bytes_with_version(7);
        let out = patch_save_bytes_with_log(bytes, 8, 0);
        assert!(out.bytes.is_none());
        assert!(out.error.is_some());
        assert!(out
            .logs
            .iter()
            .any(|e| e.source == "polished.migration.v7_to_v8"
                && e.level == gb_save_core::PatchLogLevel::Error));
    }
}
