use gb_save_core::{
    map_bitset, remap_zero_terminated_u8, Address, Patch, PatchKind, PatchLogSink, PatchMetadata,
    SaveBinary, SaveError, SaveResult, Size, SymbolDatabase,
};

use crate::{
    calculate_backup_save_checksum, calculate_main_save_checksum, get_save_version,
    symbols_for_version, validate_primary_and_backup_checksums_with_log,
    write_backup_save_checksum, write_main_save_checksum, SupportedSaveVersion,
    SAVE_VERSION_ABS_ADDRESS,
};

use super::scaffold;
use super::{addrs::PolishedAddrs, common, v8_to_v9_maps};

const PLAINBADGE: usize = 2;

const EVENT_BEAT_CANDELA: usize = 0x596;
const EVENT_BEAT_BLANCHE: usize = 0x597;
const EVENT_BEAT_SPARK: usize = 0x598;

const GOLDENROD_POKECOM_CENTER_1F: (u8, u8) = (11, 24);
const PLAYERS_HOUSE_1F: (u8, u8) = (24, 6);

#[derive(Debug)]
pub struct MigrationV8ToV9;

pub static MIGRATION_V8_TO_V9: MigrationV8ToV9 = MigrationV8ToV9;

impl Patch for MigrationV8ToV9 {
    fn metadata(&self) -> PatchMetadata {
        PatchMetadata {
            id: "polished.migration.v8_to_v9",
            kind: PatchKind::Migration,
            from_version: Some(8),
            to_version: Some(9),
        }
    }

