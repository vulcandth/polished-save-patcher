#![allow(dead_code)]

use core::mem::size_of;

pub const NUM_MOVES: usize = 4;
pub const PARTY_LENGTH: usize = 6;
pub const PLAYER_NAME_LENGTH: usize = 8;
pub const MON_NAME_LENGTH: usize = 11;
pub const MAIL_MSG_LENGTH: usize = 0x20;

pub type MovesBytes = [u8; NUM_MOVES];
pub type ExpBytes = [u8; 3];
pub type EvsBytes = [u8; 6];
pub type DvsBytes = [u8; 3];
pub type PersonalityBytes = [u8; 2];
pub type ExtraBytes = [u8; 3];
pub type NicknameBytes = [u8; MON_NAME_LENGTH - 1];
pub type OtBytes = [u8; PLAYER_NAME_LENGTH - 1];
pub type StatsBytes = [u16; 5];
pub type MailMessageBytes = [u8; MAIL_MSG_LENGTH];
pub type MailAuthorBytes = [u8; PLAYER_NAME_LENGTH];

pub const SHINY_MASK: u8 = 0b1000_0000;
pub const ABILITY_MASK: u8 = 0b0110_0000;
pub const NATURE_MASK: u8 = 0b0001_1111;
pub const GENDER_MASK: u8 = 0b1000_0000;
pub const EGG_MASK: u8 = 0b0100_0000;
pub const EXTSPECIES_MASK: u8 = 0b0010_0000;
pub const FORM_MASK: u8 = 0b0001_1111;
pub const CAUGHT_GENDER_MASK: u8 = 0b1000_0000;
pub const CAUGHT_TIME_MASK: u8 = 0b0110_0000;
pub const CAUGHT_BALL_MASK: u8 = 0b0001_1111;

pub const BREEDMON_LEN: usize = BreedMonV9::BYTE_LEN;
pub const PARTYMON_LEN: usize = PartyMonV9::BYTE_LEN;
pub const SAVEMON_LEN: usize = SaveMonV9::BYTE_LEN;
pub const ROAMMON_LEN: usize = RoamMonV9::BYTE_LEN;
pub const MAILMSG_LEN: usize = MailMsgV9::BYTE_LEN;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BreedMonV9 {
    pub species: u8,
    pub item: u8,
    pub moves_: MovesBytes,
    pub id: u16,
    pub exp: ExpBytes,
    pub evs: EvsBytes,
    pub dvs: DvsBytes,
    pub personality: PersonalityBytes,
    pub pp: MovesBytes,
    pub happiness: u8,
    pub pkrus: u8,
    pub caughtdata: u8,
    pub caughtlevel: u8,
    pub caughtlocation: u8,
    pub level: u8,
}

impl BreedMonV9 {
    pub const BYTE_LEN: usize = size_of::<u8>()
        + size_of::<u8>()
        + size_of::<MovesBytes>()
        + size_of::<u16>()
        + size_of::<ExpBytes>()
        + size_of::<EvsBytes>()
        + size_of::<DvsBytes>()
        + size_of::<PersonalityBytes>()
        + size_of::<MovesBytes>()
        + size_of::<[u8; 6]>();

    pub fn from_bytes(bytes: &[u8; Self::BYTE_LEN]) -> Self {
        let mut cursor = 0usize;

        let species = bytes[cursor];
        cursor += 1;
        let item = bytes[cursor];
        cursor += 1;

        let mut moves_ = [0u8; NUM_MOVES];
        moves_.copy_from_slice(&bytes[cursor..cursor + NUM_MOVES]);
        cursor += NUM_MOVES;

        let id = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        let mut exp = [0u8; 3];
        exp.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut evs = [0u8; 6];
        evs.copy_from_slice(&bytes[cursor..cursor + 6]);
        cursor += 6;

        let mut dvs = [0u8; 3];
        dvs.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut personality = [0u8; 2];
        personality.copy_from_slice(&bytes[cursor..cursor + 2]);
        cursor += 2;

        let mut pp = [0u8; NUM_MOVES];
        pp.copy_from_slice(&bytes[cursor..cursor + NUM_MOVES]);
        cursor += NUM_MOVES;

        let happiness = bytes[cursor];
        cursor += 1;
        let pkrus = bytes[cursor];
        cursor += 1;
        let caughtdata = bytes[cursor];
        cursor += 1;
        let caughtlevel = bytes[cursor];
        cursor += 1;
        let caughtlocation = bytes[cursor];
        cursor += 1;
        let level = bytes[cursor];
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);

