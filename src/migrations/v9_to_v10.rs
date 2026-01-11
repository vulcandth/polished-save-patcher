use gb_save_core::{
    bits_to_bytes, map_bitset, Address, AddressRange, Patch, PatchKind, PatchLogSink,
    PatchMetadata, SaveBinary, SaveError, SaveResult, Size, SymbolDatabase,
};

use crate::{
    calculate_backup_save_checksum, calculate_main_save_checksum, get_save_version,
    symbols_for_version, validate_primary_and_backup_checksums_with_log,
    write_backup_save_checksum, write_main_save_checksum, SupportedSaveVersion,
    SAVE_VERSION_ABS_ADDRESS,
};

use super::{addrs::PolishedAddrs, common, scaffold, v9_to_v10_maps};

const TEXT_DELAY_MASK: u8 = 0x03;
const NO_EXP_OPT: u8 = 2;
const RESET_INIT_OPTS: u8 = 7;

const RALPH_NAME: &[u8] = &[0x91, 0xA0, 0xAB, 0xAF, 0xA7, 0x53];

const MAIL_MSG_LENGTH: usize = 0x20;
const PLAYER_NAME_LENGTH: usize = 8;
const MAIL_STRUCT_LEN: usize = MAIL_MSG_LENGTH + 1 + PLAYER_NAME_LENGTH + 2 + 2 + 1 + 1;

const PARTY_LENGTH: usize = 6;
const MAILBOX_CAPACITY: usize = 10;

const PLAINBADGE: usize = 2;

const GOLDENROD_POKECOM_CENTER_1F: (u8, u8) = (11, 24);
const PLAYERS_HOUSE_1F: (u8, u8) = (24, 6);

#[derive(Debug)]
pub struct MigrationV9ToV10;

pub static MIGRATION_V9_TO_V10: MigrationV9ToV10 = MigrationV9ToV10;

impl MigrationV9ToV10 {}

