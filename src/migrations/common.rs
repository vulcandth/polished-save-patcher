use gb_save_core::{Address, PatchLogSink, SaveBinary, SaveError, SaveResult, SymbolDatabase};

pub(super) const MON_CENTER_2F: (u8, u8) = (20, 1);

pub(super) const VALID_PC_WARP_IDS: &[(u8, u8)] = &[
    (1, 1),   // OLIVINE_POKECENTER_1F
    (2, 3),   // MAHOGANY_POKECENTER_1F
    (4, 3),   // ECRUTEAK_POKECENTER_1F
    (5, 6),   // BLACKTHORN_POKECENTER_1F
    (6, 1),   // CINNABAR_POKECENTER_1F
    (7, 4),   // CERULEAN_POKECENTER_1F
    (7, 8),   // ROUTE_10_POKECENTER_1F
    (8, 1),   // AZALEA_POKECENTER_1F
    (10, 8),  // VIOLET_POKECENTER_1F
    (10, 11), // ROUTE_32_POKECENTER_1F
    (11, 24), // GOLDENROD_POKECOM_CENTER_1F
    (12, 5),  // VERMILION_POKECENTER_1F
    (14, 3),  // ROUTE_3_POKECENTER_1F
    (14, 8),  // PEWTER_POKECENTER_1F
    (16, 3),  // INDIGO_PLATEAU_POKECENTER_1F
    (17, 12), // FUCHSIA_POKECENTER_1F
    (18, 6),  // LAVENDER_POKECENTER_1F
    (19, 3),  // SILVER_CAVE_POKECENTER_1F
    (21, 20), // CELADON_POKECENTER_1F
    (22, 5),  // CIANWOOD_POKECENTER_1F
    (23, 11), // VIRIDIAN_POKECENTER_1F
    (25, 4),  // SAFFRON_POKECENTER_1F
    (26, 6),  // CHERRYGROVE_POKECENTER_1F
    (31, 8),  // SHAMOUTI_POKECENTER_1F
    (36, 5),  // SNOWTOP_POKECENTER_1F
];

