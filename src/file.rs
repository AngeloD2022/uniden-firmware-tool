use crate::util::{alter_length, CursorHelper};
use std::{fs, io};
use std::io::Cursor;
use binrw::helpers::count;
use crate::format::{decode_old_model, GpsDbCountry, GpsDbType, NEW_FILE_GPS_DB_IDENTIFY_STR, OLD_FILE_GPS_DB_IDENTIFY_STR, OLD_IL_GPS_DB_KEY, OLD_NZ_GPS_DB_KEY, OLD_US_GPS_DB_KEY, rd_version, RDModel, SOUND_DB_KEY};

struct FileInfo {
    length: i32,
    offset: i32,
    version: i32,
}

struct GpsDbFileInfo {
    length: i32,
    offset: i32,
    version: i32,
    poi: i32,
    file_type: GpsDbType,
    country: Option<GpsDbCountry>,
}

enum FWFile {
    UiNu(FileInfo),
    UiStm(FileInfo),
    UiNu2(FileInfo),
    DspNu(FileInfo),
    DspStmfNu(FileInfo),
    DspNu2(FileInfo),
    DspNu3(FileInfo),
    GpsNu(FileInfo),
    GpsStmf(FileInfo),
    GpsNu2(FileInfo),
    GpsNu3(FileInfo),
    SoundDbnu(FileInfo),
    SoundDbla1(FileInfo),
    SoundDbla2(FileInfo),
    GpsDb(GpsDbFileInfo),
    GpsDbSecond(GpsDbFileInfo),
    Ble(FileInfo),
    Keypad(FileInfo),
    LaserIf(FileInfo),
}

impl FWFile {
    fn to_file_name(&self) -> String {
        match self {
            FWFile::UiNu(_) => "ui_nu.bin".into(),
            FWFile::UiStm(_) => "ui_stm.bin".into(),
            FWFile::UiNu2(_) => "ui_nu2.bin".into(),
            FWFile::DspNu(_) => "dsp_nu.bin".into(),
            FWFile::DspStmfNu(_) => "dsp_stmf_nu.bin".into(),
            FWFile::DspNu2(_) => "dsp_nu2.bin".into(),
            FWFile::DspNu3(_) => "dsp_nu3.bin".into(),
            FWFile::GpsNu(_) => "gps_nu.bin".into(),
            FWFile::GpsStmf(_) => "gps_stmf.bin".into(),
            FWFile::GpsNu2(_) => "gps_nu2.bin".into(),
            FWFile::GpsNu3(_) => "gps_nu3.bin".into(),
            FWFile::SoundDbnu(_) => "sound_dbnu.bin".into(),
            FWFile::SoundDbla1(_) => "sound_dbla1.bin".into(),
            FWFile::SoundDbla2(_) => "sound_dbla2.bin".into(),
            FWFile::GpsDb(_) => "gps_db.bin".into(),
            FWFile::GpsDbSecond(_) => "gps_db_second.bin".into(),
            FWFile::Ble(_) => "ble.bin".into(),
            FWFile::Keypad(_) => "keypad.bin".into(),
            FWFile::LaserIf(_) => "laser_if.bin".into(),
        }
    }
}


struct FWMetadata {
    model: RDModel,
    format_version: i16,
    new_merge_file: bool
}

/// (offset, version, end string)
fn parse_file_basic(cursor: &mut Cursor<&Vec<u8>>, length: i32) -> io::Result<(i32, i32, String)> {
    let offset = cursor.position() as i32;
    cursor.seek(length as u64);

    let arr = cursor.read_n(9)?;
    let version = rd_version(i16::from_le_bytes(arr[0..2].try_into().unwrap())) as i32;
    let end_string = String::from_utf8(arr[2..].to_vec()).unwrap();
    return Ok((offset, version, end_string))
}


macro_rules! stfu {
    ($ex:expr) => {String::from_utf8($ex.to_vec()).unwrap()};
}


pub struct UnidenFirmware {
    metadata: Option<FWMetadata>,
    files: Vec<FWFile>,
    buffer: Vec<u8>,
}

impl UnidenFirmware {
    pub fn from(file_path: &str) -> Result<UnidenFirmware, String> {
        let buffer = fs::read(file_path).map_err(|e| e.to_string())?;

        Ok(
            Self {
                metadata: None,
                files: vec![],
                buffer
            }
        )
    }