fn expand_v9_ngram(byte: u8) -> Option<&'static [u8]> {
    match byte {
        0x09 => Some(&[0xA4, 0x7F]),
        0x0A => Some(&[0x7F, 0xB3]),
        0x0B => Some(&[0xAE, 0xB4]),
        0x0C => Some(&[0xA8, 0xAD]),
        0x0D => Some(&[0xB3, 0xA7]),
        0x0E => Some(&[0xA7, 0xA4]),
        0x0F => Some(&[0xB3, 0x7F]),
        0x10 => Some(&[0xA4, 0xB1]),
        0x11 => Some(&[0xAE, 0xAD]),
        0x12 => Some(&[0xB1, 0xA4]),
        0x13 => Some(&[0xB2, 0x7F]),
        0x14 => Some(&[0xA0, 0xB3]),
        0x15 => Some(&[0xA0, 0xAD]),
        0x16 => Some(&[0xB3, 0xAE]),
        0x17 => Some(&[0xA7, 0xA0]),
        0x18 => Some(&[0xAD, 0xA6]),
        0x19 => Some(&[0xA8, 0xB3]),
        0x1A => Some(&[0xA8, 0xB2]),
        0x1B => Some(&[0xA4, 0xA0]),
        0x1C => Some(&[0xB5, 0xA4]),
        0x1D => Some(&[0xA0, 0xB1]),
        0x1E => Some(&[0xB2, 0xB3]),
        0x1F => Some(&[0xAB, 0xA4]),
        0x20 => Some(&[0xAE, 0xB1]),
        0x21 => Some(&[0xB3, 0xA4]),
        0x22 => Some(&[0xA0, 0xB2]),
        0x23 => Some(&[0xB8, 0xAE]),
        0x24 => Some(&[0xB8, 0x7F]),
        0x25 => Some(&[0xB1, 0x7F]),
        0x26 => Some(&[0x7F, 0xA1]),
        0x27 => Some(&[0xA4, 0xAD]),
        0x28 => Some(&[0xAC, 0xA4]),
        0x29 => Some(&[0xA4, 0x7F, 0xB3]),
        0x2A => Some(&[0x9D, 0x7F]),
        0x2B => Some(&[0xA4, 0xB2]),
        0x2C => Some(&[0xA4, 0x7F, 0xB8, 0xAE, 0xB4]),
        0x2D => Some(&[0xB2, 0xA4]),
        0x2E => Some(&[0xAD, 0xA4]),
        0x2F => Some(&[0x7F, 0xA7]),
        0x30 => Some(&[0x88, 0x7F]),
        0x31 => Some(&[0xAE, 0xB4, 0xB1]),
        0x32 => Some(&[0x98, 0xAE, 0xB4]),
        0x33 => Some(&[0xAD, 0xA3]),
        0x34 => Some(&[0xAE, 0xB6]),
        0x35 => Some(&[0x7F, 0xA2]),
        0x36 => Some(&[0x7F, 0xB6, 0xA0]),
        0x37 => Some(&[0xAE, 0xAC, 0xA4]),
        0x38 => Some(&[0xA0, 0xB1, 0xA4]),
        0x39 => Some(&[0x93, 0xA7, 0xA4]),
        0x3A => Some(&[0xB3, 0xC0, 0xB2]),
        0x3B => Some(&[0xB4, 0xB3]),
        0x3C => Some(&[0xAD, 0xB3]),
        0x3D => Some(&[0xB3, 0xA7, 0xA4]),
        0x3E => Some(&[0xB8, 0xAE, 0xB4]),
        0x3F => Some(&[0xA8, 0xAD, 0xA6]),
        0x40 => Some(&[0xA7, 0xA0, 0xB3]),
        0x41 => Some(&[0xA0, 0xAD, 0xA3]),
        0x42 => Some(&[0xA5, 0xAE, 0xB1]),
        0x43 => Some(&[0xA0, 0xAB, 0xAB]),
        0x44 => Some(&[0xA7, 0xA4, 0xB1, 0xA4]),
        0x45 => Some(&[0xB3, 0xA7, 0xA0, 0xB3]),
        0x46 => Some(&[0xA7, 0xA0, 0xB5, 0xA4]),
        0x47 => Some(&[0xB1, 0xA0, 0xA8, 0xAD]),
        0x48 => Some(&[0xB3, 0xA7, 0xA8, 0xB2]),
        0x49 => Some(&[0xA8, 0xA6, 0xA7, 0xB3]),
        0x4A => Some(&[0xB6, 0xA8, 0xB3, 0xA7]),
        0x4B => Some(&[0xAE, 0xB4, 0xAB, 0xA3]),
        0x4C => Some(&[0xA0, 0xB3, 0xB3, 0xAB, 0xA4]),
        _ => None,
    }
}

fn decode_v9_to_char(data: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(data.len());

    for &b in data {
        if let Some(expansion) = expand_v9_ngram(b) {
            decoded.extend_from_slice(expansion);
            continue;
        }

        decoded.push(b);
        if b == 0x52 || b == 0x53 {
            break;
        }
    }

    decoded
}

fn convert_mailmsg_v9_to_v10_bytes(
    bytes: &[u8; MAIL_STRUCT_LEN],
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; MAIL_STRUCT_LEN] {
    let decoded = decode_v9_to_char(&bytes[0..MAIL_MSG_LENGTH]);

    if decoded.len() > MAIL_MSG_LENGTH {
        log.error(
            log_source,
            &format!(
                "Decoded mail message is too large to fit in v10 message buffer ({} > {})",
                decoded.len(),
                MAIL_MSG_LENGTH
            ),
        );
    }

    let mut out = *bytes;
    out[0..MAIL_MSG_LENGTH].fill(0);

    let copy_len = decoded.len().min(MAIL_MSG_LENGTH);
    out[0..copy_len].copy_from_slice(&decoded[..copy_len]);
    out
}

