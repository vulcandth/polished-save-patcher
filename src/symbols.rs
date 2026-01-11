use gb_save_core::{SaveError, SaveResult, SymbolDatabase};

/// Save versions currently supported by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedSaveVersion {
    /// Save version 7.
    V7,
    /// Save version 8.
    V8,
    /// Save version 9.
    V9,
    /// Save version 10.
    V10,
}

impl SupportedSaveVersion {
    /// Returns the numeric save version as stored in SRAM.
    #[must_use]
    pub fn as_u16(self) -> u16 {
        match self {
            Self::V7 => 7,
            Self::V8 => 8,
            Self::V9 => 9,
            Self::V10 => 10,
        }
    }
}

/// Gzip-compressed symbol database for save version 7.
const VERSION7_SYM_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/version7.sym.gz"
));
/// Gzip-compressed symbol database for save version 8.
const VERSION8_SYM_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/version8.sym.gz"
));
/// Gzip-compressed symbol database for save version 9.
const VERSION9_SYM_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/version9.sym.gz"
));
/// Gzip-compressed symbol database for save version 10.
const VERSION10_SYM_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/version10.sym.gz"
));

/// Loads the symbol database corresponding to a specific save version.
///
/// These symbols are embedded in the crate as gzip-compressed `.sym` resources.
///
/// # Errors
/// Returns an error if the embedded symbol database fails to decompress/parse.
pub fn symbols_for_version(version: SupportedSaveVersion) -> SaveResult<SymbolDatabase> {
    let gz = match version {
        SupportedSaveVersion::V7 => VERSION7_SYM_GZ,
        SupportedSaveVersion::V8 => VERSION8_SYM_GZ,
        SupportedSaveVersion::V9 => VERSION9_SYM_GZ,
        SupportedSaveVersion::V10 => VERSION10_SYM_GZ,
    };

    SymbolDatabase::from_gzip_bytes(gz)
}

/// Converts a numeric save version into a supported enum variant.
///
/// # Errors
/// Returns `SaveError::NotImplemented` if the version is not supported.
pub fn supported_version_from_u16(version: u16) -> SaveResult<SupportedSaveVersion> {
    match version {
        7 => Ok(SupportedSaveVersion::V7),
        8 => Ok(SupportedSaveVersion::V8),
        9 => Ok(SupportedSaveVersion::V9),
        10 => Ok(SupportedSaveVersion::V10),
        other => Err(SaveError::NotImplemented {
            feature: format!("unsupported save version {other}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::MIN_SAVE_SIZE;

    #[test]
    fn cross_version_symbol_loads_for_all_versions() {
        for v in [
            SupportedSaveVersion::V7,
            SupportedSaveVersion::V8,
            SupportedSaveVersion::V9,
            SupportedSaveVersion::V10,
        ] {
            let sym = symbols_for_version(v).unwrap();
            let addr = sym.sram_absolute_address("sGameData").unwrap();
            assert!(addr.0 < MIN_SAVE_SIZE as u32);
        }
    }

    #[test]
    fn cross_version_symbols_can_map_to_different_addresses() {
        let v8 = symbols_for_version(SupportedSaveVersion::V8).unwrap();
        let v9 = symbols_for_version(SupportedSaveVersion::V9).unwrap();

        let mut found_difference = false;
        for (name, sym8) in v8.iter() {
            let Ok(sym9) = v9.get_symbol(name) else {
                continue;
            };

            if sym8 != sym9 {
                found_difference = true;
                break;
            }
        }

        assert!(
            found_difference,
            "expected at least one symbol to differ between v8 and v9"
        );
    }

    #[test]
    fn unsupported_version_returns_typed_error() {
        let err = supported_version_from_u16(123).unwrap_err();
        match err {
            SaveError::NotImplemented { feature } => {
                assert!(feature.contains("unsupported save version"));
                assert!(feature.contains("123"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
