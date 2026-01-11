use gb_save_core::{SaveBinary, SaveResult, Size, SymbolDatabase};

use super::{v7_to_v8_maps, Address, NEWBOX_SIZE, NUM_BOXES_V7, NUM_BOXES_V8};

fn write_default_box_name(save: &mut SaveBinary, start: Address, box_num: u8) -> SaveResult<()> {
    let digits = [0xE0 + (box_num / 10), 0xE0 + (box_num % 10)];
    let bytes = [0x7F, 0x7F, 0x81, 0xAE, 0xB7, 0x7F, digits[0], digits[1]];
    save.write_bytes(start, &bytes)
}

pub(super) fn migrate_newbox(
    src: &SaveBinary,
    dst: &mut SaveBinary,
    sym7: &SymbolDatabase,
    sym8: &SymbolDatabase,
    prefix: &str,
) -> SaveResult<()> {
    for n in 1..=NUM_BOXES_V8 {
        let name = format!("{prefix}{n}");
        let addr = sym8.sram_absolute_address(&name)?;
        dst.clear_len(addr, Size(NEWBOX_SIZE as u32))?;
    }

    for n in 1..=NUM_BOXES_V7 {
        let name = format!("{prefix}{n}");
        let src_addr = sym7.sram_absolute_address(&name)?;
        let dst_addr = sym8.sram_absolute_address(&name)?;
        dst.copy_from_other(src, src_addr, dst_addr, Size(NEWBOX_SIZE as u32))?;
    }

    for n in (NUM_BOXES_V7 + 1)..=NUM_BOXES_V8 {
        let name = format!("{prefix}{n}Name");
        let addr = sym8.sram_absolute_address(&name)?;
        write_default_box_name(dst, addr, n as u8)?;
    }

    for n in 1..=NUM_BOXES_V8 {
        let theme_name = format!("{prefix}{n}Theme");
        let addr = sym8.sram_absolute_address(&theme_name)?;
        let theme = dst.read_u8(addr)?;
        let mapped = v7_to_v8_maps::map_v7_theme_to_v8(theme);
        if mapped != v7_to_v8_maps::INVALID_U8 && theme != mapped {
            dst.write_u8(addr, mapped)?;
        }
    }

    Ok(())
}