    fn apply(&self, save: &mut SaveBinary, symbols_v8: &SymbolDatabase) -> SaveResult<()> {
        scaffold::apply_noop_log(save, symbols_v8, |save, symbols, log| {
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

impl MigrationV8ToV9 {
    fn apply_impl(
        &self,
        save: &mut SaveBinary,
        symbols_v8: &SymbolDatabase,
        log: &mut dyn PatchLogSink,
    ) -> SaveResult<()> {
        validate_primary_and_backup_checksums_with_log(save, symbols_v8, log, self.metadata().id)?;
        let source = save.clone();

        let current = get_save_version(save)?;
        if current != 8 {
            return Err(SaveError::InvalidSaveState {
                reason: format!("expected v8 save, got v{current}"),
            });
        }

        let symbols_v9 = symbols_for_version(SupportedSaveVersion::V9)?;

        let addrs_v8 = PolishedAddrs::new(symbols_v8);
        let addrs_v9 = PolishedAddrs::new(&symbols_v9);

        common::require_player_in_pokecenter_2f(&source, symbols_v8, log, self.metadata().id)?;

        save.clear_len(Address(addrs_v9.player("wRTC")?.0 + 4), Size(4))?;
        save.clear_len(Address(addrs_v8.player("wTimeOfDayPal")?.0 + 1), Size(4))?;

        let key_items_v8 = addrs_v8.player("wKeyItems")?;
        let key_items_end_v8 = addrs_v8.player("wKeyItemsEnd")?;
        let key_items_v9 = addrs_v9.player("wKeyItems")?;
        let key_items_end_v9 = addrs_v9.player("wKeyItemsEnd")?;
        save.clear_len(key_items_v9, Size(key_items_end_v9.0 - key_items_v9.0))?;
        save.copy_from_other(
            &source,
            key_items_v8,
            key_items_v9,
            Size(key_items_end_v8.0 - key_items_v8.0),
        )?;

        let key_items_len_v9 = (key_items_end_v9.0 - key_items_v9.0) as usize;
        remap_zero_terminated_u8(
            save,
            key_items_v9,
            key_items_len_v9,
            |value| {
                let mapped = v8_to_v9_maps::map_v8_key_item_to_v9(value);
                (mapped != v8_to_v9_maps::INVALID_U8).then_some(mapped)
            },
            |_, value| {
                log.error(
                    self.metadata().id,
                    &format!("Key Item {value:02x} not found in version 9 key item list."),
                );
            },
        )?;

        let num_items_v8 = addrs_v8.player("wNumItems")?;
        let moo_moo_berries_v8 = addrs_v8.player("wMooMooBerries")?;
        save.copy_from_other(
            &source,
            num_items_v8,
            addrs_v9.player("wNumItems")?,
            Size((moo_moo_berries_v8.0 - 1) - num_items_v8.0),
        )?;

        let ecruteak_house_scene_id_v8 = addrs_v8.player("wEcruteakHouseSceneID")?;
        save.copy_from_other(
            &source,
            moo_moo_berries_v8,
            addrs_v9.player("wMooMooBerries")?,
            Size((ecruteak_house_scene_id_v8.0 + 1) - moo_moo_berries_v8.0),
        )?;

        save.write_u8(addrs_v9.player("wRocketHideoutB4FSceneID")?, 0)?;

        let elms_lab_scene_id_v8 = addrs_v8.player("wElmsLabSceneID")?;
        let event_flags_v8 = addrs_v8.player("wEventFlags")?;
        save.copy_from_other(
            &source,
            elms_lab_scene_id_v8,
            addrs_v9.player("wElmsLabSceneID")?,
            Size(event_flags_v8.0 - elms_lab_scene_id_v8.0),
        )?;

        let event_flags_v9 = addrs_v9.player("wEventFlags")?;
        let cur_box_v9 = addrs_v9.player("wCurBox")?;
        save.clear_len(event_flags_v9, Size(cur_box_v9.0 - event_flags_v9.0))?;

        map_bitset(
            &source,
            event_flags_v8,
            v8_to_v9_maps::NUM_EVENTS,
            save,
            event_flags_v9,
            v8_to_v9_maps::NUM_EVENTS,
            |i| {
                let mapped = v8_to_v9_maps::map_v8_event_flag_to_v9(i as u16);
                (mapped != v8_to_v9_maps::INVALID_U16).then_some(mapped as usize)
            },
            |i| {
                log.warn(
                    self.metadata().id,
                    &format!("Event Flag {i} not found in version 8 event flag list."),
                );
            },
        )?;

        let cur_box_v8 = addrs_v8.player("wCurBox")?;
        let emote_pal_v8 = addrs_v8.player("wEmotePal")?;
        save.copy_from_other(
            &source,
            cur_box_v8,
            addrs_v9.player("wCurBox")?,
            Size((emote_pal_v8.0 + 1) - cur_box_v8.0),
        )?;

        save.clear_len(Address(addrs_v9.player("wEmotePal")?.0 + 1), Size(69))?;

        let wing_amounts_v8 = addrs_v8.player("wWingAmounts")?;
        let hidden_grotto_contents_v8 = addrs_v8.player("wHiddenGrottoContents")?;
        save.copy_from_other(
            &source,
            wing_amounts_v8,
            addrs_v9.player("wWingAmounts")?,
            Size(hidden_grotto_contents_v8.0 - wing_amounts_v8.0),
        )?;

        let hidden_grotto_contents_v9 = addrs_v9.player("wHiddenGrottoContents")?;
        save.clear_len(Address(hidden_grotto_contents_v9.0 - 19), Size(19))?;

        let phone_list_end_v8 = addrs_v8.player("wPhoneListEnd")?;
        save.copy_from_other(
            &source,
            hidden_grotto_contents_v8,
            hidden_grotto_contents_v9,
            Size(phone_list_end_v8.0 - hidden_grotto_contents_v8.0),
        )?;

        save.write_u8(addrs_v9.player("wPhoneListEnd")?, 0)?;

        let park_balls_remaining_v8 = addrs_v8.player("wParkBallsRemaining")?;
        let player_data_end_v8 = addrs_v8.player("wPlayerDataEnd")?;
        save.copy_from_other(
            &source,
            park_balls_remaining_v8,
            addrs_v9.player("wParkBallsRemaining")?,
            Size(player_data_end_v8.0 - park_balls_remaining_v8.0),
        )?;

        let cur_map_data_v8 = addrs_v8.map("wCurMapData")?;
        let cur_map_data_end_v8 = addrs_v8.map("wCurMapDataEnd")?;
        save.copy_from_other(
            &source,
            cur_map_data_v8,
            addrs_v9.map("wCurMapData")?,
            Size(cur_map_data_end_v8.0 - cur_map_data_v8.0),
        )?;

        let pokemon_data_v8 = addrs_v8.pokemon("wPokemonData")?;
        let party_count_v8 = addrs_v8.pokemon("wPartyCount")?;
        save.copy_from_other(
            &source,
            pokemon_data_v8,
            addrs_v9.pokemon("wPokemonData")?,
            Size((party_count_v8.0 + 1) - pokemon_data_v8.0),
        )?;

        save.clear_len(Address(addrs_v9.pokemon("wPartyCount")?.0 + 1), Size(7))?;

        let pokedex_caught_v9 = addrs_v9.pokemon("wPokedexCaught")?;
        let unlocked_unowns_v9 = addrs_v9.pokemon("wUnlockedUnowns")?;
        save.clear_len(
            pokedex_caught_v9,
            Size(unlocked_unowns_v9.0 - pokedex_caught_v9.0),
        )?;

        let party_mons_v8 = addrs_v8.pokemon("wPartyMons")?;
        let end_pokedex_caught_v8 = addrs_v8.pokemon("wEndPokedexCaught")?;
        save.copy_from_other(
            &source,
            party_mons_v8,
            addrs_v9.pokemon("wPartyMons")?,
            Size(end_pokedex_caught_v8.0 - party_mons_v8.0),
        )?;

        let pokedex_seen_v8 = addrs_v8.pokemon("wPokedexSeen")?;
        let end_pokedex_seen_v8 = addrs_v8.pokemon("wEndPokedexSeen")?;
        save.copy_from_other(
            &source,
            pokedex_seen_v8,
            addrs_v9.pokemon("wPokedexSeen")?,
            Size(end_pokedex_seen_v8.0 - pokedex_seen_v8.0),
        )?;

        let unlocked_unowns_v8 = addrs_v8.pokemon("wUnlockedUnowns")?;
        save.write_u8(unlocked_unowns_v9, source.read_u8(unlocked_unowns_v8)?)?;

        save.clear_len(Address(unlocked_unowns_v9.0 + 1), Size(2))?;

        let day_care_man_v8 = addrs_v8.pokemon("wDayCareMan")?;
        let best_magikarp_len_v8 = addrs_v8.pokemon("wBestMagikarpLengthMm")?;
        save.copy_from_other(
            &source,
            day_care_man_v8,
            addrs_v9.pokemon("wDayCareMan")?,
            Size(best_magikarp_len_v8.0 - day_care_man_v8.0),
        )?;

        let best_magikarp_len_v9 = addrs_v9.pokemon("wBestMagikarpLengthMm")?;
        save.clear_len(Address(best_magikarp_len_v9.0 - 4), Size(4))?;

        let pokemon_data_end_v8 = addrs_v8.pokemon("wPokemonDataEnd")?;
        save.copy_from_other(
            &source,
            best_magikarp_len_v8,
            best_magikarp_len_v9,
            Size(pokemon_data_end_v8.0 - best_magikarp_len_v8.0),
        )?;

        save.clear_len(Address(hidden_grotto_contents_v8.0 - 19), Size(19))?;

        save.write_indexed_bit(event_flags_v9, EVENT_BEAT_CANDELA, false)?;
        save.write_indexed_bit(event_flags_v9, EVENT_BEAT_BLANCHE, false)?;
        save.write_indexed_bit(event_flags_v9, EVENT_BEAT_SPARK, false)?;

        save.write_u8(addrs_v9.player("wCurMapSceneScriptCount")?, 0)?;
        save.write_u8(addrs_v9.player("wCurMapCallbackCount")?, 0)?;
        save.write_u16_le(addrs_v9.player("wCurMapSceneScriptPointer")?, 0)?;

        common::reset_invalid_prev_map_warp(
            save,
            &symbols_v9,
            log,
            self.metadata().id,
            PLAINBADGE,
            GOLDENROD_POKECOM_CENTER_1F,
            PLAYERS_HOUSE_1F,
        )?;

        save.write_u16_be(Address(SAVE_VERSION_ABS_ADDRESS), 9)?;

        // The golden fixtures expect the backup slot to retain the pre-migration game data.
        // Populate v9's sBackupGameData with the original v8 sGameData bytes.
        let src_game_start = symbols_v8.sram_absolute_address("sGameData")?;
        let src_game_end = symbols_v8.sram_absolute_address("sGameDataEnd")?;
        let backup_start = symbols_v9.sram_absolute_address("sBackupGameData")?;
        save.copy_from_other(
            &source,
            src_game_start,
            backup_start,
            Size(src_game_end.0 - src_game_start.0),
        )?;

        let main_checksum = calculate_main_save_checksum(save, &symbols_v9)?;
        let backup_checksum = calculate_backup_save_checksum(save, &symbols_v9)?;
        write_main_save_checksum(save, main_checksum)?;
        write_backup_save_checksum(save, backup_checksum)?;

        Ok(())
    }
}
