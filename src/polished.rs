use gb_save_core::{Address, SaveBinary, SaveResult};

/// Size of a single SRAM bank in bytes.
pub const SRAM_BANK_SIZE: u32 = 0x2000;
/// The Game Boy SRAM base address.
pub const SRAM_START_ADDRESS: u32 = 0xA000;
/// Number of SRAM banks used by the save.
pub const NUM_SRAM_BANKS: u32 = 4;

/// SRAM bank containing the save version field.
pub const SAVE_VERSION_SRAM_BANK: u32 = 0x00;
/// In-bank address of the save version field.
pub const SAVE_VERSION_ADDRESS: u32 = 0xABE2;

/// Absolute offset (within the save buffer) of the save version field.
pub const SAVE_VERSION_ABS_ADDRESS: u32 =
    SAVE_VERSION_SRAM_BANK * SRAM_BANK_SIZE + SAVE_VERSION_ADDRESS - SRAM_START_ADDRESS;

/// Minimum byte length required for a complete save buffer.
pub const MIN_SAVE_SIZE: usize = (SRAM_BANK_SIZE * NUM_SRAM_BANKS) as usize;

/// Reads the current save version.
///
/// # Errors
/// Returns an error if the save is too small.
pub fn get_save_version(save: &SaveBinary) -> SaveResult<u16> {
    save.require_min_size(MIN_SAVE_SIZE)?;
    save.read_u16_be(Address(SAVE_VERSION_ABS_ADDRESS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_save_version_be_word() {
        let mut bytes = vec![0u8; MIN_SAVE_SIZE];
        bytes[SAVE_VERSION_ABS_ADDRESS as usize] = 0x00;
        bytes[SAVE_VERSION_ABS_ADDRESS as usize + 1] = 0x09;
        let save = SaveBinary::new(bytes);
        assert_eq!(get_save_version(&save).unwrap(), 9);
    }
}
