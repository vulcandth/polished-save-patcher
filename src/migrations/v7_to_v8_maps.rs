// Mapping tables for the v7→v8 migration.
//
// The human-maintainable mapping entries are vendored in
// `v7_to_v8_maps_data.rs` and preserve upstream comments and formatting.

#![allow(clippy::unreadable_literal)]

pub const INVALID_U8: u8 = 0xFF;
pub const INVALID_U16: u16 = 0xFFFF;

// Keep this value in sync with the upstream save format.
pub const NUM_EVENTS: usize = 0x8FF;

include!("v7_to_v8_maps_data.rs");

const fn build_u8_table(pairs: &[(u8, u8)], default: u8) -> [u8; 256] {
    let mut out = [default; 256];
    let mut i = 0usize;
    while i < pairs.len() {
        let (k, v) = pairs[i];
        out[k as usize] = v;
        i += 1;
    }
    out
}

const fn build_u8_identity_table(pairs: &[(u8, u8)]) -> [u8; 256] {
    let mut out = [0u8; 256];
    let mut i = 0usize;
    while i < 256 {
        out[i] = i as u8;
        i += 1;
    }

    let mut j = 0usize;
    while j < pairs.len() {
        let (k, v) = pairs[j];
        out[k as usize] = v;
        j += 1;
    }
    out
}

const fn build_u16_table<const N: usize>(pairs: &[(u16, u16)], default: u16) -> [u16; N] {
    let mut out = [default; N];
    let mut i = 0usize;
    while i < pairs.len() {
        let (k, v) = pairs[i];
        let idx = k as usize;
        if idx < N {
            out[idx] = v;
        }
        i += 1;
    }
    out
}

const fn build_u16_from_u8_key_table(pairs: &[(u8, u16)], default: u16) -> [u16; 256] {
    let mut out = [default; 256];
    let mut i = 0usize;
    while i < pairs.len() {
        let (k, v) = pairs[i];
        out[k as usize] = v;
        i += 1;
    }
    out
}

pub const KEY_ITEM_MAP: [u8; 256] = build_u8_table(KEY_ITEM_ENTRIES, INVALID_U8);
pub const ITEM_MAP: [u8; 256] = build_u8_table(ITEM_ENTRIES, INVALID_U8);
pub const LANDMARK_MAP: [u8; 256] = build_u8_table(LANDMARK_ENTRIES, INVALID_U8);
pub const SPAWN_MAP: [u8; 256] = build_u8_table(SPAWN_ENTRIES, INVALID_U8);
pub const MAGIKARP_FORM_MAP: [u8; 256] = build_u8_table(MAGIKARP_FORM_ENTRIES, INVALID_U8);
pub const THEME_MAP: [u8; 256] = build_u8_table(THEME_ENTRIES, INVALID_U8);

pub const CHAR_MAP: [u8; 256] = build_u8_identity_table(CHAR_ENTRIES);

pub const PKMN_MAP: [u16; 256] = build_u16_from_u8_key_table(PKMN_ENTRIES, INVALID_U16);
pub const EVENT_FLAG_MAP: [u16; NUM_EVENTS] =
    build_u16_table::<NUM_EVENTS>(EVENT_FLAG_ENTRIES, INVALID_U16);

pub fn map_v7_key_item_to_v8(v7: u8) -> u8 {
    KEY_ITEM_MAP[v7 as usize]
}

pub fn map_v7_item_to_v8(v7: u8) -> u8 {
    ITEM_MAP[v7 as usize]
}

pub fn map_v7_event_flag_to_v8(v7: u16) -> u16 {
    let idx = v7 as usize;
    if idx >= NUM_EVENTS {
        INVALID_U16
    } else {
        EVENT_FLAG_MAP[idx]
    }
}

pub fn map_v7_landmark_to_v8(v7: u8) -> u8 {
    LANDMARK_MAP[v7 as usize]
}

pub fn map_v7_spawn_to_v8(v7: u8) -> u8 {
    SPAWN_MAP[v7 as usize]
}

pub fn map_v7_pkmn_to_v8(v7: u16) -> u16 {
    let idx = v7 as usize;
    if idx >= 256 {
        INVALID_U16
    } else {
        PKMN_MAP[idx]
    }
}

pub fn map_v7_map_to_v8(group: u8, map: u8) -> (u8, u8) {
    for &((from_group, from_map), (to_group, to_map)) in MAP_GROUP_NUMBER_ENTRIES {
        if from_group == group && from_map == map {
            return (to_group, to_map);
        }
    }

    (0, 0)
}

pub fn map_v7_species_form_to_v8_extspecies(species: u16, form: u8) -> u16 {
    if species > (u8::MAX as u16) {
        return INVALID_U16;
    }

    let key = (species as u8, form);
    for &(k, v) in SPECIES_FORM_TO_EXTSPECIES_ENTRIES {
        if k == key {
            return v;
        }
    }

    INVALID_U16
}

pub fn map_v7_magikarp_form_to_v8(v7: u8) -> u8 {
    MAGIKARP_FORM_MAP[v7 as usize]
}

pub fn map_v7_theme_to_v8(v7: u8) -> u8 {
    THEME_MAP[v7 as usize]
}

pub fn map_v7_char_to_v8(v7: u8) -> u8 {
    CHAR_MAP[v7 as usize]
}