    pub fn read_buffer(&mut self) -> io::Result<()> {

        let mut files = Vec::new();
        let mut metadata = FWMetadata {
            model: RDModel::Unknown,
            format_version: 0,
            new_merge_file: false,
        };

        let mut cursor = Cursor::new(&self.buffer);

        let first_element = i32::from_le_bytes(
            cursor.read_n(4)?.try_into().unwrap()
        );
        println!("First: {:x}", first_element);

        let ui_nu_len = alter_length(first_element & 0xFFFFFF);
        let flag_includes_sound_db = (first_element >> 0x18) & 0x1;

        let dsp_nu_len = alter_length(
            i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap())
        );
        let gps_nu_len = alter_length(
            i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap())
        );

        let mut sound_db_nu_len = 0;
        if flag_includes_sound_db == 1 {
            let slice = &cursor.read_n(12)?[9..];
            sound_db_nu_len = i32::from_le_bytes(slice.try_into().unwrap());
        }

        if ui_nu_len != 0 {
            let ui_nu_offset = cursor.position();
            cursor.seek(ui_nu_len as u64);

            let arr = cursor.read_n(9)?;
            let mv_data = i16::from_le_bytes(arr[0..2].try_into().unwrap());

            let model = RDModel::from_data(mv_data);
            let ui_nu_version = rd_version(mv_data);

            if String::from_utf8(arr[2..].to_vec()).unwrap() != "DRSWMAI" {
                // todo: replace these panics with a custom error
                panic!("Wrong format.");
            }

            metadata.model = model;
            files.push(FWFile::UiNu(FileInfo{
                length: ui_nu_len,
                offset: ui_nu_offset as i32,
                version: ui_nu_version as i32,
            }));
        }

        if dsp_nu_len != 0 {
            let (offset, version, endstr) = parse_file_basic(&mut cursor, dsp_nu_len)?;
            if endstr != "DRSWDSP" {
                panic!("Wrong format.");
            }

            files.push(FWFile::DspNu(FileInfo{
                length: dsp_nu_len,
                offset,
                version,
            }));
        }

        if gps_nu_len != 0 {
            let (offset, version, endstr) = parse_file_basic(&mut cursor, gps_nu_len)?;
            if endstr != "DRSWSUB" {
                panic!("Wrong format.");
            }

            files.push(FWFile::GpsNu(FileInfo{
                length: dsp_nu_len,
                offset,
                version,
            }));
        }

        if sound_db_nu_len != 0 {
            let offset = cursor.position() as i32;
            cursor.seek(sound_db_nu_len as u64 - 12);

            let arr = cursor.read_n(12)?;
            let vbuf = decode_old_model(SOUND_DB_KEY, &arr, 0, 4);

            let version = rd_version(i32::from_le_bytes(vbuf.try_into().unwrap()) as i16) as i32;

            let arr = cursor.read_n(7)?;
            if String::from_utf8(arr).unwrap() != "DRSWSDB" {
                panic!("Wrong format.");
            }

            files.push(FWFile::SoundDbnu(FileInfo{
                length: sound_db_nu_len,
                offset,
                version
            }));
        }

        if cursor.position() == cursor.get_ref().len() as u64 {
            return Ok(())
        }

        while cursor.position() != cursor.get_ref().len() as u64 {
            let arr = cursor.read_n(12)?;
            let switch = String::from_utf8(arr[0..5].to_vec()).unwrap();
            let current_length = i32::from_le_bytes(arr[8..].try_into().unwrap());
            let current_offset = cursor.position();

            match switch.as_ref() {
                "GPSD" => {
                    cursor.seek(current_length as u64 - 12);
                    let arr = cursor.read_n(12);
                    let gps_db = stfu!(arr[8..]);
                    let mut file = GpsDbFileInfo{
                        length: current_length,
                        offset: current_offset,
                        version: 0,
                        poi: 0,
                        file_type: GpsDbType::Unknown,
                        country: None,
                    };
                    if OLD_FILE_GPS_DB_IDENTIFY_STR.contains(&gps_db) {
                        file.file_type = GpsDbType::GpsDbOldEnc;
                        let (key, country) = match gps_db.as_ref() {
                            "LRDB" => (OLD_US_GPS_DB_KEY, GpsDbCountry::Us),
                            "DFDB" => (OLD_NZ_GPS_DB_KEY, GpsDbCountry::Nz),
                            "IRDB" => (OLD_IL_GPS_DB_KEY, GpsDbCountry::Il),
                            _ => panic!("malformed gps db."),
                        };
                        file.country = country;
                        file.poi = i32::from_le_bytes(
                            decode_old_model(key, &arr, 0, 4).try_into().unwrap()
                        );
                    } else if NEW_FILE_GPS_DB_IDENTIFY_STR.contains(&gps_db) {
                        file.file_type = GpsDbType::GpsDbAes128;
                        file.country = match gps_db.as_ref() {
                            "AEUS" => GpsDbCountry::Us,
                            "AENZ" => GpsDbCountry::Nz,
                            "AEIL" => GpsDbCountry::Il,
                            "AEEU" => GpsDbCountry::Eu,
                            _ => panic!("malformed gps db."),
                        };
                        file.poi = i32::from_le_bytes(
                            arr[0..5].try_into().unwrap()
                        );
                    } else {
                        panic!("Malformed GpsDb!");
                    }
                    file.version = i32::from_le_bytes(arr[4..9].try_into().unwrap());
                    files.push(FWFile::GpsDb(file));
                }
                _ => {}
            }
        }

        todo!()
    }

    pub fn extract_to(&self, directory: &str) {
        for file in self.files {
            let content = &self.buffer[]
        }
    }
}



