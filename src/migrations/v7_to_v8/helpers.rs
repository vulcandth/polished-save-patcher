use gb_save_core::{Address, PatchLogSink, SaveBinary, SaveResult};

use super::v7_to_v8_maps;

pub(super) fn contains_u16(vec: &[u16], value: u16) -> bool {
    vec.contains(&value)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn convert_item_list(
    src: &SaveBinary,
    dst: &mut SaveBinary,
    num_items_addr7: Address,
    items_addr7: Address,
    num_items_addr8: Address,
    items_addr8: Address,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> SaveResult<()> {
    let num_items_v7 = src.read_u8(num_items_addr7)? as usize;
    dst.write_u8(num_items_addr8, num_items_v7 as u8)?;

    let mut num_items_v8: u8 = 0;
    let mut src_cursor = items_addr7;
    let mut dst_cursor = items_addr8;

    for _ in 0..=num_items_v7 {
        let item_id_v7 = src.read_u8(src_cursor)?;
        src_cursor = Address(src_cursor.0 + 1);
        if item_id_v7 == 0xFF {
            dst.write_u8(dst_cursor, 0xFF)?;
            break;
        }

        let mapped = v7_to_v8_maps::map_v7_item_to_v8(item_id_v7);
        if mapped != v7_to_v8_maps::INVALID_U8 {
            num_items_v8 = num_items_v8.wrapping_add(1);
            dst.write_u8(dst_cursor, mapped)?;
            dst_cursor = Address(dst_cursor.0 + 1);

            let qty = src.read_u8(src_cursor)?;
            src_cursor = Address(src_cursor.0 + 1);
            dst.write_u8(dst_cursor, qty)?;
            dst_cursor = Address(dst_cursor.0 + 1);
        } else {
            log.error(
                log_source,
                &format!("Item {item_id_v7:02x} not found in version 8 item list."),
            );
            src_cursor = Address(src_cursor.0 + 1);
            dst_cursor = Address(dst_cursor.0 + 1);
        }
    }

    dst.write_u8(num_items_addr8, num_items_v8)?;
    Ok(())
}

pub(super) fn map_and_write_map_group_number(
    src: &SaveBinary,
    dst: &mut SaveBinary,
    group_addr7: Address,
    group_addr8: Address,
    map_addr7: Address,
    map_addr8: Address,
) -> SaveResult<()> {
    let group = src.read_u8(group_addr7)?;
    let map = src.read_u8(map_addr7)?;
    let (g8, m8) = v7_to_v8_maps::map_v7_map_to_v8(group, map);
    dst.write_u8(group_addr8, g8)?;
    dst.write_u8(map_addr8, m8)?;
    Ok(())
}
