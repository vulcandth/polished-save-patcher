use gb_save_core::{
    bits_to_bytes, remap_fixed_len_u8_skip_zero, Address, Patch, PatchKind, PatchLogSink,
    PatchMetadata, SaveBinary, SaveError, SaveResult, Size, SymbolDatabase,
};

use crate::{
    calculate_backup_save_checksum, calculate_main_save_checksum, calculate_newbox_checksum,
    extract_stored_newbox_checksum, get_save_version, read_backup_save_checksum,
    read_main_save_checksum, symbols_for_version, write_backup_save_checksum,
    write_main_save_checksum, write_newbox_checksum, SupportedSaveVersion,
    SAVE_VERSION_ABS_ADDRESS,
};

use super::structs_v8::{
    BREEDMON_LEN, HOFMON_LEN, MAILMSG_LEN, MON_NAME_LENGTH, PARTYMON_LEN, PARTY_LENGTH,
    PLAYER_NAME_LENGTH, ROAMMON_LEN, SAVEMON_LEN,
};
use super::{common, scaffold, v7_to_v8_maps};

mod boxes;
mod helpers;
mod mon_conversion;

use self::boxes::migrate_newbox;
use self::helpers::{contains_u16, convert_item_list, map_and_write_map_group_number};
use self::mon_conversion::{
    convert_breedmon_v7_to_v8_bytes, convert_hofmon_v7_to_v8_bytes, convert_mailmsg_v7_to_v8_bytes,
    convert_party_v7_to_v8_bytes, convert_roam_v7_to_v8_bytes, convert_savemon_v7_to_v8_bytes,
};

const SHAMOUTI_POKECENTER_1F_GROUP: u8 = 31;
const SHAMOUTI_POKECENTER_1F_MAP: u8 = 8;

const NUM_OBJECT_STRUCTS: usize = 13;
const OBJECT_PALETTE_V7: u32 = 0x06;
const OBJECT_LENGTH_V7: u32 = 0x21;
const OBJECT_LENGTH_V8: u32 = 0x22;

const NUM_KEY_ITEMS_V7: usize = 0x1D;
const NUM_APRICORNS: u32 = 0x07;

const NUM_EVENTS: usize = 0x8FF;
const NUM_FRUIT_TREES_V7: usize = 0x23;
const NUM_FRUIT_TREES_V8: usize = 0x2E;
const NUM_LANDMARKS_V8: usize = 0x91;

const CONTACT_LIST_SIZE_V7: usize = 30;
const NUM_PHONE_CONTACTS_V8: usize = 0x25;

const NUM_SPAWNS_V7: usize = 30;
const NUM_SPAWNS_V8: usize = 34;

const NUM_POKEMON_V7: usize = 0xFE;
const NUM_UNIQUE_POKEMON_V8: usize = 0x188;

const MONDB_ENTRIES_V7: usize = 167;
const MONDB_ENTRIES_A_V8: usize = 167;
const MONDB_ENTRIES_B_V8: usize = 28;
const MONDB_ENTRIES_C_V8: usize = 12;
const MONDB_ENTRIES_V8: usize = MONDB_ENTRIES_A_V8 + MONDB_ENTRIES_B_V8 + MONDB_ENTRIES_C_V8;

const MONS_PER_BOX: usize = 20;
const MIN_MONDB_SLACK: usize = 10;
const NUM_BOXES_V7: usize = (MONDB_ENTRIES_V7 * 2 - MIN_MONDB_SLACK) / MONS_PER_BOX;
const NUM_BOXES_V8: usize = (MONDB_ENTRIES_V8 * 2 - MIN_MONDB_SLACK) / MONS_PER_BOX;
const BOX_NAME_LENGTH: usize = 9;
const NEWBOX_SIZE: usize = MONS_PER_BOX + MONS_PER_BOX.div_ceil(8) + BOX_NAME_LENGTH + 1;

const BATTLETOWER_PARTYDATA_SIZE: u32 = 6;
const MAILBOX_CAPACITY: usize = 10;
const NUM_HOF_TEAMS_V8: usize = 10;
const HOF_LENGTH: usize = 1 + HOFMON_LEN * PARTY_LENGTH + 1;

const INVALID_U16: u16 = 0xFFFF;

const MAGIKARP_V8: u16 = 0x81;
const GYARADOS_V8: u16 = 0x82;
const GYARADOS_RED_FORM_V7: u8 = 0x11;
const GYARADOS_RED_FORM_V8: u8 = 0x15;
const PIKACHU_V8: u16 = 0x19;
const PIKACHU_SURF_FORM_V7: u8 = 0x03;
const PIKACHU_FLY_FORM_V7: u8 = 0x02;
const SURF_V7: u8 = 0x39;
const FLY_V7: u8 = 0x13;

const AFFECTION_OPT: u8 = 5;
const EVS_OPT_CLASSIC: u8 = 1;
const RESET_INIT_OPTS: u8 = 7;

const HO_OH_V8: u16 = 0xFA;
const LUGIA_V8: u16 = 0xF9;
const RAIKOU_V8: u16 = 0xF3;
const ENTEI_V8: u16 = 0xF4;
const SUICUNE_V8: u16 = 0xF5;
const ARTICUNO_V8: u16 = 0x90;
const ZAPDOS_V8: u16 = 0x91;
const MOLTRES_V8: u16 = 0x92;
const MEW_V8: u16 = 0x97;
const MEWTWO_V8: u16 = 0x96;
const CELEBI_V8: u16 = 0xFB;
const SUDOWOODO_V8: u16 = 0xB9;

const EVENT_CRYS_IN_NAVEL_ROCK: u16 = 0x108;

// v8 layout sizes/offsets live in structs_v8 (no packed structs / no unsafe).

#[derive(Debug)]
pub struct MigrationV7ToV8;

pub static MIGRATION_V7_TO_V8: MigrationV7ToV8 = MigrationV7ToV8;

impl MigrationV7ToV8 {
    fn options_address(symbols: &SymbolDatabase, wram_symbol: &str) -> SaveResult<Address> {
        symbols.wram_relative_to_sram_absolute_address("wOptions", "sOptions", wram_symbol)
    }

    fn player_data_address(symbols: &SymbolDatabase, wram_symbol: &str) -> SaveResult<Address> {
        symbols.wram_relative_to_sram_absolute_address("wPlayerData", "sPlayerData", wram_symbol)
    }

    fn map_data_address(symbols: &SymbolDatabase, wram_symbol: &str) -> SaveResult<Address> {
        symbols.wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", wram_symbol)
    }