        Self {
            species,
            item,
            moves_,
            id,
            exp,
            evs,
            dvs,
            personality,
            pp,
            happiness,
            pkrus,
            caughtdata,
            caughtlevel,
            caughtlocation,
            level,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
        let mut out = [0u8; Self::BYTE_LEN];
        let mut cursor = 0usize;

        out[cursor] = self.species;
        cursor += 1;
        out[cursor] = self.item;
        cursor += 1;

        out[cursor..cursor + NUM_MOVES].copy_from_slice(&self.moves_);
        cursor += NUM_MOVES;

        out[cursor..cursor + 2].copy_from_slice(&self.id.to_le_bytes());
        cursor += 2;

        out[cursor..cursor + 3].copy_from_slice(&self.exp);
        cursor += 3;
        out[cursor..cursor + 6].copy_from_slice(&self.evs);
        cursor += 6;
        out[cursor..cursor + 3].copy_from_slice(&self.dvs);
        cursor += 3;

        out[cursor..cursor + 2].copy_from_slice(&self.personality);
        cursor += 2;
        out[cursor..cursor + NUM_MOVES].copy_from_slice(&self.pp);
        cursor += NUM_MOVES;

        out[cursor] = self.happiness;
        cursor += 1;
        out[cursor] = self.pkrus;
        cursor += 1;
        out[cursor] = self.caughtdata;
        cursor += 1;
        out[cursor] = self.caughtlevel;
        cursor += 1;
        out[cursor] = self.caughtlocation;
        cursor += 1;
        out[cursor] = self.level;
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);
        out
    }

    pub fn is_shiny(&self) -> bool {
        (self.personality[0] & SHINY_MASK) != 0
    }

    pub fn set_shiny(&mut self, shiny: bool) {
        if shiny {
            self.personality[0] |= SHINY_MASK;
        } else {
            self.personality[0] &= !SHINY_MASK;
        }
    }

    pub fn ability(&self) -> u8 {
        (self.personality[0] & ABILITY_MASK) >> 5
    }

    pub fn set_ability(&mut self, ability: u8) {
        self.personality[0] =
            (self.personality[0] & !ABILITY_MASK) | ((ability << 5) & ABILITY_MASK);
    }

    pub fn nature(&self) -> u8 {
        self.personality[0] & NATURE_MASK
    }

    pub fn set_nature(&mut self, nature: u8) {
        self.personality[0] = (self.personality[0] & !NATURE_MASK) | (nature & NATURE_MASK);
    }

    pub fn gender(&self) -> bool {
        (self.personality[1] & GENDER_MASK) != 0
    }

    pub fn set_gender(&mut self, gender: bool) {
        if gender {
            self.personality[1] |= GENDER_MASK;
        } else {
            self.personality[1] &= !GENDER_MASK;
        }
    }

    pub fn is_egg(&self) -> bool {
        (self.personality[1] & EGG_MASK) != 0
    }

    pub fn set_egg(&mut self, egg: bool) {
        if egg {
            self.personality[1] |= EGG_MASK;
        } else {
            self.personality[1] &= !EGG_MASK;
        }
    }

    pub fn ext_species(&self) -> u16 {
        (((self.personality[1] & EXTSPECIES_MASK) as u16) << 3) | (self.species as u16)
    }

    pub fn set_ext_species(&mut self, extspecies: u16) {
        self.personality[1] =
            (self.personality[1] & !EXTSPECIES_MASK) | (((extspecies & 0x100) >> 3) as u8);
        self.species = (extspecies & 0xFF) as u8;
    }

    pub fn form(&self) -> u8 {
        self.personality[1] & FORM_MASK
    }

    pub fn set_form(&mut self, form: u8) {
        self.personality[1] = (self.personality[1] & !FORM_MASK) | (form & FORM_MASK);
    }

    pub fn caught_time(&self) -> u8 {
        (self.caughtdata & CAUGHT_TIME_MASK) >> 5
    }

    pub fn set_caught_time(&mut self, time: u8) {
        self.caughtdata = (self.caughtdata & !CAUGHT_TIME_MASK) | ((time << 5) & CAUGHT_TIME_MASK);
    }

    pub fn caught_ball(&self) -> u8 {
        self.caughtdata & CAUGHT_BALL_MASK
    }

    pub fn set_caught_ball(&mut self, ball: u8) {
        self.caughtdata = (self.caughtdata & !CAUGHT_BALL_MASK) | (ball & CAUGHT_BALL_MASK);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartyMonV9 {
    pub breedmon: BreedMonV9,
    pub status: u8,
    pub unused: u8,
    pub hp: u16,
    pub maxhp: u16,
    pub stats: StatsBytes,
}

impl PartyMonV9 {
    pub const BYTE_LEN: usize = BreedMonV9::BYTE_LEN
        + size_of::<[u8; 2]>()
        + size_of::<u16>()
        + size_of::<u16>()
        + size_of::<StatsBytes>();

    pub fn from_bytes(bytes: &[u8; Self::BYTE_LEN]) -> Self {
        let mut cursor = 0usize;

        let mut breed_bytes = [0u8; BreedMonV9::BYTE_LEN];
        breed_bytes.copy_from_slice(&bytes[cursor..cursor + BreedMonV9::BYTE_LEN]);
        cursor += BreedMonV9::BYTE_LEN;
        let breedmon = BreedMonV9::from_bytes(&breed_bytes);

        let status = bytes[cursor];
        cursor += 1;
        let unused = bytes[cursor];
        cursor += 1;

        let hp = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        let maxhp = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        let mut stats = [0u16; 5];
        for stat in &mut stats {
            *stat = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
            cursor += 2;
        }

        debug_assert_eq!(cursor, Self::BYTE_LEN);

        Self {
            breedmon,
            status,
            unused,
            hp,
            maxhp,
            stats,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
        let mut out = [0u8; Self::BYTE_LEN];
        let mut cursor = 0usize;

        out[cursor..cursor + BreedMonV9::BYTE_LEN].copy_from_slice(&self.breedmon.to_bytes());
        cursor += BreedMonV9::BYTE_LEN;

        out[cursor] = self.status;
        cursor += 1;
        out[cursor] = self.unused;
        cursor += 1;

        out[cursor..cursor + 2].copy_from_slice(&self.hp.to_le_bytes());
        cursor += 2;
        out[cursor..cursor + 2].copy_from_slice(&self.maxhp.to_le_bytes());
        cursor += 2;

        for stat in self.stats {
            out[cursor..cursor + 2].copy_from_slice(&stat.to_le_bytes());
            cursor += 2;
        }

        debug_assert_eq!(cursor, Self::BYTE_LEN);
        out
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SaveMonV9 {
    pub species: u8,
    pub item: u8,
    pub moves_: MovesBytes,
    pub id: u16,
    pub exp: ExpBytes,
    pub evs: EvsBytes,
    pub dvs: DvsBytes,
    pub personality: PersonalityBytes,
    pub ppups: u8,
    pub happiness: u8,
    pub pkrus: u8,
    pub caughtdata: u8,
    pub caughtlevel: u8,
    pub caughtlocation: u8,
    pub level: u8,
    pub extra: ExtraBytes,
    pub nickname: NicknameBytes,
    pub ot: OtBytes,
}

impl SaveMonV9 {
    pub const BYTE_LEN: usize = size_of::<u8>()
        + size_of::<u8>()
        + size_of::<MovesBytes>()
        + size_of::<u16>()
        + size_of::<ExpBytes>()
        + size_of::<EvsBytes>()
        + size_of::<DvsBytes>()
        + size_of::<PersonalityBytes>()
        + size_of::<[u8; 7]>()
        + size_of::<ExtraBytes>()
        + size_of::<NicknameBytes>()
        + size_of::<OtBytes>();

    pub fn from_bytes(bytes: &[u8; Self::BYTE_LEN]) -> Self {
        let mut cursor = 0usize;

        let species = bytes[cursor];
        cursor += 1;
        let item = bytes[cursor];
        cursor += 1;

        let mut moves_ = [0u8; NUM_MOVES];
        moves_.copy_from_slice(&bytes[cursor..cursor + NUM_MOVES]);
        cursor += NUM_MOVES;

        let id = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        let mut exp = [0u8; 3];
        exp.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut evs = [0u8; 6];
        evs.copy_from_slice(&bytes[cursor..cursor + 6]);
        cursor += 6;

        let mut dvs = [0u8; 3];
        dvs.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut personality = [0u8; 2];
        personality.copy_from_slice(&bytes[cursor..cursor + 2]);
        cursor += 2;

        let ppups = bytes[cursor];
        cursor += 1;
        let happiness = bytes[cursor];
        cursor += 1;
        let pkrus = bytes[cursor];
        cursor += 1;
        let caughtdata = bytes[cursor];
        cursor += 1;
        let caughtlevel = bytes[cursor];
        cursor += 1;
        let caughtlocation = bytes[cursor];
        cursor += 1;
        let level = bytes[cursor];
        cursor += 1;

        let mut extra = [0u8; 3];
        extra.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut nickname = [0u8; MON_NAME_LENGTH - 1];
        nickname.copy_from_slice(&bytes[cursor..cursor + (MON_NAME_LENGTH - 1)]);
        cursor += MON_NAME_LENGTH - 1;

        let mut ot = [0u8; PLAYER_NAME_LENGTH - 1];
        ot.copy_from_slice(&bytes[cursor..cursor + (PLAYER_NAME_LENGTH - 1)]);
        cursor += PLAYER_NAME_LENGTH - 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);

        Self {
            species,
            item,
            moves_,
            id,
            exp,
            evs,
            dvs,
            personality,
            ppups,
            happiness,
            pkrus,
            caughtdata,
            caughtlevel,
            caughtlocation,
            level,
            extra,
            nickname,
            ot,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
        let mut out = [0u8; Self::BYTE_LEN];
        let mut cursor = 0usize;

        out[cursor] = self.species;
        cursor += 1;
        out[cursor] = self.item;
        cursor += 1;

        out[cursor..cursor + NUM_MOVES].copy_from_slice(&self.moves_);
        cursor += NUM_MOVES;

        out[cursor..cursor + 2].copy_from_slice(&self.id.to_le_bytes());
        cursor += 2;

        out[cursor..cursor + 3].copy_from_slice(&self.exp);
        cursor += 3;
        out[cursor..cursor + 6].copy_from_slice(&self.evs);
        cursor += 6;
        out[cursor..cursor + 3].copy_from_slice(&self.dvs);
        cursor += 3;

        out[cursor..cursor + 2].copy_from_slice(&self.personality);
        cursor += 2;

        out[cursor] = self.ppups;
        cursor += 1;
        out[cursor] = self.happiness;
        cursor += 1;
        out[cursor] = self.pkrus;
        cursor += 1;
        out[cursor] = self.caughtdata;
        cursor += 1;
        out[cursor] = self.caughtlevel;
        cursor += 1;
        out[cursor] = self.caughtlocation;
        cursor += 1;
        out[cursor] = self.level;
        cursor += 1;

        out[cursor..cursor + 3].copy_from_slice(&self.extra);
        cursor += 3;

        out[cursor..cursor + (MON_NAME_LENGTH - 1)].copy_from_slice(&self.nickname);
        cursor += MON_NAME_LENGTH - 1;

        out[cursor..cursor + (PLAYER_NAME_LENGTH - 1)].copy_from_slice(&self.ot);
        cursor += PLAYER_NAME_LENGTH - 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);
        out
    }

    pub fn is_shiny(&self) -> bool {
        (self.personality[0] & SHINY_MASK) != 0
    }

    pub fn set_shiny(&mut self, shiny: bool) {
        if shiny {
            self.personality[0] |= SHINY_MASK;
        } else {
            self.personality[0] &= !SHINY_MASK;
        }
    }

    pub fn ability(&self) -> u8 {
        (self.personality[0] & ABILITY_MASK) >> 5
    }

    pub fn set_ability(&mut self, ability: u8) {
        self.personality[0] =
            (self.personality[0] & !ABILITY_MASK) | ((ability << 5) & ABILITY_MASK);
    }

    pub fn nature(&self) -> u8 {
        self.personality[0] & NATURE_MASK
    }

    pub fn set_nature(&mut self, nature: u8) {
        self.personality[0] = (self.personality[0] & !NATURE_MASK) | (nature & NATURE_MASK);
    }

    pub fn gender(&self) -> bool {
        (self.personality[1] & GENDER_MASK) != 0
    }

    pub fn set_gender(&mut self, gender: bool) {
        if gender {
            self.personality[1] |= GENDER_MASK;
        } else {
            self.personality[1] &= !GENDER_MASK;
        }
    }

    pub fn is_egg(&self) -> bool {
        (self.personality[1] & EGG_MASK) != 0
    }

    pub fn set_egg(&mut self, egg: bool) {
        if egg {
            self.personality[1] |= EGG_MASK;
        } else {
            self.personality[1] &= !EGG_MASK;
        }
    }

    pub fn ext_species(&self) -> u16 {
        (((self.personality[1] & EXTSPECIES_MASK) as u16) << 3) | (self.species as u16)
    }

    pub fn set_ext_species(&mut self, extspecies: u16) {
        self.personality[1] =
            (self.personality[1] & !EXTSPECIES_MASK) | (((extspecies & 0x100) >> 3) as u8);
        self.species = (extspecies & 0xFF) as u8;
    }

    pub fn form(&self) -> u8 {
        self.personality[1] & FORM_MASK
    }

    pub fn set_form(&mut self, form: u8) {
        self.personality[1] = (self.personality[1] & !FORM_MASK) | (form & FORM_MASK);
    }

    pub fn caught_time(&self) -> u8 {
        (self.caughtdata & CAUGHT_TIME_MASK) >> 5
    }

    pub fn set_caught_time(&mut self, time: u8) {
        self.caughtdata = (self.caughtdata & !CAUGHT_TIME_MASK) | ((time << 5) & CAUGHT_TIME_MASK);
    }

    pub fn caught_ball(&self) -> u8 {
        self.caughtdata & CAUGHT_BALL_MASK
    }

    pub fn set_caught_ball(&mut self, ball: u8) {
        self.caughtdata = (self.caughtdata & !CAUGHT_BALL_MASK) | (ball & CAUGHT_BALL_MASK);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoamMonV9 {
    pub species: u8,
    pub level: u8,
    pub map_group: u8,
    pub map_number: u8,
    pub hp: u8,
    pub dvs: DvsBytes,
    pub personality: PersonalityBytes,
    pub status: u8,
}

impl RoamMonV9 {
    pub const BYTE_LEN: usize = size_of::<u8>()
        + size_of::<u8>()
        + size_of::<u8>()
        + size_of::<u8>()
        + size_of::<u8>()
        + size_of::<DvsBytes>()
        + size_of::<PersonalityBytes>()
        + size_of::<u8>();

    pub fn from_bytes(bytes: &[u8; Self::BYTE_LEN]) -> Self {
        let mut cursor = 0usize;

        let species = bytes[cursor];
        cursor += 1;
        let level = bytes[cursor];
        cursor += 1;
        let map_group = bytes[cursor];
        cursor += 1;
        let map_number = bytes[cursor];
        cursor += 1;
        let hp = bytes[cursor];
        cursor += 1;

        let mut dvs = [0u8; 3];
        dvs.copy_from_slice(&bytes[cursor..cursor + 3]);
        cursor += 3;

        let mut personality = [0u8; 2];
        personality.copy_from_slice(&bytes[cursor..cursor + 2]);
        cursor += 2;

        let status = bytes[cursor];
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);

        Self {
            species,
            level,
            map_group,
            map_number,
            hp,
            dvs,
            personality,
            status,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
        let mut out = [0u8; Self::BYTE_LEN];
        let mut cursor = 0usize;

        out[cursor] = self.species;
        cursor += 1;
        out[cursor] = self.level;
        cursor += 1;
        out[cursor] = self.map_group;
        cursor += 1;
        out[cursor] = self.map_number;
        cursor += 1;
        out[cursor] = self.hp;
        cursor += 1;

        out[cursor..cursor + 3].copy_from_slice(&self.dvs);
        cursor += 3;
        out[cursor..cursor + 2].copy_from_slice(&self.personality);
        cursor += 2;

        out[cursor] = self.status;
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);
        out
    }

    pub fn is_shiny(&self) -> bool {
        (self.personality[0] & SHINY_MASK) != 0
    }

    pub fn set_shiny(&mut self, shiny: bool) {
        if shiny {
            self.personality[0] |= SHINY_MASK;
        } else {
            self.personality[0] &= !SHINY_MASK;
        }
    }

    pub fn ability(&self) -> u8 {
        (self.personality[0] & ABILITY_MASK) >> 5
    }

    pub fn set_ability(&mut self, ability: u8) {
        self.personality[0] =
            (self.personality[0] & !ABILITY_MASK) | ((ability << 5) & ABILITY_MASK);
    }

    pub fn nature(&self) -> u8 {
        self.personality[0] & NATURE_MASK
    }

    pub fn set_nature(&mut self, nature: u8) {
        self.personality[0] = (self.personality[0] & !NATURE_MASK) | (nature & NATURE_MASK);
    }

    pub fn gender(&self) -> bool {
        (self.personality[1] & GENDER_MASK) != 0
    }

    pub fn set_gender(&mut self, gender: bool) {
        if gender {
            self.personality[1] |= GENDER_MASK;
        } else {
            self.personality[1] &= !GENDER_MASK;
        }
    }

    pub fn is_egg(&self) -> bool {
        (self.personality[1] & EGG_MASK) != 0
    }

    pub fn set_egg(&mut self, egg: bool) {
        if egg {
            self.personality[1] |= EGG_MASK;
        } else {
            self.personality[1] &= !EGG_MASK;
        }
    }

    pub fn map(&self) -> (u8, u8) {
        (self.map_group, self.map_number)
    }

    pub fn set_map(&mut self, map: (u8, u8)) {
        (self.map_group, self.map_number) = map;
    }

    pub fn set_ext_species(&mut self, extspecies: u16) {
        self.personality[1] =
            (self.personality[1] & !EXTSPECIES_MASK) | (((extspecies & 0x100) >> 3) as u8);
        self.species = (extspecies & 0xFF) as u8;
    }

    pub fn form(&self) -> u8 {
        self.personality[1] & FORM_MASK
    }

    pub fn set_form(&mut self, form: u8) {
        self.personality[1] = (self.personality[1] & !FORM_MASK) | (form & FORM_MASK);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MailMsgV9 {
    pub message: MailMessageBytes,
    pub message_end: u8,
    pub author: MailAuthorBytes,
    pub nationality: u16,
    pub author_id: u16,
    pub species: u8,
    pub r#type: u8,
}

impl MailMsgV9 {
    pub const BYTE_LEN: usize = size_of::<MailMessageBytes>()
        + size_of::<u8>()
        + size_of::<MailAuthorBytes>()
        + size_of::<u16>()
        + size_of::<u16>()
        + size_of::<u8>()
        + size_of::<u8>();

    pub fn from_bytes(bytes: &[u8; Self::BYTE_LEN]) -> Self {
        let mut cursor = 0usize;

        let mut message = [0u8; MAIL_MSG_LENGTH];
        message.copy_from_slice(&bytes[cursor..cursor + MAIL_MSG_LENGTH]);
        cursor += MAIL_MSG_LENGTH;

        let message_end = bytes[cursor];
        cursor += 1;

        let mut author = [0u8; PLAYER_NAME_LENGTH];
        author.copy_from_slice(&bytes[cursor..cursor + PLAYER_NAME_LENGTH]);
        cursor += PLAYER_NAME_LENGTH;

        let nationality = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        let author_id = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        let species = bytes[cursor];
        cursor += 1;
        let r#type = bytes[cursor];
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);

        Self {
            message,
            message_end,
            author,
            nationality,
            author_id,
            species,
            r#type,
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
        let mut out = [0u8; Self::BYTE_LEN];
        let mut cursor = 0usize;

        out[cursor..cursor + MAIL_MSG_LENGTH].copy_from_slice(&self.message);
        cursor += MAIL_MSG_LENGTH;

        out[cursor] = self.message_end;
        cursor += 1;

        out[cursor..cursor + PLAYER_NAME_LENGTH].copy_from_slice(&self.author);
        cursor += PLAYER_NAME_LENGTH;

        out[cursor..cursor + 2].copy_from_slice(&self.nationality.to_le_bytes());
        cursor += 2;
        out[cursor..cursor + 2].copy_from_slice(&self.author_id.to_le_bytes());
        cursor += 2;

        out[cursor] = self.species;
        cursor += 1;
        out[cursor] = self.r#type;
        cursor += 1;

        debug_assert_eq!(cursor, Self::BYTE_LEN);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breedmon_round_trip_bytes() {
        let mon = BreedMonV9 {
            species: 0x12,
            item: 0x34,
            moves_: [1, 2, 3, 4],
            id: 0xBEEF,
            exp: [9, 8, 7],
            evs: [1, 1, 2, 3, 5, 8],
            dvs: [0xAA, 0xBB, 0xCC],
            personality: [0x10, 0x20],
            pp: [10, 11, 12, 13],
            happiness: 0x55,
            pkrus: 0x66,
            caughtdata: 0x77,
            caughtlevel: 0x88,
            caughtlocation: 0x99,
            level: 0x2A,
        };

        let bytes = mon.to_bytes();
        let decoded = BreedMonV9::from_bytes(&bytes);
        assert_eq!(decoded, mon);
    }

    #[test]
    fn savemon_round_trip_bytes() {
        let mon = SaveMonV9 {
            species: 0x01,
            item: 0x02,
            moves_: [0x10, 0x11, 0x12, 0x13],
            id: 0x1234,
            exp: [0xAA, 0xBB, 0xCC],
            evs: [1, 2, 3, 4, 5, 6],
            dvs: [0x01, 0x02, 0x03],
            personality: [0xFE, 0xED],
            ppups: 0x0F,
            happiness: 0x80,
            pkrus: 0x00,
            caughtdata: 0x5A,
            caughtlevel: 0x19,
            caughtlocation: 0x77,
            level: 0x64,
            extra: [0xDE, 0xAD, 0xBE],
            nickname: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            ot: [11, 12, 13, 14, 15, 16, 17],
        };

        let bytes = mon.to_bytes();
        let decoded = SaveMonV9::from_bytes(&bytes);
        assert_eq!(decoded, mon);
    }

    #[test]
    fn party_round_trip_bytes() {
        let mon = PartyMonV9 {
            breedmon: BreedMonV9 {
                species: 0x12,
                item: 0x34,
                moves_: [1, 2, 3, 4],
                id: 0xBEEF,
                exp: [9, 8, 7],
                evs: [1, 1, 2, 3, 5, 8],
                dvs: [0xAA, 0xBB, 0xCC],
                personality: [0x10, 0x20],
                pp: [10, 11, 12, 13],
                happiness: 0x55,
                pkrus: 0x66,
                caughtdata: 0x77,
                caughtlevel: 0x88,
                caughtlocation: 0x99,
                level: 0x2A,
            },
            status: 0x11,
            unused: 0x22,
            stats: [1, 2, 3, 4, 5],
            hp: 0x1234,
            maxhp: 0x2345,
        };

        let bytes = mon.to_bytes();
        let decoded = PartyMonV9::from_bytes(&bytes);
        assert_eq!(decoded, mon);
    }

    #[test]
    fn roammon_round_trip_bytes() {
        let mon = RoamMonV9 {
            species: 0x42,
            level: 0x33,
            map_group: 0x05,
            map_number: 0x06,
            hp: 0x99,
            dvs: [0x01, 0x02, 0x03],
            status: 0xAA,
            personality: [0xAB, 0xCD],
        };

        let bytes = mon.to_bytes();
        let decoded = RoamMonV9::from_bytes(&bytes);
        assert_eq!(decoded, mon);
    }

    #[test]
    fn mailmsg_round_trip_bytes() {
        let msg = MailMsgV9 {
            message: core::array::from_fn(|i| (i as u8).wrapping_mul(3).wrapping_add(1)),
            ..Default::default()
        };

        let bytes = msg.to_bytes();
        let decoded = MailMsgV9::from_bytes(&bytes);
        assert_eq!(decoded, msg);
    }
}
