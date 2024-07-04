pub const SOUND_DB_KEY: u8 = 255;
pub const OLD_US_GPS_DB_KEY: u8 = 210;
pub const OLD_NZ_GPS_DB_KEY: u8 = 194;
pub const OLD_IL_GPS_DB_KEY: u8 = 226;

pub const OLD_FILE_GPS_DB_IDENTIFY_STR: [&str; 3] = ["LRDB", "DFDB", "IRDB"];

pub const NEW_FILE_GPS_DB_IDENTIFY_STR: [&str; 4] = ["AEUS", "AENZ", "AEIL", "AEEU"];

pub enum GpsDbType {
    GpsDbOldEnc,
    GpsDbAes128,
    Unknown,
}

pub enum GpsDbCountry {
    Us,
    Nz,
    Il,
    Eu,
}

#[repr(u8)]
pub enum RDModel {
    R1 = 1,
    R3 = 3,
    R3Nz = 4,
    R3Nzk = 5,
    R3Plus = 64,
    R3NzkPlus = 65,
    R7 = 7,
    R7Nz = 8,
    R7Il = 9,
    R4 = 14,
    R4Nz = 15,
    R4Il = 16,
    R4Eu = 17,
    R8 = 18,
    R8Nz = 19,
    R8Il = 20,
    R8Eu = 21,
    R4W = 24,
    R8W = 28,
    DbEu = 236,
    DbIl = 237,
    DbUs = 238,
    DbNz = 239,
    Unknown = 255,
}

impl From<u8> for RDModel {
    fn from(item: u8) -> Self {
        match item {
            1 => RDModel::R1,
            3 => RDModel::R3,
            4 => RDModel::R3Nz,
            5 => RDModel::R3Nzk,
            64 => RDModel::R3Plus,
            65 => RDModel::R3NzkPlus,
            7 => RDModel::R7,
            8 => RDModel::R7Nz,
            9 => RDModel::R7Il,
            14 => RDModel::R4,
            15 => RDModel::R4Nz,
            16 => RDModel::R4Il,
            17 => RDModel::R4Eu,
            18 => RDModel::R8,
            19 => RDModel::R8Nz,
            20 => RDModel::R8Il,
            21 => RDModel::R8Eu,
            24 => RDModel::R4W,
            28 => RDModel::R8W,
            236 => RDModel::DbEu,
            237 => RDModel::DbIl,
            238 => RDModel::DbUs,
            239 => RDModel::DbNz,
            _ => RDModel::Unknown,
        }
    }
}

impl RDModel {
    pub(crate) fn from_data(data: i16) -> Self {
        // mask the upper six bits from `data`
        let model = ((data >> 10) & 0x3F) as u8;
        model.into()
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            RDModel::R1 => "R1",
            RDModel::R3 => "R3",
            RDModel::R3Nz => "R3NZ",
            RDModel::R3Nzk => "R3NZK",
            RDModel::R3Plus => "R3-PLUS",
            RDModel::R3NzkPlus => "R3NZK-PLUS",
            RDModel::R7 => "R7",
            RDModel::R7Nz => "R7NZ",
            RDModel::R7Il => "R7IL",
            RDModel::R4 => "R4",
            RDModel::R4Nz => "R4NZ",
            RDModel::R4Il => "R4IL",
            RDModel::R4Eu => "R4EU",
            RDModel::R8 => "R8",
            RDModel::R8Nz => "R8NZ",
            RDModel::R8Il => "R8IL",
            RDModel::R8Eu => "R8EU",
            RDModel::R4W => "R4W",
            RDModel::R8W => "R8W",
            RDModel::DbEu => "DBEU",
            RDModel::DbIl => "DBIL",
            RDModel::DbUs => "DBUS",
            RDModel::DbNz => "DBNZ",
            RDModel::Unknown => "UNKNOWN",
        }
    }
}

#[inline(always)]
pub(crate) fn rd_version(data: i16) -> i16 {
    if data == -1 {
        data
    } else {
        data & 0x3FF
    }
}

pub(crate) fn decode_old_model(key: u8, data: &[u8], offset: usize, length: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; length];

    for i in (0..length).step_by(4) {
        buffer[i] = data[i + offset] & 3;
        buffer[i] += (data[i + 1 + offset] & 3) << 2;
        buffer[i] += (data[i + 2 + offset] & 3) << 4;
        buffer[i] += (data[i + 3 + offset] & 3) << 6;

        buffer[i + 1] = (data[i + offset] & 0xC) >> 2;
        buffer[i + 1] += data[i + 1 + offset] & 0xC;
        buffer[i + 1] += (data[i + 2 + offset] & 0xC) << 2;
        buffer[i + 1] += (data[i + 3 + offset] & 0xC) << 4;

        buffer[i + 2] = (data[i + offset] & 0x30) >> 4;
        buffer[i + 2] += (data[i + 1 + offset] & 0x30) >> 2;
        buffer[i + 2] += data[i + 2 + offset] & 0x30;
        buffer[i + 2] += (data[i + 3 + offset] & 0x30) << 2;

        buffer[i + 3] = (data[i + offset] & 0xC0) >> 6;
        buffer[i + 3] += (data[i + 1 + offset] & 0xC0) >> 4;
        buffer[i + 3] += (data[i + 2 + offset] & 0xC0) >> 2;
        buffer[i + 3] += data[i + 3 + offset] & 0xC0;

        buffer[i] = (buffer[i] as i8 - key as i8) as u8;
        buffer[i + 1] = (buffer[i + 1] as i8 - key as i8) as u8;
        buffer[i + 2] = (buffer[i + 2] as i8 - key as i8) as u8;
        buffer[i + 3] = (buffer[i + 3] as i8 - key as i8) as u8;
    }

    buffer
}
