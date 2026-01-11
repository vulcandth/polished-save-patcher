#![forbid(unsafe_code)]

//! Pokémon Polished Crystal save patching.
//!
//! This crate provides:
//! - Save version detection
//! - Checksum helpers (main + backup)
//! - Migration patching between supported save versions
//! - A patch runner API that can collect structured logs
//!
//! Most callers should start with [`patch_save_bytes`] or [`patch_save_bytes_with_log`].
//!
//! # Example
//! ```no_run
//! # use gb_save_polished::{patch_save_bytes, SaveBinary};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bytes = vec![0u8; gb_save_polished::MIN_SAVE_SIZE];
//! let save = SaveBinary::new(bytes);
//! let version = gb_save_polished::get_save_version(&save)?;
//! let patched = patch_save_bytes(save.into_bytes(), version, 0)?;
//! drop(patched);
//! # Ok(())
//! # }
//! ```

mod checksum;
mod fixes;
mod migrations;
mod polished;
mod polished_checksums;
mod polished_patcher;
mod symbols;
mod validation;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use gb_save_core::{PatchLogLevel, SaveBinary};

pub use checksum::{
    calculate_newbox_checksum, calculate_save_checksum, extract_stored_newbox_checksum,
    write_newbox_checksum,
};

pub use polished::{get_save_version, MIN_SAVE_SIZE, SAVE_VERSION_ABS_ADDRESS};
pub use polished_checksums::{
    calculate_backup_save_checksum, calculate_main_save_checksum, read_backup_save_checksum,
    read_main_save_checksum, validate_backup_save_checksum, validate_main_save_checksum,
    write_backup_save_checksum, write_main_save_checksum, SAVE_BACKUP_CHECKSUM_ABS_ADDRESS,
    SAVE_CHECKSUM_ABS_ADDRESS,
};

pub use polished_patcher::{patch_save_bytes, patch_save_bytes_with_log, PatchSaveOutcome};
pub use symbols::{supported_version_from_u16, symbols_for_version, SupportedSaveVersion};
pub use validation::validate_primary_and_backup_checksums;
pub use validation::validate_primary_and_backup_checksums_with_log;

pub(crate) use fixes::polished_fix_patches;
pub(crate) use migrations::polished_migrations;
