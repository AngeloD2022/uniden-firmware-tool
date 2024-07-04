use crate::util::{alter_length, CursorHelper};
use std::{fs, io};
use std::io::Cursor;
use crate::format::{GpsDbCountry, GpsDbType, rd_version, RDModel};

struct FileInfo {
    length: i32,
    offset: i32,
    version: i32,
}

struct GpsDbFileInfo {
    length: i32,
    offset: i32,
    version: i32,
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


struct FWMetadata {
    model: RDModel,
    format_version: i16,
    new_merge_file: bool
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

        let ui_nu_len = alter_length(first_element & 0xFFFFFF00);
        let flag = (first_element >> 0x18) & 0x1;

        let dsp_nu_len = alter_length(
            i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap())
        );
        let gps_nu_len = alter_length(
            i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap())
        );

        let mut sound_db_nu_len = 0;
        if flag == 1 {
            let slice = &cursor.read_n(12)?[9..];
            sound_db_nu_len = i32::from_le_bytes(slice.try_into().unwrap());
        }

        if ui_nu_len != 0 {
            let ui_nu_offset = cursor.position();
            cursor.seek(ui_nu_len as u64);

            let arr = cursor.read_n(9)?;
            let mv_data = i16::from_le_bytes(arr[0..3].try_into().unwrap());

            let model = RDModel::from_data(mv_data);
            let ui_nu_version = rd_version(mv_data);

            if String::from_utf8(arr[3..].to_vec()).unwrap() != "DRSWMAI" {
                // todo: replace this
                panic!("Wrong format.");
            }

            metadata.model = model;
            files.push(FWFile::UiNu(FileInfo{
                length: ui_nu_len,
                offset: ui_nu_offset as i32,
                version: ui_nu_version as i32,
            }));
        }

        todo!()
    }

    pub fn extract_to(&self, directory: &str) {
        todo!()
    }
}
