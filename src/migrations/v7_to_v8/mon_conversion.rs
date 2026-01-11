use gb_save_core::PatchLogSink;

use super::super::structs_v8::{
    BreedMonV8, HofMonV8, MailMsgV8, PartyMonV8, RoamMonV8, SaveMonV8, BREEDMON_LEN,
    CAUGHT_GENDER_MASK, HOFMON_LEN, MAILMSG_LEN, PARTYMON_LEN, ROAMMON_LEN, SAVEMON_LEN,
};
use super::super::v7_to_v8_maps;
use super::{
    FLY_V7, GYARADOS_RED_FORM_V7, GYARADOS_RED_FORM_V8, GYARADOS_V8, INVALID_U16, MAGIKARP_V8,
    PIKACHU_FLY_FORM_V7, PIKACHU_SURF_FORM_V7, PIKACHU_V8, SURF_V7,
};

fn map_u8_or_default(
    value: u8,
    invalid: u8,
    default: u8,
    map: impl FnOnce(u8) -> u8,
    mut on_invalid: impl FnMut(u8),
) -> u8 {
    let mapped = map(value);
    if mapped == invalid {
        on_invalid(value);
        default
    } else {
        mapped
    }
}

fn convert_savemon_v7_to_v8_struct(
    savemon: &SaveMonV8,
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> SaveMonV8 {
    let v7_species = savemon.species as u16;
    let species_v8 = v7_to_v8_maps::map_v7_pkmn_to_v8(v7_species);
    if species_v8 == INVALID_U16 {
        log.error(
            log_source,
            &format!("Savemon species {v7_species:02x} not found in version 8 mon list.",),
        );
        return savemon.clone();
    }

    let mut out = savemon.clone();

    out.item = map_u8_or_default(
        savemon.item,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_item_to_v8,
        |value| {
            log.error(
                log_source,
                &format!(
                    "Savemon item {:02x} not found in version 8 item list.",
                    value
                ),
            );
        },
    );

    out.set_ext_species(species_v8);
    let v7_form = savemon.form();
    if v7_form == 0 {
        out.set_form(1);
    } else {
        out.set_form(v7_form);
    }

    let mut extspecies_form = INVALID_U16;
    if species_v8 == PIKACHU_V8 {
        for &mv in &savemon.moves_ {
            if mv == SURF_V7 {
                out.set_form(PIKACHU_SURF_FORM_V7);
                extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(
                    v7_species,
                    PIKACHU_SURF_FORM_V7,
                );
                break;
            }
            if mv == FLY_V7 {
                out.set_form(PIKACHU_FLY_FORM_V7);
                extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(
                    v7_species,
                    PIKACHU_FLY_FORM_V7,
                );
                break;
            }
        }
    } else {
        extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(v7_species, v7_form);
    }

    if extspecies_form != INVALID_U16 {
        seen.push(extspecies_form);
        caught.push(extspecies_form);
    } else {
        seen.push(species_v8);
        caught.push(species_v8);
    }

    if species_v8 == MAGIKARP_V8 {
        let mapped_form = v7_to_v8_maps::map_v7_magikarp_form_to_v8(v7_form);
        if mapped_form == v7_to_v8_maps::INVALID_U8 {
            log.warn(
                log_source,
                &format!("Magikarp form {v7_form:02x} not found in version 8 form list.",),
            );
        } else {
            out.set_form(mapped_form);
        }
    }

    if species_v8 == GYARADOS_V8 && v7_form == GYARADOS_RED_FORM_V7 {
        out.set_form(GYARADOS_RED_FORM_V8);
    }

    let time = savemon.caught_time();
    let ball_v7 = savemon.caught_ball();
    out.caughtdata = savemon.caughtdata & !CAUGHT_GENDER_MASK;
    out.set_caught_time(time);

    out.set_caught_ball(map_u8_or_default(
        ball_v7,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_item_to_v8,
        |value| {
            log.error(
                log_source,
                &format!("Savemon ball {value:02x} not found in version 8 item list.",),
            );
        },
    ));

    out.caughtlocation = map_u8_or_default(
        savemon.caughtlocation,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_landmark_to_v8,
        |value| {
            log.error(
                log_source,
                &format!(
                    "Savemon landmark {:02x} not found in version 8 landmark list.",
                    value
                ),
            );
        },
    );

    out
}

pub(super) fn convert_savemon_v7_to_v8_bytes(
    bytes: &[u8; SAVEMON_LEN],
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; SAVEMON_LEN] {
    let savemon = SaveMonV8::from_bytes(bytes);
    convert_savemon_v7_to_v8_struct(&savemon, seen, caught, log, log_source).to_bytes()
}

fn convert_breedmon_v7_to_v8_struct(
    breedmon: &BreedMonV8,
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> BreedMonV8 {
    let v7_species = breedmon.species as u16;
    let species_v8 = v7_to_v8_maps::map_v7_pkmn_to_v8(v7_species);
    if species_v8 == INVALID_U16 {
        log.error(
            log_source,
            &format!("Breedmon species {v7_species:02x} not found in version 8 mon list.",),
        );
        return breedmon.clone();
    }

    let mut out = breedmon.clone();

    out.item = map_u8_or_default(
        breedmon.item,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_item_to_v8,
        |value| {
            log.error(
                log_source,
                &format!(
                    "Breedmon item {:02x} not found in version 8 item list.",
                    value
                ),
            );
        },
    );

    out.set_ext_species(species_v8);
    let v7_form = breedmon.form();
    if v7_form == 0 {
        out.set_form(1);
    } else {
        out.set_form(v7_form);
    }

    let mut extspecies_form = INVALID_U16;
    if species_v8 == PIKACHU_V8 {
        for &mv in &breedmon.moves_ {
            if mv == SURF_V7 {
                out.set_form(PIKACHU_SURF_FORM_V7);
                extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(
                    v7_species,
                    PIKACHU_SURF_FORM_V7,
                );
                break;
            }
            if mv == FLY_V7 {
                out.set_form(PIKACHU_FLY_FORM_V7);
                extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(
                    v7_species,
                    PIKACHU_FLY_FORM_V7,
                );
                break;
            }
        }
    } else {
        extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(v7_species, v7_form);
    }

    if extspecies_form != INVALID_U16 {
        seen.push(extspecies_form);
        caught.push(extspecies_form);
    }

    if species_v8 == MAGIKARP_V8 {
        let mapped_form = v7_to_v8_maps::map_v7_magikarp_form_to_v8(v7_form);
        if mapped_form == v7_to_v8_maps::INVALID_U8 {
            log.warn(
                log_source,
                &format!("Magikarp form {v7_form:02x} not found in version 8 form list.",),
            );
        } else {
            out.set_form(mapped_form);
        }
    }

    if species_v8 == GYARADOS_V8 && v7_form == GYARADOS_RED_FORM_V7 {
        out.set_form(GYARADOS_RED_FORM_V8);
    }

    let time = breedmon.caught_time();
    let ball_v7 = breedmon.caught_ball();
    out.caughtdata = breedmon.caughtdata & !CAUGHT_GENDER_MASK;
    out.set_caught_time(time);
    out.set_caught_ball(map_u8_or_default(
        ball_v7,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_item_to_v8,
        |value| {
            log.error(
                log_source,
                &format!("Breedmon ball {value:02x} not found in version 8 item list.",),
            );
        },
    ));

    out.caughtlocation = map_u8_or_default(
        breedmon.caughtlocation,
        v7_to_v8_maps::INVALID_U8,
        0,
        v7_to_v8_maps::map_v7_landmark_to_v8,
        |value| {
            log.error(
                log_source,
                &format!(
                    "Breedmon landmark {:02x} not found in version 8 landmark list.",
                    value
                ),
            );
        },
    );

    out
}

pub(super) fn convert_breedmon_v7_to_v8_bytes(
    bytes: &[u8; BREEDMON_LEN],
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; BREEDMON_LEN] {
    let breedmon = BreedMonV8::from_bytes(bytes);
    convert_breedmon_v7_to_v8_struct(&breedmon, seen, caught, log, log_source).to_bytes()
}

pub(super) fn convert_party_v7_to_v8_bytes(
    bytes: &[u8; PARTYMON_LEN],
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; PARTYMON_LEN] {
    let party = PartyMonV8::from_bytes(bytes);
    let mut out = party.clone();
    out.breedmon = convert_breedmon_v7_to_v8_struct(&party.breedmon, seen, caught, log, log_source);
    out.to_bytes()
}

fn convert_hofmon_v7_to_v8_struct(
    hofmon: &HofMonV8,
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> HofMonV8 {
    let v7_species = hofmon.species as u16;
    let species_v8 = v7_to_v8_maps::map_v7_pkmn_to_v8(v7_species);
    if species_v8 == INVALID_U16 {
        log.error(
            log_source,
            &format!("Hofmon species {v7_species:02x} not found in version 8 mon list.",),
        );
        return hofmon.clone();
    }

    let mut out = hofmon.clone();
    out.set_ext_species(species_v8);

    let v7_form = hofmon.form();
    out.set_form(v7_form);

    let extspecies_form = v7_to_v8_maps::map_v7_species_form_to_v8_extspecies(v7_species, v7_form);
    if extspecies_form != INVALID_U16 {
        seen.push(extspecies_form);
        caught.push(extspecies_form);
    }

    if species_v8 == MAGIKARP_V8 {
        let mapped_form = v7_to_v8_maps::map_v7_magikarp_form_to_v8(v7_form);
        if mapped_form == v7_to_v8_maps::INVALID_U8 {
            log.warn(
                log_source,
                &format!("Magikarp form {v7_form:02x} not found in version 8 form list.",),
            );
        } else {
            out.set_form(mapped_form);
        }
    }
    if species_v8 == GYARADOS_V8 && v7_form == GYARADOS_RED_FORM_V7 {
        out.set_form(GYARADOS_RED_FORM_V8);
    }

    out
}

pub(super) fn convert_hofmon_v7_to_v8_bytes(
    bytes: &[u8; HOFMON_LEN],
    seen: &mut Vec<u16>,
    caught: &mut Vec<u16>,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; HOFMON_LEN] {
    let hofmon = HofMonV8::from_bytes(bytes);
    convert_hofmon_v7_to_v8_struct(&hofmon, seen, caught, log, log_source).to_bytes()
}

fn convert_roam_v7_to_v8_struct(
    roam: &RoamMonV8,
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> RoamMonV8 {
    let v7_species = roam.species as u16;
    let species_v8 = v7_to_v8_maps::map_v7_pkmn_to_v8(v7_species);
    if species_v8 == INVALID_U16 {
        log.error(
            log_source,
            &format!("Roam species {v7_species:02x} not found in version 8 mon list.",),
        );
        return roam.clone();
    }

    let mut out = roam.clone();
    out.set_ext_species(species_v8);

    let (g, m) = v7_to_v8_maps::map_v7_map_to_v8(roam.map_group, roam.map_number);
    if g == 0 && m == 0 {
        out.set_map((0xFF, 0xFF));
    } else {
        out.set_map((g, m));
    }

    if roam.form() == 0 {
        out.set_form(1);
    }

    out
}

pub(super) fn convert_roam_v7_to_v8_bytes(
    bytes: &[u8; ROAMMON_LEN],
    log: &mut dyn PatchLogSink,
    log_source: &'static str,
) -> [u8; ROAMMON_LEN] {
    let roam = RoamMonV8::from_bytes(bytes);
    convert_roam_v7_to_v8_struct(&roam, log, log_source).to_bytes()
}

pub(super) fn convert_mailmsg_v7_to_v8_bytes(bytes: &[u8; MAILMSG_LEN]) -> [u8; MAILMSG_LEN] {
    let mut msg = MailMsgV8::from_bytes(bytes);
    for b in &mut msg.message {
        *b = v7_to_v8_maps::map_v7_char_to_v8(*b);
    }
    msg.to_bytes()
}

#[cfg(test)]
mod tests {
    use gb_save_core::{PatchLogLevel, VecPatchLogSink};

    use super::*;

    fn find_v7_species_mapping_to(target_v8: u16) -> u8 {
        (0u16..=u8::MAX as u16)
            .find(|&s| v7_to_v8_maps::map_v7_pkmn_to_v8(s) == target_v8)
            .expect("expected v7 species mapping to requested v8 species") as u8
    }

    fn find_v7_species_mapping_to_any_valid() -> u8 {
        (0u16..=u8::MAX as u16)
            .find(|&s| v7_to_v8_maps::map_v7_pkmn_to_v8(s) != INVALID_U16)
            .expect("expected at least one mapped v7 species") as u8
    }

    fn find_v7_species_mapping_to_regular_valid() -> u8 {
        (0u16..=u8::MAX as u16)
            .find(|&s| {
                let mapped = v7_to_v8_maps::map_v7_pkmn_to_v8(s);
                mapped != INVALID_U16
                    && mapped != PIKACHU_V8
                    && mapped != MAGIKARP_V8
                    && mapped != GYARADOS_V8
            })
            .expect("expected at least one mapped v7 species that is not special-cased")
            as u8
    }

    fn find_valid_item_v7() -> u8 {
        (0u8..=u8::MAX)
            .find(|&v| v7_to_v8_maps::map_v7_item_to_v8(v) != v7_to_v8_maps::INVALID_U8)
            .expect("expected at least one valid v7 item")
    }

    fn find_invalid_item_v7() -> u8 {
        (0u8..=u8::MAX)
            .find(|&v| v7_to_v8_maps::map_v7_item_to_v8(v) == v7_to_v8_maps::INVALID_U8)
            .expect("expected at least one invalid v7 item")
    }

    fn find_valid_landmark_v7() -> u8 {
        (0u8..=u8::MAX)
            .find(|&v| v7_to_v8_maps::map_v7_landmark_to_v8(v) != v7_to_v8_maps::INVALID_U8)
            .expect("expected at least one valid v7 landmark")
    }

    #[test]
    fn savemon_unmapped_species_logs_exact_message() {
        let v7_species = (0u16..=u8::MAX as u16)
            .find(|&s| v7_to_v8_maps::map_v7_pkmn_to_v8(s) == INVALID_U16)
            .expect("expected at least one unmapped v7 species");

        let savemon = SaveMonV8 {
            species: v7_species as u8,
            ..Default::default()
        };

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let _ = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );

        let entries = log.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, PatchLogLevel::Error);
        assert_eq!(entries[0].source, "v7_to_v8");
        assert_eq!(
            entries[0].message,
            format!("Savemon species {v7_species:02x} not found in version 8 mon list.",)
        );
    }

    #[test]
    fn savemon_form_zero_becomes_one_for_regular_species() {
        let v7_species = find_v7_species_mapping_to_regular_valid();
        let item = find_valid_item_v7();
        let landmark = find_valid_landmark_v7();

        let mut savemon = SaveMonV8 {
            species: v7_species,
            item,
            caughtlocation: landmark,
            ..Default::default()
        };
        savemon.set_form(0);
        savemon.set_caught_ball(item);
        savemon.set_caught_time(3);

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let bytes = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );
        let decoded = SaveMonV8::from_bytes(&bytes);
        assert_eq!(decoded.form(), 1);
        assert_eq!(seen.len(), 1);
        assert_eq!(caught.len(), 1);
        assert!(log.into_entries().is_empty());
    }

    #[test]
    fn savemon_invalid_item_logs_error_and_defaults_to_zero() {
        let v7_species = find_v7_species_mapping_to_any_valid();
        let invalid_item = find_invalid_item_v7();
        let valid_item = find_valid_item_v7();
        let landmark = find_valid_landmark_v7();

        let mut savemon = SaveMonV8 {
            species: v7_species,
            item: invalid_item,
            caughtlocation: landmark,
            ..Default::default()
        };
        savemon.set_caught_ball(valid_item);

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let bytes = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );
        let decoded = SaveMonV8::from_bytes(&bytes);
        assert_eq!(decoded.item, 0);

        let entries = log.into_entries();
        assert!(entries.iter().any(|e| {
            e.level == PatchLogLevel::Error
                && e.source == "v7_to_v8"
                && e.message.contains("Savemon item")
        }));
    }

    #[test]
    fn savemon_pikachu_with_surf_sets_surf_form() {
        let v7_species = find_v7_species_mapping_to(PIKACHU_V8);
        let item = find_valid_item_v7();
        let landmark = find_valid_landmark_v7();

        let mut savemon = SaveMonV8 {
            species: v7_species,
            item,
            moves_: [SURF_V7, 0, 0, 0],
            caughtlocation: landmark,
            ..Default::default()
        };
        savemon.set_caught_ball(item);

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let bytes = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );
        let decoded = SaveMonV8::from_bytes(&bytes);
        assert_eq!(decoded.form(), PIKACHU_SURF_FORM_V7);
    }

    #[test]
    fn savemon_gyarados_red_form_maps_to_v8_red_form() {
        let v7_species = find_v7_species_mapping_to(GYARADOS_V8);
        let item = find_valid_item_v7();
        let landmark = find_valid_landmark_v7();

        let mut savemon = SaveMonV8 {
            species: v7_species,
            item,
            caughtlocation: landmark,
            ..Default::default()
        };
        savemon.set_form(GYARADOS_RED_FORM_V7);
        savemon.set_caught_ball(item);

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let bytes = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );
        let decoded = SaveMonV8::from_bytes(&bytes);
        assert_eq!(decoded.form(), GYARADOS_RED_FORM_V8);
    }

    #[test]
    fn savemon_magikarp_invalid_form_emits_warning() {
        let v7_species = find_v7_species_mapping_to(MAGIKARP_V8);
        let item = find_valid_item_v7();
        let landmark = find_valid_landmark_v7();
        let invalid_form = (0u8..=u8::MAX)
            .find(|&v| v7_to_v8_maps::map_v7_magikarp_form_to_v8(v) == v7_to_v8_maps::INVALID_U8)
            .expect("expected at least one invalid magikarp form");

        let mut savemon = SaveMonV8 {
            species: v7_species,
            item,
            caughtlocation: landmark,
            ..Default::default()
        };
        savemon.set_form(invalid_form);
        savemon.set_caught_ball(item);

        let mut log = VecPatchLogSink::new();
        let mut seen = Vec::new();
        let mut caught = Vec::new();

        let _ = convert_savemon_v7_to_v8_bytes(
            &savemon.to_bytes(),
            &mut seen,
            &mut caught,
            &mut log,
            "v7_to_v8",
        );

        let entries = log.into_entries();
        assert!(entries.iter().any(|e| {
            e.level == PatchLogLevel::Warning
                && e.source == "v7_to_v8"
                && e.message.contains("Magikarp form")
        }));
    }

    #[test]
    fn mailmsg_character_mapping_applies_to_message_bytes() {
        let input = 0x55;
        let mut bytes = [0u8; MAILMSG_LEN];
        bytes[0] = input;

        let out = convert_mailmsg_v7_to_v8_bytes(&bytes);
        assert_eq!(out[0], v7_to_v8_maps::map_v7_char_to_v8(input));
    }
}
