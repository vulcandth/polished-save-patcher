include!("v9_to_v10_maps_data.rs");

pub const INVALID_U16: u16 = 0xFFFF;

pub const NUM_EVENTS: usize = 0x8FF;

const fn build_u16_table<const N: usize>(entries: &[(u16, u16)], default_value: u16) -> [u16; N] {
    let mut table = [default_value; N];

    let mut i = 0;
    while i < entries.len() {
        let (k, v) = entries[i];
        let idx = k as usize;
        if idx < N {
            table[idx] = v;
        }
        i += 1;
    }

    table
}

pub const EVENT_FLAG_MAP: [u16; NUM_EVENTS] =
    build_u16_table::<NUM_EVENTS>(EVENT_FLAG_ENTRIES, INVALID_U16);

#[inline]
pub fn map_v9_event_flag_to_v10(flag: u16) -> u16 {
    EVENT_FLAG_MAP
        .get(flag as usize)
        .copied()
        .unwrap_or(INVALID_U16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn event_flag_entries_are_unique_and_in_range() {
        let mut keys = HashSet::new();
        for &(from, to) in EVENT_FLAG_ENTRIES {
            assert!(
                (from as usize) < NUM_EVENTS,
                "event flag key out of range: {from}"
            );
            assert!(keys.insert(from), "duplicate event flag key: {from}");

            assert!(
                to == INVALID_U16 || (to as usize) < NUM_EVENTS,
                "event flag value out of range: {to}"
            );
        }
    }

    #[test]
    fn out_of_range_event_flags_map_to_invalid() {
        assert_eq!(map_v9_event_flag_to_v10(u16::MAX), INVALID_U16);
    }

    #[test]
    fn in_range_event_flag_maps_match_table_entries() {
        let (from, to) = EVENT_FLAG_ENTRIES
            .first()
            .copied()
            .expect("expected at least one event flag mapping");
        assert_eq!(map_v9_event_flag_to_v10(from), to);
    }
}