pub(super) fn require_player_in_pokecenter_2f(
    source: &SaveBinary,
    symbols: &SymbolDatabase,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> SaveResult<()> {
    let map_group_addr =
        symbols.wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wMapGroup")?;
    let map_num_addr =
        symbols.wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wMapNumber")?;

    let map_group = source.read_u8(map_group_addr)?;
    let map_num = source.read_u8(map_num_addr)?;

    if map_group != MON_CENTER_2F.0 || map_num != MON_CENTER_2F.1 {
        let msg = "Player is not in the PKMN Center 2nd Floor. Go to where you heal in game, and head upstairs. Then re-save your game and try again.";
        log.error(log_source, msg);
        return Err(SaveError::InvalidSaveState {
            reason: msg.to_string(),
        });
    }

    Ok(())
}

pub(super) fn reset_invalid_prev_map_warp(
    save: &mut SaveBinary,
    symbols: &SymbolDatabase,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
    plainbadge_bit_index: usize,
    goldenrod_pokecenter_1f: (u8, u8),
    players_house_1f: (u8, u8),
) -> SaveResult<()> {
    let prev_map_group = save.read_u8(symbols.wram_relative_to_sram_absolute_address(
        "wCurMapData",
        "sMapData",
        "wBackupMapGroup",
    )?)?;
    let prev_map_num = save.read_u8(symbols.wram_relative_to_sram_absolute_address(
        "wCurMapData",
        "sMapData",
        "wBackupMapNumber",
    )?)?;

    let valid_prev_map = VALID_PC_WARP_IDS
        .iter()
        .any(|&(g, n)| g == prev_map_group && n == prev_map_num);
    if valid_prev_map {
        return Ok(());
    }

    log.warn(
        log_source,
        "Player's previous map is not a valid PKMN Center Warp ID! We will reset it to one.",
    );

    let johto_badges = symbols.wram_relative_to_sram_absolute_address(
        "wPlayerData",
        "sPlayerData",
        "wJohtoBadges",
    )?;
    let has_plainbadge = save.read_indexed_bit(johto_badges, plainbadge_bit_index)?;

    let warp_addr = symbols.wram_relative_to_sram_absolute_address(
        "wCurMapData",
        "sMapData",
        "wBackupWarpNumber",
    )?;
    let group_addr = symbols.wram_relative_to_sram_absolute_address(
        "wCurMapData",
        "sMapData",
        "wBackupMapGroup",
    )?;
    let num_addr = symbols.wram_relative_to_sram_absolute_address(
        "wCurMapData",
        "sMapData",
        "wBackupMapNumber",
    )?;

    if has_plainbadge {
        log.warn(
            log_source,
            "Player has the PLAINBADGE, the stairs will now take you to Goldenrod PKMN Center.",
        );
        save.write_u8(warp_addr, 4)?;
        save.write_u8(group_addr, goldenrod_pokecenter_1f.0)?;
        save.write_u8(num_addr, goldenrod_pokecenter_1f.1)?;
    } else {
        log.warn(
            log_source,
            "Player does not have the PLAINBADGE, the stairs will now warp you to your house.",
        );
        save.write_u8(warp_addr, 3)?;
        save.write_u8(group_addr, players_house_1f.0)?;
        save.write_u8(num_addr, players_house_1f.1)?;
    }

    Ok(())
}

pub(super) fn try_player_data_address(
    symbols: &SymbolDatabase,
    wram_symbol: &str,
) -> SaveResult<Option<Address>> {
    match symbols.wram_relative_to_sram_absolute_address("wPlayerData", "sPlayerData", wram_symbol)
    {
        Ok(addr) => Ok(Some(addr)),
        Err(SaveError::SymbolNotFound { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use gb_save_core::{PatchLogLevel, VecPatchLogSink};

    use crate::{symbols_for_version, SupportedSaveVersion, MIN_SAVE_SIZE};

    #[test]
    fn require_player_in_pokecenter_2f_errors_and_logs() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let map_group_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wMapGroup")
            .unwrap();
        let map_num_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wMapNumber")
            .unwrap();

        save.write_u8(map_group_addr, MON_CENTER_2F.0.wrapping_add(1))
            .unwrap();
        save.write_u8(map_num_addr, MON_CENTER_2F.1).unwrap();

        let mut log = VecPatchLogSink::new();
        let err = require_player_in_pokecenter_2f(&save, &symbols, &mut log, "test").unwrap_err();
        match err {
            SaveError::InvalidSaveState { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }

        let entries = log.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, PatchLogLevel::Error);
        assert_eq!(entries[0].source, "test");
        assert!(entries[0].message.contains("PKMN Center 2nd Floor"));
    }

    #[test]
    fn reset_invalid_prev_map_warp_sets_house_when_no_plainbadge() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let warp_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupWarpNumber")
            .unwrap();
        let group_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupMapGroup")
            .unwrap();
        let num_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupMapNumber")
            .unwrap();

        save.write_u8(group_addr, 0).unwrap();
        save.write_u8(num_addr, 0).unwrap();
        save.write_u8(warp_addr, 0).unwrap();

        let mut log = VecPatchLogSink::new();
        reset_invalid_prev_map_warp(&mut save, &symbols, &mut log, "test", 2, (11, 24), (9, 9))
            .unwrap();

        assert_eq!(save.read_u8(warp_addr).unwrap(), 3);
        assert_eq!(save.read_u8(group_addr).unwrap(), 9);
        assert_eq!(save.read_u8(num_addr).unwrap(), 9);

        let entries = log.into_entries();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.level == PatchLogLevel::Warning));
        assert!(entries[0]
            .message
            .contains("previous map is not a valid PKMN Center Warp ID"));
        assert!(entries[1].message.contains("does not have the PLAINBADGE"));
    }

    #[test]
    fn reset_invalid_prev_map_warp_sets_goldenrod_when_has_plainbadge() {
        let symbols = symbols_for_version(SupportedSaveVersion::V9).unwrap();
        let mut save = SaveBinary::new(vec![0u8; MIN_SAVE_SIZE]);

        let warp_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupWarpNumber")
            .unwrap();
        let group_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupMapGroup")
            .unwrap();
        let num_addr = symbols
            .wram_relative_to_sram_absolute_address("wCurMapData", "sMapData", "wBackupMapNumber")
            .unwrap();

        save.write_u8(group_addr, 0).unwrap();
        save.write_u8(num_addr, 0).unwrap();
        save.write_u8(warp_addr, 0).unwrap();

        let johto_badges = symbols
            .wram_relative_to_sram_absolute_address("wPlayerData", "sPlayerData", "wJohtoBadges")
            .unwrap();
        save.write_indexed_bit(johto_badges, 2, true).unwrap();

        let mut log = VecPatchLogSink::new();
        reset_invalid_prev_map_warp(&mut save, &symbols, &mut log, "test", 2, (11, 24), (9, 9))
            .unwrap();

        assert_eq!(save.read_u8(warp_addr).unwrap(), 4);
        assert_eq!(save.read_u8(group_addr).unwrap(), 11);
        assert_eq!(save.read_u8(num_addr).unwrap(), 24);

        let entries = log.into_entries();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.level == PatchLogLevel::Warning));
        assert!(entries[0]
            .message
            .contains("previous map is not a valid PKMN Center Warp ID"));
        assert!(entries[1].message.contains("has the PLAINBADGE"));
    }
}