fn fix_mail_block(
    source: &SaveBinary,
    save: &mut SaveBinary,
    src_base: Address,
    dst_base: Address,
    count: usize,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> SaveResult<()> {
    for i in 0..count {
        let src_addr = Address(src_base.0 + (i as u32) * (MAIL_STRUCT_LEN as u32));
        let dst_addr = Address(dst_base.0 + (i as u32) * (MAIL_STRUCT_LEN as u32));

        let start = src_addr.as_usize();
        let end = start + MAIL_STRUCT_LEN;
        if end > source.len() {
            return Err(SaveError::RangeOutOfBounds {
                range: AddressRange::new(src_addr, Address(src_addr.0 + MAIL_STRUCT_LEN as u32)),
                len: source.len(),
            });
        }

        let mut bytes = [0u8; MAIL_STRUCT_LEN];
        bytes.copy_from_slice(&source.as_bytes()[start..end]);
        let new_bytes = convert_mailmsg_v9_to_v10_bytes(&bytes, log, log_source);
        save.write_bytes(dst_addr, &new_bytes)?;
    }

    Ok(())
}

impl Patch for MigrationV9ToV10 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: "polished.migration.v9_to_v10",
            kind: PatchKind::Migration,
            from_version: Some(9),
            to_version: Some(10),
        }
    }

    fn apply(&self, save: &mut SaveBinary, symbols_v9: &SymbolDatabase) -> SaveResult<()> {
        scaffold::apply_noop_log(save, symbols_v9, |save, symbols, log| {
            self.apply_impl(save, symbols, log)
        })
    }

    fn apply_with_log(
        &self,
        save: &mut SaveBinary,
        symbols: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        self.apply_impl(save, symbols, log)
    }
}

impl MigrationV9ToV10 {
    fn fix_text_speed_bits(
        &self,
        save: &mut SaveBinary,
        addrs_v10: &PolishedAddrs<'_>,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        let opt1 = addrs_v10.options("wOptions1")?;
        let opt1_byte = save.read_u8(opt1)?;
        let original_bits = opt1_byte & TEXT_DELAY_MASK;
        let reversed_bits = TEXT_DELAY_MASK.wrapping_sub(original_bits);

        match original_bits {
            0x00 => {
                if reversed_bits != 0x03 {
                    log.error(self.metadata().id, "Text speed is set to an invalid value.");
                }
            }
            0x01 => {
                if reversed_bits != 0x02 {
                    log.error(self.metadata().id, "Text speed is set to an invalid value.");
                }
            }
            0x02 => {
                if reversed_bits != 0x01 {
                    log.error(self.metadata().id, "Text speed is set to an invalid value.");
                }
            }
            0x03 => {
                if reversed_bits != 0x00 {
                    log.error(self.metadata().id, "Text speed is set to an invalid value.");
                }
            }
            _ => {
                let msg = "Text speed is set to an invalid value.";
                log.error(self.metadata().id, msg);
                return Err(SaveError::InvalidSaveState {
                    reason: msg.to_string(),
                });
            }
        }
        save.write_u8(opt1, (opt1_byte & !TEXT_DELAY_MASK) | reversed_bits)?;
        Ok(())
    }

