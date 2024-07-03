

struct FileInfo {
    length: i32,
    offset: i32,
    version: i32
}

enum GpsDbType {
    GpsDbOldEnc,
    GpsDbAes128,
    Unknown
}

enum GpsDbCountry {
    Us,
    Nz,
    Il,
    Eu
}

struct GpsDbFileInfo {
    length: i32,
    offset: i32,
    version: i32,
    file_type: GpsDbType,
    country: Option<GpsDbCountry>
}

enum R8FileType {
    UiNu(FileInfo),
    UiStm(FileInfo),
    UiNu2(FileInfo),
    DspNu(FileInfo),
    DspSTMFNu(FileInfo),
    DspNu2(FileInfo),
    DspNu3(FileInfo),
    GpsNu(FileInfo),
    GpsStmf(FileInfo),
    GpsNu2(FileInfo),
    GpsNu3(FileInfo),
    SoundDbnu(FileInfo),
    SoundDbla1(FileInfo),
    SoundDbla2(FileInfo),
    GpsDb(FileInfo),
    GpsDbSecond(FileInfo),
    Ble(FileInfo),
    Keypad(FileInfo),
    LaserIf(FileInfo)
}

#[repr(u8)]
pub enum ModelName{
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
    UNKNOWN = 255,
}