    fn pokemon_data_address(symbols: &SymbolDatabase, wram_symbol: &str) -> SaveResult<Address> {
        symbols.wram_relative_to_sram_absolute_address("wPokemonData", "sPokemonData", wram_symbol)
    }
}

impl Patch for MigrationV7ToV8 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: "polished.migration.v7_to_v8",
            kind: PatchKind::Migration,
            from_version: Some(7),
            to_version: Some(8),
        }
    }

    fn apply(&self, save: &mut SaveBinary, symbols_v7: &SymbolDatabase) -> SaveResult<()> {
        scaffold::apply_noop_log(save, symbols_v7, |save, symbols, log| {
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

impl MigrationV7ToV8 {
    fn apply_impl(
        &self,
        save: &mut SaveBinary,
        symbols_v7: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        let stored_main = read_main_save_checksum(save)?;
        let calc_main = calculate_main_save_checksum(save, symbols_v7)?;
        if stored_main != calc_main {
            let start = symbols_v7.sram_absolute_address("sGameData")?;
            let end = symbols_v7.sram_absolute_address("sGameDataEnd")?;
            log.error(self.metadata().id, &format!("sGameData: {:x}", start.0));
            log.error(self.metadata().id, &format!("sGameDataEnd: {:x}", end.0));
            log.error(
                self.metadata().id,
                &format!("Checksum mismatch! Expected: {calc_main:04x}, got: {stored_main:04x}",),
            );
            return Err(SaveError::ChecksumMismatch {
                which: "main",
                stored: stored_main,
                calculated: calc_main,
            });
        }

        let stored_backup = read_backup_save_checksum(save)?;
        let calc_backup = calculate_backup_save_checksum(save, symbols_v7)?;
        if stored_backup != calc_backup {
            log.error(
                self.metadata().id,
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

        let source = save.clone();

        let current = get_save_version(save)?;
        if current != 7 {
            return Err(SaveError::InvalidSaveState {
                reason: format!("expected v7 save, got v{current}"),
            });
        }

        common::require_player_in_pokecenter_2f(&source, symbols_v7, log, self.metadata().id)?;

        let prev_group_addr = Self::map_data_address(symbols_v7, "wBackupMapGroup")?;
        let prev_num_addr = Self::map_data_address(symbols_v7, "wBackupMapNumber")?;
        let prev_group = source.read_u8(prev_group_addr)?;
        let prev_num = source.read_u8(prev_num_addr)?;
        if prev_group == SHAMOUTI_POKECENTER_1F_GROUP && prev_num == SHAMOUTI_POKECENTER_1F_MAP {
            let msg = "Due to a change in map blocks, we cannot support saving in the Shamouti PKMN center!";
            log.error(self.metadata().id, msg);
            return Err(SaveError::InvalidSaveState {
                reason: msg.to_string(),
            });
        }

        let symbols_v8 = symbols_for_version(SupportedSaveVersion::V8)?;

        let mut seen_mons: Vec<u16> = Vec::new();
        let mut caught_mons: Vec<u16> = Vec::new();

        migrate_newbox(&source, save, symbols_v7, &symbols_v8, "sNewBox")?;
        migrate_newbox(&source, save, symbols_v7, &symbols_v8, "sBackupNewBox")?;

        // Box mon storage
        let s_box_mons1_v7 = symbols_v7.sram_absolute_address("sBoxMons1")?;
        let s_box_mons1a_v8 = symbols_v8.sram_absolute_address("sBoxMons1A")?;
        save.copy_from_other(
            &source,
            s_box_mons1_v7,
            s_box_mons1a_v8,
            Size((MONDB_ENTRIES_A_V8 * SAVEMON_LEN) as u32),
        )?;
        save.clear_len(
            symbols_v8.sram_absolute_address("sBoxMons1B")?,
            Size((MONDB_ENTRIES_B_V8 * SAVEMON_LEN) as u32),
        )?;
        save.clear_len(
            symbols_v8.sram_absolute_address("sBoxMons1C")?,
            Size((MONDB_ENTRIES_C_V8 * SAVEMON_LEN) as u32),
        )?;

        let s_box_mons2_v7 = symbols_v7.sram_absolute_address("sBoxMons2")?;
        let s_box_mons2a_v8 = symbols_v8.sram_absolute_address("sBoxMons2A")?;
        save.copy_from_other(
            &source,
            s_box_mons2_v7,
            s_box_mons2a_v8,
            Size((MONDB_ENTRIES_A_V8 * SAVEMON_LEN) as u32),
        )?;
        save.clear_len(
            symbols_v8.sram_absolute_address("sBoxMons2B")?,
            Size((MONDB_ENTRIES_B_V8 * SAVEMON_LEN) as u32),
        )?;
        save.clear_len(
            symbols_v8.sram_absolute_address("sBoxMons2C")?,
            Size((MONDB_ENTRIES_C_V8 * SAVEMON_LEN) as u32),
        )?;

        // Patch sBoxMons1A/2A when checksums match
        for i in 0..MONDB_ENTRIES_A_V8 {
            let addr = Address(s_box_mons1a_v8.0 + (i as u32) * (SAVEMON_LEN as u32));
            let calc = calculate_newbox_checksum(save, addr)?;
            let stored = extract_stored_newbox_checksum(save, addr)?;
            if calc == stored {
                let mut bytes = [0u8; SAVEMON_LEN];
                bytes.copy_from_slice(
                    &save.as_bytes()[addr.as_usize()..addr.as_usize() + SAVEMON_LEN],
                );
                let new_bytes = convert_savemon_v7_to_v8_bytes(
                    &bytes,
                    &mut seen_mons,
                    &mut caught_mons,
                    log,
                    self.metadata().id,
                );
                save.write_bytes(addr, &new_bytes)?;
                write_newbox_checksum(save, addr)?;
            }
        }
        for i in 0..MONDB_ENTRIES_A_V8 {
            let addr = Address(s_box_mons2a_v8.0 + (i as u32) * (SAVEMON_LEN as u32));
            let calc = calculate_newbox_checksum(save, addr)?;
            let stored = extract_stored_newbox_checksum(save, addr)?;
            if calc == stored {
                let mut bytes = [0u8; SAVEMON_LEN];
                bytes.copy_from_slice(
                    &save.as_bytes()[addr.as_usize()..addr.as_usize() + SAVEMON_LEN],
                );
                let new_bytes = convert_savemon_v7_to_v8_bytes(
                    &bytes,
                    &mut seen_mons,
                    &mut caught_mons,
                    log,
                    self.metadata().id,
                );
                save.write_bytes(addr, &new_bytes)?;
                write_newbox_checksum(save, addr)?;
            }
        }

        // Copy [sLinkBattleResults, sLinkBattleStatsEnd)
        let link_start7 = symbols_v7.sram_absolute_address("sLinkBattleResults")?;
        let link_end7 = symbols_v7.sram_absolute_address("sLinkBattleStatsEnd")?;
        save.copy_from_other(
            &source,
            link_start7,
            symbols_v8.sram_absolute_address("sLinkBattleResults")?,
            Size(link_end7.0 - link_start7.0),
        )?;

        // Copy [sBattleTowerChallengeState, sBT_OTMonParty3 + BATTLETOWER_PARTYDATA_SIZE + 1)
        let bt_start7 = symbols_v7.sram_absolute_address("sBattleTowerChallengeState")?;
        let bt_end7 =
            symbols_v7.sram_absolute_address("sBT_OTMonParty3")?.0 + BATTLETOWER_PARTYDATA_SIZE + 1;
        save.copy_from_other(
            &source,
            bt_start7,
            symbols_v8.sram_absolute_address("sBattleTowerChallengeState")?,
            Size(bt_end7 - bt_start7.0),
        )?;

        // Copy [sPartyMail, sSaveVersion)
        let party_mail7 = symbols_v7.sram_absolute_address("sPartyMail")?;
        let save_version7 = symbols_v7.sram_absolute_address("sSaveVersion")?;
        save.copy_from_other(
            &source,
            party_mail7,
            symbols_v8.sram_absolute_address("sPartyMail")?,
            Size(save_version7.0 - party_mail7.0),
        )?;

        // Fix mail (party + mailbox, both primary and backup)
        for (name, count) in [
            ("sPartyMail", PARTY_LENGTH),
            ("sPartyMailBackup", PARTY_LENGTH),
            ("sMailbox", MAILBOX_CAPACITY),
            ("sMailboxBackup", MAILBOX_CAPACITY),
        ] {
            let base = symbols_v8.sram_absolute_address(name)?;
            for i in 0..count {
                let addr = Address(base.0 + (i as u32) * (MAILMSG_LEN as u32));
                let mut bytes = [0u8; MAILMSG_LEN];
                bytes.copy_from_slice(
                    &save.as_bytes()[addr.as_usize()..addr.as_usize() + MAILMSG_LEN],
                );
                let new_bytes = convert_mailmsg_v7_to_v8_bytes(&bytes);
                save.write_bytes(addr, &new_bytes)?;
            }
        }

        // Copy [sUpgradeStep, sWritingBackup + 1]
        let upgrade7 = symbols_v7.sram_absolute_address("sUpgradeStep")?;
        let writing_backup7 = symbols_v7.sram_absolute_address("sWritingBackup")?;
        save.copy_from_other(
            &source,
            upgrade7,
            symbols_v8.sram_absolute_address("sUpgradeStep")?,
            Size((writing_backup7.0 + 1) - upgrade7.0),
        )?;

        // Copy [sRTCStatusFlags, sLuckyIDNumber + 2]
        let rtc7 = symbols_v7.sram_absolute_address("sRTCStatusFlags")?;
        let lucky7 = symbols_v7.sram_absolute_address("sLuckyIDNumber")?;
        save.copy_from_other(
            &source,
            rtc7,
            symbols_v8.sram_absolute_address("sRTCStatusFlags")?,
            Size((lucky7.0 + 2) - rtc7.0),
        )?;

        // Copy [sOptions, sGameData)
        let options7 = symbols_v7.sram_absolute_address("sOptions")?;
        let game_data7 = symbols_v7.sram_absolute_address("sGameData")?;
        save.copy_from_other(
            &source,
            options7,
            symbols_v8.sram_absolute_address("sOptions")?,
            Size(game_data7.0 - options7.0),
        )?;

        // Options tweaks
        let init_opts8 = Self::options_address(&symbols_v8, "wInitialOptions")?;
        save.write_bit(init_opts8, AFFECTION_OPT, false)?;
        let init_opts2_8 = Address(init_opts8.0 + 1);
        save.write_u8(init_opts2_8, 0)?;
        save.write_bit(init_opts2_8, EVS_OPT_CLASSIC, true)?;
        save.write_bit(init_opts2_8, RESET_INIT_OPTS, true)?;

        // Keep sBackupOptions in sync without overwriting sBackupGameData.
        let options8 = symbols_v8.sram_absolute_address("sOptions")?;
        let backup_options8 = symbols_v8.sram_absolute_address("sBackupOptions")?;
        let backup_game_data8 = symbols_v8.sram_absolute_address("sBackupGameData")?;
        let len = (backup_game_data8.0 - backup_options8.0) as usize;
        let bytes = save.as_bytes()[options8.as_usize()..options8.as_usize() + len].to_vec();
        save.write_bytes(backup_options8, &bytes)?;

        // Copy [wPlayerData, wObjectStructs)
        let player_data7 = Self::player_data_address(symbols_v7, "wPlayerData")?;
        let object_structs7 = Self::player_data_address(symbols_v7, "wObjectStructs")?;
        save.copy_from_other(
            &source,
            player_data7,
            Self::player_data_address(&symbols_v8, "wPlayerData")?,
            Size(object_structs7.0 - player_data7.0),
        )?;

        // Clear unused bytes after wRTC: [wRTC + 4, wRTC + 8)
        let w_rtc8 = Self::player_data_address(&symbols_v8, "wRTC")?;
        save.clear_len(Address(w_rtc8.0 + 4), Size(4))?;

        // Patch object structs (expand by 1 byte)
        let obj_base7 = Self::player_data_address(symbols_v7, "wObjectStructs")?;
        let obj_base8 = Self::player_data_address(&symbols_v8, "wObjectStructs")?;
        for i in 0..NUM_OBJECT_STRUCTS {
            let src_addr = Address(obj_base7.0 + (i as u32) * OBJECT_LENGTH_V7);
            let dst_addr = Address(obj_base8.0 + (i as u32) * OBJECT_LENGTH_V8);

            let object_struct = if i == 0 {
                "wPlayerStruct".to_string()
            } else {
                format!("wObject{i}Struct")
            };
            if let Ok(expected7) = Self::player_data_address(symbols_v7, &object_struct) {
                if expected7 != src_addr {
                    log.error(
                        self.metadata().id,
                        &format!(
                            "Unexpected address for {object_struct} in version 7 save file: {:x}, expected: {:x}",
                            src_addr.0,
                            expected7.0
                        ),
                    );
                }
            }
            if let Ok(expected8) = Self::player_data_address(&symbols_v8, &object_struct) {
                if expected8 != dst_addr {
                    log.error(
                        self.metadata().id,
                        &format!(
                            "Unexpected address for {object_struct} in version 8 save file: {:x}, expected: {:x}",
                            dst_addr.0,
                            expected8.0
                        ),
                    );
                }
            }

            save.copy_from_other(&source, src_addr, dst_addr, Size(OBJECT_LENGTH_V7))?;
            let palette = source.read_u8(Address(src_addr.0 + OBJECT_PALETTE_V7))? & 0x0F;
            save.write_u8(Address(dst_addr.0 + OBJECT_LENGTH_V7), palette)?;
        }

        // Copy [wObjectStructsEnd, wBattleFactorySwapCount + 1]
        let obj_end7 = Self::player_data_address(symbols_v7, "wObjectStructsEnd")?;
        let bf_swap7 = Self::player_data_address(symbols_v7, "wBattleFactorySwapCount")?;
        save.copy_from_other(
            &source,
            obj_end7,
            Self::player_data_address(&symbols_v8, "wObjectStructsEnd")?,
            Size((bf_swap7.0 + 1) - obj_end7.0),
        )?;

        // Copy [wMapObjects, wEnteredMapFromContinue)
        let map_objects7 = Self::player_data_address(symbols_v7, "wMapObjects")?;
        let entered7 = Self::player_data_address(symbols_v7, "wEnteredMapFromContinue")?;
        save.copy_from_other(
            &source,
            map_objects7,
            Self::player_data_address(&symbols_v8, "wMapObjects")?,
            Size(entered7.0 - map_objects7.0),
        )?;
        // Copy wEnteredMapFromContinue
        save.write_u8(
            Self::player_data_address(&symbols_v8, "wEnteredMapFromContinue")?,
            source.read_u8(entered7)?,
        )?;
        // Copy wStatusFlags3
        let status_flags3_7 = Self::player_data_address(symbols_v7, "wStatusFlags3")?;
        save.write_u8(
            Self::player_data_address(&symbols_v8, "wStatusFlags3")?,
            source.read_u8(status_flags3_7)?,
        )?;

        // Copy [wTimeOfDayPal, wBadgesEnd)
        let tod7 = Self::player_data_address(symbols_v7, "wTimeOfDayPal")?;
        let badges_end7 = Self::player_data_address(symbols_v7, "wBadgesEnd")?;
        save.copy_from_other(
            &source,
            tod7,
            Self::player_data_address(&symbols_v8, "wTimeOfDayPal")?,
            Size(badges_end7.0 - tod7.0),
        )?;
        save.clear_len(
            Address(Self::player_data_address(&symbols_v8, "wTimeOfDayPal")?.0 + 1),
            Size(4),
        )?;

        // Pokemon journals
        let journals8 = Self::player_data_address(&symbols_v8, "wPokemonJournals")?;
        let journals_end8 = Self::player_data_address(&symbols_v8, "wPokemonJournalsEnd")?;
        save.clear_len(journals8, Size(journals_end8.0 - journals8.0))?;
        let journals7 = Self::player_data_address(symbols_v7, "wPokemonJournals")?;
        let journals_end7 = Self::player_data_address(symbols_v7, "wPokemonJournalsEnd")?;
        save.copy_from_other(
            &source,
            journals7,
            journals8,
            Size(journals_end7.0 - journals7.0),
        )?;

        // TMs/HMs
        let tms7 = Self::player_data_address(symbols_v7, "wTMsHMs")?;
        let tms_end7 = Self::player_data_address(symbols_v7, "wTMsHMsEnd")?;
        save.copy_from_other(
            &source,
            tms7,
            Self::player_data_address(&symbols_v8, "wTMsHMs")?,
            Size(tms_end7.0 - tms7.0),
        )?;

        // Key items: flags -> list
        let key_items8 = Self::player_data_address(&symbols_v8, "wKeyItems")?;
        let key_items_end8 = Self::player_data_address(&symbols_v8, "wKeyItemsEnd")?;
        save.clear_len(key_items8, Size(key_items_end8.0 - key_items8.0))?;
        let key_items7 = Self::player_data_address(symbols_v7, "wKeyItems")?;
        let mut dst_cursor = key_items8;
        for i in 0..NUM_KEY_ITEMS_V7 {
            if source.read_indexed_bit(key_items7, i)? {
                let mapped = v7_to_v8_maps::map_v7_key_item_to_v8(i as u8);
                if mapped == v7_to_v8_maps::INVALID_U8 {
                    log.error(
                        self.metadata().id,
                        &format!("Key Item {i:02x} not found in version 8 key item list.",),
                    );
                } else {
                    save.write_u8(dst_cursor, mapped)?;
                    dst_cursor = Address(dst_cursor.0 + 1);
                }
            }
        }
        save.write_u8(dst_cursor, 0)?;

        // Item lists
        convert_item_list(
            &source,
            save,
            Self::player_data_address(symbols_v7, "wNumItems")?,
            Self::player_data_address(symbols_v7, "wItems")?,
            Self::player_data_address(&symbols_v8, "wNumItems")?,
            Self::player_data_address(&symbols_v8, "wItems")?,
            log,
            self.metadata().id,
        )?;
        convert_item_list(
            &source,
            save,
            Self::player_data_address(symbols_v7, "wNumMedicine")?,
            Self::player_data_address(symbols_v7, "wMedicine")?,
            Self::player_data_address(&symbols_v8, "wNumMedicine")?,
            Self::player_data_address(&symbols_v8, "wMedicine")?,
            log,
            self.metadata().id,
        )?;
        convert_item_list(
            &source,
            save,
            Self::player_data_address(symbols_v7, "wNumBalls")?,
            Self::player_data_address(symbols_v7, "wBalls")?,
            Self::player_data_address(&symbols_v8, "wNumBalls")?,
            Self::player_data_address(&symbols_v8, "wBalls")?,
            log,
            self.metadata().id,
        )?;
        convert_item_list(
            &source,
            save,
            Self::player_data_address(symbols_v7, "wNumBerries")?,
            Self::player_data_address(symbols_v7, "wBerries")?,
            Self::player_data_address(&symbols_v8, "wNumBerries")?,
            Self::player_data_address(&symbols_v8, "wBerries")?,
            log,
            self.metadata().id,
        )?;
        convert_item_list(
            &source,
            save,
            Self::player_data_address(symbols_v7, "wNumPCItems")?,
            Self::player_data_address(symbols_v7, "wPCItems")?,
            Self::player_data_address(&symbols_v8, "wNumPCItems")?,
            Self::player_data_address(&symbols_v8, "wPCItems")?,
            log,
            self.metadata().id,
        )?;

        // Apricorns
        save.copy_from_other(
            &source,
            Self::player_data_address(symbols_v7, "wApricorns")?,
            Self::player_data_address(&symbols_v8, "wApricorns")?,
            Size(NUM_APRICORNS),
        )?;

        // [wPokegearFlags, wAlways0SceneID)
        let pokegear7 = Self::player_data_address(symbols_v7, "wPokegearFlags")?;
        let always0_7 = Self::player_data_address(symbols_v7, "wAlways0SceneID")?;
        save.copy_from_other(
            &source,
            pokegear7,
            Self::player_data_address(&symbols_v8, "wPokegearFlags")?,
            Size(always0_7.0 - pokegear7.0),
        )?;

        // Clear byte before wMooMooBerries
        let moomoo8 = Self::player_data_address(&symbols_v8, "wMooMooBerries")?;
        save.write_u8(Address(moomoo8.0 - 1), 0)?;

        // [wAlways0SceneID, wEcruteakHouseSceneID + 1]
        let ecruteak_house7 = Self::player_data_address(symbols_v7, "wEcruteakHouseSceneID")?;
        save.copy_from_other(
            &source,
            always0_7,
            Self::player_data_address(&symbols_v8, "wAlways0SceneID")?,
            Size((ecruteak_house7.0 + 1) - always0_7.0),
        )?;

        // Clear wEcruteakPokecenter1FSceneID (byte after wEcruteakHouseSceneID)
        let ecruteak_house8 = Self::player_data_address(&symbols_v8, "wEcruteakHouseSceneID")?;
        save.write_u8(Address(ecruteak_house8.0 + 1), 0)?;

        // [wElmsLabSceneID, wEventFlags)
        let elms7 = Self::player_data_address(symbols_v7, "wElmsLabSceneID")?;
        let event_flags7 = Self::player_data_address(symbols_v7, "wEventFlags")?;
        save.copy_from_other(
            &source,
            elms7,
            Self::player_data_address(&symbols_v8, "wElmsLabSceneID")?,
            Size(event_flags7.0 - elms7.0),
        )?;

        // Event flags remap
        let event_flags8 = Self::player_data_address(&symbols_v8, "wEventFlags")?;
        save.clear_len(event_flags8, Size(bits_to_bytes(NUM_EVENTS) as u32))?;
        for i in 0..NUM_EVENTS {
            if source.read_indexed_bit(event_flags7, i)? {
                let mapped = v7_to_v8_maps::map_v7_event_flag_to_v8(i as u16);
                if mapped != INVALID_U16 {
                    save.write_indexed_bit(event_flags8, mapped as usize, true)?;
                } else {
                    log.warn(
                        self.metadata().id,
                        &format!("Event Flag {i} not found in version 8 event flag list.",),
                    );
                }
            }
        }
        save.write_indexed_bit(event_flags8, EVENT_CRYS_IN_NAVEL_ROCK as usize, true)?;

        // Copy wCurBox
        let cur_box7 = Self::player_data_address(symbols_v7, "wCurBox")?;
        save.write_u8(
            Self::player_data_address(&symbols_v8, "wCurBox")?,
            source.read_u8(cur_box7)?,
        )?;

        // Clear [wUsedObjectPals, wNeededPalIndex + 1]
        let used_obj_pals8 = Self::player_data_address(&symbols_v8, "wUsedObjectPals")?;
        let needed_pal_index8 = Self::player_data_address(&symbols_v8, "wNeededPalIndex")?;
        save.clear_len(
            used_obj_pals8,
            Size((needed_pal_index8.0 + 1) - used_obj_pals8.0),
        )?;

        // Fill wLoadedObjPal0-7 with 0xFF
        save.fill_len(
            Self::player_data_address(&symbols_v8, "wLoadedObjPal0")?,
            Size(8),
            0xFF,
        )?;

        // Clear 70 bytes after wEmotePal
        let emote_pal8 = Self::player_data_address(&symbols_v8, "wEmotePal")?;
        save.clear_len(Address(emote_pal8.0 + 1), Size(70))?;

        // Copy [wCelebiEvent, wCurMapCallbacksPointer + 2]
        let celebi7 = Self::player_data_address(symbols_v7, "wCelebiEvent")?;
        let callbacks_ptr7 = Self::player_data_address(symbols_v7, "wCurMapCallbacksPointer")?;
        save.copy_from_other(
            &source,
            celebi7,
            Self::player_data_address(&symbols_v8, "wCelebiEvent")?,
            Size((callbacks_ptr7.0 + 2) - celebi7.0),
        )?;

        // Clear unused byte before wDecoBed
        let deco_bed8 = Self::player_data_address(&symbols_v8, "wDecoBed")?;
        save.write_u8(Address(deco_bed8.0 - 1), 0)?;

        // Copy [wDecoBed, wFruitTreeFlags)
        let deco_bed7 = Self::player_data_address(symbols_v7, "wDecoBed")?;
        let fruit_flags7 = Self::player_data_address(symbols_v7, "wFruitTreeFlags")?;
        save.copy_from_other(
            &source,
            deco_bed7,
            Self::player_data_address(&symbols_v8, "wDecoBed")?,
            Size(fruit_flags7.0 - deco_bed7.0),
        )?;

        // Copy wFruitTreeFlags
        save.copy_from_other(
            &source,
            fruit_flags7,
            Self::player_data_address(&symbols_v8, "wFruitTreeFlags")?,
            Size(bits_to_bytes(NUM_FRUIT_TREES_V7) as u32),
        )?;
        // Clear 19 bytes after wFruitTreeFlags + v8 length
        let fruit_flags8 = Self::player_data_address(&symbols_v8, "wFruitTreeFlags")?;
        save.clear_len(
            Address(fruit_flags8.0 + bits_to_bytes(NUM_FRUIT_TREES_V8) as u32),
            Size(19),
        )?;

        // Clear wNuzlockeLandmarkFlags if present in the symbol set.
        if let Some(nuz_flags8) =
            common::try_player_data_address(&symbols_v8, "wNuzlockeLandmarkFlags")?
        {
            save.clear_len(nuz_flags8, Size(bits_to_bytes(NUM_LANDMARKS_V8) as u32))?;
        }

        // Clear [wHiddenGrottoContents, wCurHiddenGrotto + 1]
        let grotto_contents8 = Self::player_data_address(&symbols_v8, "wHiddenGrottoContents")?;
        let cur_grotto8 = Self::player_data_address(&symbols_v8, "wCurHiddenGrotto")?;
        save.clear_len(
            grotto_contents8,
            Size((cur_grotto8.0 + 1) - grotto_contents8.0),
        )?;

        // Copy [wLuckyNumberDayBuffer, wPhoneList)
        let lucky_buf7 = Self::player_data_address(symbols_v7, "wLuckyNumberDayBuffer")?;
        let phone_list7 = Self::player_data_address(symbols_v7, "wPhoneList")?;
        save.copy_from_other(
            &source,
            lucky_buf7,
            Self::player_data_address(&symbols_v8, "wLuckyNumberDayBuffer")?,
            Size(phone_list7.0 - lucky_buf7.0),
        )?;

        // Phone list list->bitflags
        let phone_list8 = Self::player_data_address(&symbols_v8, "wPhoneList")?;
        save.clear_len(
            phone_list8,
            Size(bits_to_bytes(NUM_PHONE_CONTACTS_V8) as u32),
        )?;
        for i in 0..CONTACT_LIST_SIZE_V7 {
            let v = source.read_u8(Address(phone_list7.0 + i as u32))?;
            if v != 0 {
                let bit = (v - 1) as usize;
                save.write_indexed_bit(phone_list8, bit, true)?;
            }
        }
        let phone_end8 = Self::player_data_address(&symbols_v8, "wPhoneListEnd")?;
        save.write_u8(phone_end8, 0)?;

        // Copy [wParkBallsRemaining, wPlayerDataEnd)
        let park7 = Self::player_data_address(symbols_v7, "wParkBallsRemaining")?;
        let player_end7 = Self::player_data_address(symbols_v7, "wPlayerDataEnd")?;
        save.copy_from_other(
            &source,
            park7,
            Self::player_data_address(&symbols_v8, "wParkBallsRemaining")?,
            Size(player_end7.0 - park7.0),
        )?;

        // Visited spawns remap
        let visited7 = Self::map_data_address(symbols_v7, "wVisitedSpawns")?;
        let visited8 = Self::map_data_address(&symbols_v8, "wVisitedSpawns")?;
        save.clear_len(visited8, Size(bits_to_bytes(NUM_SPAWNS_V8) as u32))?;
        for i in 0..NUM_SPAWNS_V7 {
            if source.read_indexed_bit(visited7, i)? {
                let mapped = v7_to_v8_maps::map_v7_spawn_to_v8(i as u8);
                if mapped != v7_to_v8_maps::INVALID_U8 {
                    save.write_indexed_bit(visited8, mapped as usize, true)?;
                }
            }
        }

        // Copy [wDigWarpNumber, wCurMapDataEnd)
        let digwarp7 = Self::map_data_address(symbols_v7, "wDigWarpNumber")?;
        let map_end7 = Self::map_data_address(symbols_v7, "wCurMapDataEnd")?;
        save.copy_from_other(
            &source,
            digwarp7,
            Self::map_data_address(&symbols_v8, "wDigWarpNumber")?,
            Size(map_end7.0 - digwarp7.0),
        )?;

        map_and_write_map_group_number(
            &source,
            save,
            Self::map_data_address(symbols_v7, "wDigMapGroup")?,
            Self::map_data_address(&symbols_v8, "wDigMapGroup")?,
            Self::map_data_address(symbols_v7, "wDigMapNumber")?,
            Self::map_data_address(&symbols_v8, "wDigMapNumber")?,
        )?;
        map_and_write_map_group_number(
            &source,
            save,
            Self::map_data_address(symbols_v7, "wBackupMapGroup")?,
            Self::map_data_address(&symbols_v8, "wBackupMapGroup")?,
            Self::map_data_address(symbols_v7, "wBackupMapNumber")?,
            Self::map_data_address(&symbols_v8, "wBackupMapNumber")?,
        )?;
        map_and_write_map_group_number(
            &source,
            save,
            Self::map_data_address(symbols_v7, "wLastSpawnMapGroup")?,
            Self::map_data_address(&symbols_v8, "wLastSpawnMapGroup")?,
            Self::map_data_address(symbols_v7, "wLastSpawnMapNumber")?,
            Self::map_data_address(&symbols_v8, "wLastSpawnMapNumber")?,
        )?;
        map_and_write_map_group_number(
            &source,
            save,
            Self::map_data_address(symbols_v7, "wMapGroup")?,
            Self::map_data_address(&symbols_v8, "wMapGroup")?,
            Self::map_data_address(symbols_v7, "wMapNumber")?,
            Self::map_data_address(&symbols_v8, "wMapNumber")?,
        )?;

        // Pokemon data: party
        let party_count7 = Self::pokemon_data_address(symbols_v7, "wPartyCount")?;
        let party_count8 = Self::pokemon_data_address(&symbols_v8, "wPartyCount")?;
        save.write_u8(party_count8, source.read_u8(party_count7)?)?;
        save.clear_len(Address(party_count8.0 + 1), Size(7))?;

        let party_mons7 = Self::pokemon_data_address(symbols_v7, "wPartyMons")?;
        let party_mons8 = Self::pokemon_data_address(&symbols_v8, "wPartyMons")?;
        save.copy_from_other(
            &source,
            party_mons7,
            party_mons8,
            Size((PARTYMON_LEN * PARTY_LENGTH) as u32),
        )?;
        for i in 0..PARTY_LENGTH {
            let addr = Address(party_mons8.0 + (i as u32) * (PARTYMON_LEN as u32));
            let species = save.read_u8(addr)?;
            if species == 0 {
                continue;
            }
            let mut bytes = [0u8; PARTYMON_LEN];
            bytes
                .copy_from_slice(&save.as_bytes()[addr.as_usize()..addr.as_usize() + PARTYMON_LEN]);
            let new_bytes = convert_party_v7_to_v8_bytes(
                &bytes,
                &mut seen_mons,
                &mut caught_mons,
                log,
                self.metadata().id,
            );
            save.write_bytes(addr, &new_bytes)?;
        }

        // Party OT/nicknames
        let party_ots7 = Self::pokemon_data_address(symbols_v7, "wPartyMonOTs")?;
        let party_ots8 = Self::pokemon_data_address(&symbols_v8, "wPartyMonOTs")?;
        save.copy_from_other(
            &source,
            party_ots7,
            party_ots8,
            Size((PARTY_LENGTH * (PLAYER_NAME_LENGTH + 3)) as u32),
        )?;

        let party_nicks7 = Self::pokemon_data_address(symbols_v7, "wPartyMonNicknames")?;
        let party_nicks8 = Self::pokemon_data_address(&symbols_v8, "wPartyMonNicknames")?;
        save.copy_from_other(
            &source,
            party_nicks7,
            party_nicks8,
            Size((PARTY_LENGTH * MON_NAME_LENGTH) as u32),
        )?;
        let party_nicks_end8 = Self::pokemon_data_address(&symbols_v8, "wPartyMonNicknamesEnd")?;
        save.write_u8(party_nicks_end8, 0)?;

        // Clear unused bytes after pokedex arrays
        save.write_u8(
            Self::pokemon_data_address(&symbols_v8, "wEndPokedexCaught")?,
            0,
        )?;
        save.write_u8(
            Self::pokemon_data_address(&symbols_v8, "wEndPokedexSeen")?,
            0,
        )?;

        // Unown unlocks
        let unowns7 = Self::pokemon_data_address(symbols_v7, "wUnlockedUnowns")?;
        let unowns8 = Self::pokemon_data_address(&symbols_v8, "wUnlockedUnowns")?;
        save.write_u8(unowns8, source.read_u8(unowns7)?)?;
        save.clear_len(Address(unowns8.0 + 1), Size(2))?;

        // Copy [wDayCareMan, wBreedMon2 + sizeof(breedmon)]
        let daycare7 = Self::pokemon_data_address(symbols_v7, "wDayCareMan")?;
        let breed2_7 = Self::pokemon_data_address(symbols_v7, "wBreedMon2")?;
        save.copy_from_other(
            &source,
            daycare7,
            Self::pokemon_data_address(&symbols_v8, "wDayCareMan")?,
            Size((breed2_7.0 + BREEDMON_LEN as u32) - daycare7.0),
        )?;

        // Fix breed mon 1/2
        let breed1_8 = Self::pokemon_data_address(&symbols_v8, "wBreedMon1")?;
        let breed1_species8 = Self::pokemon_data_address(&symbols_v8, "wBreedMon1Species")?;
        if save.read_u8(breed1_species8)? != 0 {
            let mut bytes = [0u8; BREEDMON_LEN];
            bytes.copy_from_slice(
                &save.as_bytes()[breed1_8.as_usize()..breed1_8.as_usize() + BREEDMON_LEN],
            );
            let new_bytes = convert_breedmon_v7_to_v8_bytes(
                &bytes,
                &mut seen_mons,
                &mut caught_mons,
                log,
                self.metadata().id,
            );
            save.write_bytes(breed1_8, &new_bytes)?;
        }
        let breed2_8 = Self::pokemon_data_address(&symbols_v8, "wBreedMon2")?;
        let breed2_species8 = Self::pokemon_data_address(&symbols_v8, "wBreedMon2Species")?;
        if save.read_u8(breed2_species8)? != 0 {
            let mut bytes = [0u8; BREEDMON_LEN];
            bytes.copy_from_slice(
                &save.as_bytes()[breed2_8.as_usize()..breed2_8.as_usize() + BREEDMON_LEN],
            );
            let new_bytes = convert_breedmon_v7_to_v8_bytes(
                &bytes,
                &mut seen_mons,
                &mut caught_mons,
                log,
                self.metadata().id,
            );
            save.write_bytes(breed2_8, &new_bytes)?;
        }

        // Clear [wLevelUpMonNickname, wBugContestBackupPartyCount)
        let levelup_nick8 = Self::pokemon_data_address(&symbols_v8, "wLevelUpMonNickname")?;
        let bug_backup_count8 =
            Self::pokemon_data_address(&symbols_v8, "wBugContestBackupPartyCount")?;
        save.clear_len(levelup_nick8, Size(bug_backup_count8.0 - levelup_nick8.0))?;
        save.write_u8(bug_backup_count8, 0)?;

        // Copy [wContestMon, wPokemonDataEnd)
        let contest7 = Self::pokemon_data_address(symbols_v7, "wContestMon")?;
        let pokemon_end7 = Self::pokemon_data_address(symbols_v7, "wPokemonDataEnd")?;
        save.copy_from_other(
            &source,
            contest7,
            Self::pokemon_data_address(&symbols_v8, "wContestMon")?,
            Size(pokemon_end7.0 - contest7.0),
        )?;

        // Fix contest mon
        let contest_species8 = Self::pokemon_data_address(&symbols_v8, "wContestMonSpecies")?;
        if save.read_u8(contest_species8)? != 0 {
            let contest8 = Self::pokemon_data_address(&symbols_v8, "wContestMon")?;
            let mut bytes = [0u8; PARTYMON_LEN];
            bytes.copy_from_slice(
                &save.as_bytes()[contest8.as_usize()..contest8.as_usize() + PARTYMON_LEN],
            );
            let new_bytes = convert_party_v7_to_v8_bytes(
                &bytes,
                &mut seen_mons,
                &mut caught_mons,
                log,
                self.metadata().id,
            );
            save.write_bytes(contest8, &new_bytes)?;
        }

        map_and_write_map_group_number(
            &source,
            save,
            Self::pokemon_data_address(symbols_v7, "wDunsparceMapGroup")?,
            Self::pokemon_data_address(&symbols_v8, "wDunsparceMapGroup")?,
            Self::pokemon_data_address(symbols_v7, "wDunsparceMapNumber")?,
            Self::pokemon_data_address(&symbols_v8, "wDunsparceMapNumber")?,
        )?;

        // Roamers
        for name in ["wRoamMon1", "wRoamMon2", "wRoamMon3"] {
            let base = Self::pokemon_data_address(&symbols_v8, name)?;
            let species = save.read_u8(base)?;
            if species != 0 {
                let mut bytes = [0u8; ROAMMON_LEN];
                bytes.copy_from_slice(
                    &save.as_bytes()[base.as_usize()..base.as_usize() + ROAMMON_LEN],
                );
                let new_bytes = convert_roam_v7_to_v8_bytes(&bytes, log, self.metadata().id);
                save.write_bytes(base, &new_bytes)?;
            } else {
                save.write_u8(Address(base.0 + 2), 0xFF)?;
                save.write_u8(Address(base.0 + 3), 0xFF)?;
            }
        }
        let roam3 = Self::pokemon_data_address(&symbols_v8, "wRoamMon3")?;
        save.clear_len(Address(roam3.0 + ROAMMON_LEN as u32), Size(4))?;

        // Registered key items
        let reg_items8 = Self::pokemon_data_address(&symbols_v8, "wRegisteredItems")?;
        remap_fixed_len_u8_skip_zero(
            save,
            reg_items8,
            4,
            |value| {
                let mapped = v7_to_v8_maps::map_v7_key_item_to_v8(value.wrapping_sub(1));
                (mapped != v7_to_v8_maps::INVALID_U8).then_some(mapped)
            },
            |_, value| {
                log.warn(
                    self.metadata().id,
                    &format!("Registered Item {value:02x} not found in version 8 key item list.",),
                );
                0
            },
        )?;

        // Copy sCheckValue2
        let check2_7 = symbols_v7.sram_absolute_address("sCheckValue2")?;
        let check2_8 = symbols_v8.sram_absolute_address("sCheckValue2")?;
        save.write_u8(check2_8, source.read_u8(check2_7)?)?;

        // Hall of fame
        let hof8 = symbols_v8.sram_absolute_address("sHallOfFame")?;
        let hof_end8 = symbols_v8.sram_absolute_address("sHallOfFameEnd")?;
        let hof7 = symbols_v7.sram_absolute_address("sHallOfFame")?;
        save.copy_from_other(&source, hof7, hof8, Size(hof_end8.0 - hof8.0))?;

        let hof_mon1 = symbols_v8.sram_absolute_address("sHallOfFame01Mon1")?;
        for team in 0..NUM_HOF_TEAMS_V8 {
            let base = Address(hof_mon1.0 + (team as u32) * (HOF_LENGTH as u32));
            for slot in 0..PARTY_LENGTH {
                let addr = Address(base.0 + (slot as u32) * (HOFMON_LEN as u32));
                if save.read_u8(addr)? == 0 {
                    continue;
                }
                let mut bytes = [0u8; HOFMON_LEN];
                bytes.copy_from_slice(
                    &save.as_bytes()[addr.as_usize()..addr.as_usize() + HOFMON_LEN],
                );
                let new_bytes = convert_hofmon_v7_to_v8_bytes(
                    &bytes,
                    &mut seen_mons,
                    &mut caught_mons,
                    log,
                    self.metadata().id,
                );
                save.write_bytes(addr, &new_bytes)?;
            }
        }

        // Pokedex caught/seen
        let pokedex_caught7 = Self::pokemon_data_address(symbols_v7, "wPokedexCaught")?;
        let pokedex_caught8 = Self::pokemon_data_address(&symbols_v8, "wPokedexCaught")?;
        save.clear_len(
            pokedex_caught8,
            Size(bits_to_bytes(NUM_UNIQUE_POKEMON_V8) as u32),
        )?;
        for i in 0..NUM_POKEMON_V7 {
            if source.read_indexed_bit(pokedex_caught7, i)? {
                let idx_v7 = (i + 1) as u16;
                let mapped = v7_to_v8_maps::map_v7_pkmn_to_v8(idx_v7);
                if mapped != INVALID_U16 {
                    save.write_indexed_bit(pokedex_caught8, (mapped - 1) as usize, true)?;
                }
            }
        }
        for &mon in &caught_mons {
            if mon != 0 {
                save.write_indexed_bit(pokedex_caught8, (mon - 1) as usize, true)?;
            }
        }

        let pokedex_seen7 = Self::pokemon_data_address(symbols_v7, "wPokedexSeen")?;
        let pokedex_seen8 = Self::pokemon_data_address(&symbols_v8, "wPokedexSeen")?;
        save.clear_len(
            pokedex_seen8,
            Size(bits_to_bytes(NUM_UNIQUE_POKEMON_V8) as u32),
        )?;
        for i in 0..NUM_POKEMON_V7 {
            if source.read_indexed_bit(pokedex_seen7, i)? {
                let idx_v7 = (i + 1) as u16;
                let mapped = v7_to_v8_maps::map_v7_pkmn_to_v8(idx_v7);
                if mapped != INVALID_U16 {
                    save.write_indexed_bit(pokedex_seen8, (mapped - 1) as usize, true)?;
                }
            }
        }
        for &mon in &seen_mons {
            if mon != 0 {
                save.write_indexed_bit(pokedex_seen8, (mon - 1) as usize, true)?;
            }
        }

        // Player caught legendaries bitset
        let player_caught8 = Self::player_data_address(&symbols_v8, "wPlayerCaught")?;
        let player_caught2_8 = Self::player_data_address(&symbols_v8, "wPlayerCaught2")?;
        save.write_u8(player_caught8, 0)?;
        save.write_u8(player_caught2_8, 0)?;
        for (mon, addr, bit) in [
            (HO_OH_V8, player_caught8, 0u8),
            (LUGIA_V8, player_caught8, 1u8),
            (RAIKOU_V8, player_caught8, 2u8),
            (ENTEI_V8, player_caught8, 3u8),
            (SUICUNE_V8, player_caught8, 4u8),
            (ARTICUNO_V8, player_caught8, 5u8),
            (ZAPDOS_V8, player_caught8, 6u8),
            (MOLTRES_V8, player_caught8, 7u8),
            (MEW_V8, player_caught2_8, 0u8),
            (MEWTWO_V8, player_caught2_8, 1u8),
            (CELEBI_V8, player_caught2_8, 2u8),
            (SUDOWOODO_V8, player_caught2_8, 3u8),
        ] {
            if contains_u16(&caught_mons, mon) {
                save.write_bit(addr, bit, true)?;
            }
        }

        // Prevent map scripts on load
        save.write_u8(
            Self::player_data_address(&symbols_v8, "wCurMapSceneScriptCount")?,
            0,
        )?;
        save.write_u8(
            Self::player_data_address(&symbols_v8, "wCurMapCallbackCount")?,
            0,
        )?;
        save.write_u16_le(
            Self::player_data_address(&symbols_v8, "wCurMapSceneScriptPointer")?,
            0,
        )?;

        // Write v8 version (big-endian word)
        save.write_u16_be(Address(SAVE_VERSION_ABS_ADDRESS), 8)?;

        // Checksums
        let main_checksum = calculate_main_save_checksum(save, &symbols_v8)?;
        let backup_checksum = calculate_backup_save_checksum(save, &symbols_v8)?;
        write_main_save_checksum(save, main_checksum)?;
        write_backup_save_checksum(save, backup_checksum)?;

        Ok(())
    }
}