    fn apply_impl(
        &self,
        save: &mut SaveBinary,
        symbols_v9: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        validate_primary_and_backup_checksums_with_log(save, symbols_v9, log, self.metadata().id)?;
        let source = save.clone();

        let current = get_save_version(save)?;
        if current != 9 {
            return Err(SaveError::InvalidSaveState {
                reason: format!("expected v9 save, got v{current}"),
            });
        }

        let symbols_v10 = symbols_for_version(SupportedSaveVersion::V10)?;

        let addrs_v9 = PolishedAddrs::new(symbols_v9);
        let addrs_v10 = PolishedAddrs::new(&symbols_v10);

        common::require_player_in_pokecenter_2f(&source, symbols_v9, log, self.metadata().id)?;

        self.fix_text_speed_bits(save, &addrs_v10, log)?;

        let init_opts2 = addrs_v10.options("wInitialOptions2")?;
        let mut init_opts2_byte = save.read_u8(init_opts2)?;
        init_opts2_byte &= !(1u8 << NO_EXP_OPT);
        init_opts2_byte |= 1u8 << RESET_INIT_OPTS;
        save.write_u8(init_opts2, init_opts2_byte)?;

        let magikarp_name = addrs_v9.pokemon("wMagikarpRecordHoldersName")?;
        let name_start = magikarp_name.as_usize();
        let name_end = name_start + RALPH_NAME.len();
        if name_end <= source.len() {
            let is_ralph = source.as_bytes()[name_start..name_end] == *RALPH_NAME;
            if is_ralph {
                let magikarp_len_v9 = addrs_v9.pokemon("wBestMagikarpLengthMm")?;
                let magikarp_length = source.read_u16_be(magikarp_len_v9)?;
                if magikarp_length == 0x0306 {
                    save.write_u16_be(addrs_v10.pokemon("wBestMagikarpLengthMm")?, 0x042B)?;
                }
            }
        }

        let event_flags_v9 = addrs_v9.player("wEventFlags")?;
        let event_flags_v10 = addrs_v10.player("wEventFlags")?;
        save.clear_len(
            event_flags_v10,
            Size(bits_to_bytes(v9_to_v10_maps::NUM_EVENTS) as u32),
        )?;

        map_bitset(
            &source,
            event_flags_v9,
            v9_to_v10_maps::NUM_EVENTS,
            save,
            event_flags_v10,
            v9_to_v10_maps::NUM_EVENTS,
            |i| {
                let mapped = v9_to_v10_maps::map_v9_event_flag_to_v10(i as u16);
                (mapped != v9_to_v10_maps::INVALID_U16).then_some(mapped as usize)
            },
            |i| {
                log.warn(
                    self.metadata().id,
                    &format!("Event flag {i} not found in version 10 event flag list."),
                );
            },
        )?;

        fix_mail_block(
            &source,
            save,
            symbols_v9.sram_absolute_address("sPartyMail")?,
            symbols_v10.sram_absolute_address("sPartyMail")?,
            PARTY_LENGTH,
            log,
            self.metadata().id,
        )?;
        fix_mail_block(
            &source,
            save,
            symbols_v9.sram_absolute_address("sPartyMailBackup")?,
            symbols_v10.sram_absolute_address("sPartyMailBackup")?,
            PARTY_LENGTH,
            log,
            self.metadata().id,
        )?;
        fix_mail_block(
            &source,
            save,
            symbols_v9.sram_absolute_address("sMailbox")?,
            symbols_v10.sram_absolute_address("sMailbox")?,
            MAILBOX_CAPACITY,
            log,
            self.metadata().id,
        )?;
        fix_mail_block(
            &source,
            save,
            symbols_v9.sram_absolute_address("sMailboxBackup")?,
            symbols_v10.sram_absolute_address("sMailboxBackup")?,
            MAILBOX_CAPACITY,
            log,
            self.metadata().id,
        )?;

        save.write_u8(addrs_v10.player("wCurMapSceneScriptCount")?, 0)?;
        save.write_u8(addrs_v10.player("wCurMapCallbackCount")?, 0)?;
        save.write_u16_le(addrs_v10.player("wCurMapSceneScriptPointer")?, 0)?;

        common::reset_invalid_prev_map_warp(
            save,
            &symbols_v10,
            log,
            self.metadata().id,
            PLAINBADGE,
            GOLDENROD_POKECOM_CENTER_1F,
            PLAYERS_HOUSE_1F,
        )?;

        save.write_u16_be(Address(SAVE_VERSION_ABS_ADDRESS), 10)?;

        let src_game_start = symbols_v9.sram_absolute_address("sGameData")?;
        let src_game_end = symbols_v9.sram_absolute_address("sGameDataEnd")?;
        let backup_start = symbols_v10.sram_absolute_address("sBackupGameData")?;
        save.copy_from_other(
            &source,
            src_game_start,
            backup_start,
            Size(src_game_end.0 - src_game_start.0),
        )?;

        let main_checksum = calculate_main_save_checksum(save, &symbols_v10)?;
        let backup_checksum = calculate_backup_save_checksum(save, &symbols_v10)?;
        write_main_save_checksum(save, main_checksum)?;
        write_backup_save_checksum(save, backup_checksum)?;

        Ok(())
    }
}
