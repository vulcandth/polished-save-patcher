use gb_save_core::{
    calculate_additive_u16_checksum, Address, AddressRange, SaveBinary, SaveResult,
};

/// Calculates the additive checksum of a save byte range.
///
/// This matches the common "sum of bytes (wrapping u16)" scheme used by many saves.
///
/// # Errors
/// Returns an error if `range` is invalid or falls outside the save buffer.
pub fn calculate_save_checksum(save: &SaveBinary, range: AddressRange) -> SaveResult<u16> {
    calculate_additive_u16_checksum(save, range)
}

/// Calculates the "new box" checksum for a boxed data block starting at `start`.
///
/// The exact layout/weighting matches the original implementation this project ports.
///
/// # Errors
/// Returns an error if any referenced bytes are out of bounds.
pub fn calculate_newbox_checksum(save: &SaveBinary, start: Address) -> SaveResult<u16> {
    let mut checksum: u16 = 127;

    for i in 0..=0x1F_u32 {
        let value = save.read_u8(Address(start.0 + i))? as u16;
        checksum = checksum.wrapping_add(value.wrapping_mul((i + 1) as u16));
    }

    for i in 0x20_u32..=0x30_u32 {
        let value = (save.read_u8(Address(start.0 + i))? & 0x7F) as u16;
        checksum = checksum.wrapping_add(value.wrapping_mul((i + 2) as u16));
    }

    Ok(checksum)
}

/// Extracts the stored "new box" checksum bits from a boxed data block.
///
/// # Errors
/// Returns an error if any referenced bytes are out of bounds.
pub fn extract_stored_newbox_checksum(save: &SaveBinary, start: Address) -> SaveResult<u16> {
    let mut stored: u16 = 0;
    for i in 0..=0xF_u32 {
        let msb = (save.read_u8(Address(start.0 + 0x20 + i))? & 0x80) >> 7;
        stored |= (msb as u16) << (0xF - i);
    }
    Ok(stored)
}

/// Recomputes and writes the "new box" checksum bits back into the save.
///
/// # Errors
/// Returns an error if any referenced bytes are out of bounds.
pub fn write_newbox_checksum(save: &mut SaveBinary, start: Address) -> SaveResult<()> {
    let checksum = calculate_newbox_checksum(save, start)?;

    for i in 0..=0xF_u32 {
        let address = Address(start.0 + 0x20 + i);
        let mut byte = save.read_u8(address)?;
        byte &= 0x7F;
        byte |= (((checksum >> (0xF - i)) & 0x1) as u8) << 7;
        save.write_u8(address, byte)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_checksum_wraps_like_u16() {
        let save = SaveBinary::new(vec![0xFF; 4]);
        let checksum =
            calculate_save_checksum(&save, AddressRange::new(Address(0), Address(4))).unwrap();
        assert_eq!(checksum, 0x03FC);
    }

    #[test]
    fn newbox_round_trip_stored_bits() {
        let mut save = SaveBinary::new(vec![0; 0x200]);
        let start = Address(0x10);
        save.write_u8(Address(start.0 + 0x20), 0x80).unwrap();
        let before = extract_stored_newbox_checksum(&save, start).unwrap();
        assert_eq!(before & 0x8000, 0x8000);

        write_newbox_checksum(&mut save, start).unwrap();
        let stored = extract_stored_newbox_checksum(&save, start).unwrap();
        let expected = calculate_newbox_checksum(&save, start).unwrap();
        assert_eq!(stored, expected);
    }
}
