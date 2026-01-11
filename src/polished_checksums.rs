use gb_save_core::{Address, AddressRange, SaveBinary, SaveResult, SymbolDatabase};

use crate::calculate_save_checksum;

/// Size of a single SRAM bank in bytes.
pub const SRAM_BANK_SIZE: u32 = 0x2000;
/// The Game Boy SRAM base address.
pub const SRAM_START_ADDRESS: u32 = 0xA000;

pub const SAVE_CHECKSUM_SRAM_BANK: u32 = 0x01;
pub const SAVE_CHECKSUM_ADDRESS: u32 = 0xAD0D;
/// Absolute offset (within the save buffer) of the main checksum word.
pub const SAVE_CHECKSUM_ABS_ADDRESS: u32 =
    SAVE_CHECKSUM_SRAM_BANK * SRAM_BANK_SIZE + SAVE_CHECKSUM_ADDRESS - SRAM_START_ADDRESS;

pub const SAVE_BACKUP_CHECKSUM_SRAM_BANK: u32 = 0x00;
pub const SAVE_BACKUP_CHECKSUM_ADDRESS: u32 = 0xBF0D;
/// Absolute offset (within the save buffer) of the backup checksum word.
pub const SAVE_BACKUP_CHECKSUM_ABS_ADDRESS: u32 = SAVE_BACKUP_CHECKSUM_SRAM_BANK * SRAM_BANK_SIZE
    + SAVE_BACKUP_CHECKSUM_ADDRESS
    - SRAM_START_ADDRESS;

/// Calculates the checksum for the primary save region.
///
/// # Errors
/// Returns an error if required symbols are missing or if any referenced bytes are out of bounds.
pub fn calculate_main_save_checksum(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
) -> SaveResult<u16> {
    let start = symbols.sram_absolute_address("sGameData")?;
    let end = symbols.sram_absolute_address("sGameDataEnd")?;
    calculate_save_checksum(save, AddressRange::new(start, end))
}

/// Calculates the checksum for the backup save region.
///
/// # Errors
/// Returns an error if required symbols are missing or if any referenced bytes are out of bounds.
pub fn calculate_backup_save_checksum(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
) -> SaveResult<u16> {
    let start = symbols.sram_absolute_address("sBackupGameData")?;
    let end = symbols.sram_absolute_address("sBackupGameDataEnd")?;
    calculate_save_checksum(save, AddressRange::new(start, end))
}

/// Reads the stored primary checksum.
pub fn read_main_save_checksum(save: &SaveBinary) -> SaveResult<u16> {
    save.read_u16_le(Address(SAVE_CHECKSUM_ABS_ADDRESS))
}

/// Writes the stored primary checksum.
pub fn write_main_save_checksum(save: &mut SaveBinary, checksum: u16) -> SaveResult<()> {
    save.write_u16_le(Address(SAVE_CHECKSUM_ABS_ADDRESS), checksum)
}

/// Reads the stored backup checksum.
pub fn read_backup_save_checksum(save: &SaveBinary) -> SaveResult<u16> {
    save.read_u16_le(Address(SAVE_BACKUP_CHECKSUM_ABS_ADDRESS))
}

/// Writes the stored backup checksum.
pub fn write_backup_save_checksum(save: &mut SaveBinary, checksum: u16) -> SaveResult<()> {
    save.write_u16_le(Address(SAVE_BACKUP_CHECKSUM_ABS_ADDRESS), checksum)
}

/// Validates the primary checksum and returns whether it matches.
pub fn validate_main_save_checksum(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
) -> SaveResult<bool> {
    let stored = read_main_save_checksum(save)?;
    let calculated = calculate_main_save_checksum(save, symbols)?;
    Ok(stored == calculated)
}

/// Validates the backup checksum and returns whether it matches.
pub fn validate_backup_save_checksum(
    save: &SaveBinary,
    symbols: &SymbolDatabase,
) -> SaveResult<bool> {
    let stored = read_backup_save_checksum(save)?;
    let calculated = calculate_backup_save_checksum(save, symbols)?;
    Ok(stored == calculated)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{symbols_for_version, SupportedSaveVersion, MIN_SAVE_SIZE};

    #[test]
    fn checksum_is_little_endian_on_disk() {
        let mut save = SaveBinary::new(vec![0; 0x4000]);
        write_main_save_checksum(&mut save, 0x1234).unwrap();
        assert_eq!(save.as_bytes()[SAVE_CHECKSUM_ABS_ADDRESS as usize], 0x34);
        assert_eq!(
            save.as_bytes()[SAVE_CHECKSUM_ABS_ADDRESS as usize + 1],
            0x12
        );
        assert_eq!(read_main_save_checksum(&save).unwrap(), 0x1234);
    }

    #[test]
    fn backup_checksum_is_little_endian_on_disk() {
        let mut save = SaveBinary::new(vec![0; MIN_SAVE_SIZE]);
        write_backup_save_checksum(&mut save, 0xBEEF).unwrap();
        assert_eq!(
            save.as_bytes()[SAVE_BACKUP_CHECKSUM_ABS_ADDRESS as usize],
            0xEF
        );
        assert_eq!(
            save.as_bytes()[SAVE_BACKUP_CHECKSUM_ABS_ADDRESS as usize + 1],
            0xBE
        );
        assert_eq!(read_backup_save_checksum(&save).unwrap(), 0xBEEF);
    }

    #[test]
    fn validate_main_checksum_true_then_false_after_mutation() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let calculated = calculate_main_save_checksum(&save, &symbols).unwrap();
        write_main_save_checksum(&mut save, calculated).unwrap();
        assert!(validate_main_save_checksum(&save, &symbols).unwrap());

        let start = symbols.sram_absolute_address("sGameData").unwrap();
        let prev = save.read_u8(start).unwrap();
        save.write_u8(start, prev ^ 0xFF).unwrap();
        assert!(!validate_main_save_checksum(&save, &symbols).unwrap());
    }

    #[test]
    fn validate_backup_checksum_true_then_false_after_mutation() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let calculated = calculate_backup_save_checksum(&save, &symbols).unwrap();
        write_backup_save_checksum(&mut save, calculated).unwrap();
        assert!(validate_backup_save_checksum(&save, &symbols).unwrap());

        let start = symbols.sram_absolute_address("sBackupGameData").unwrap();
        let prev = save.read_u8(start).unwrap();
        save.write_u8(start, prev ^ 0xFF).unwrap();
        assert!(!validate_backup_save_checksum(&save, &symbols).unwrap());
    }
}
