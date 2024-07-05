use crate::format::{
    decode_old_model, rd_version, GpsDbCountry, GpsDbType, RDModel, NEW_FILE_GPS_DB_IDENTIFY_STR,
    OLD_FILE_GPS_DB_IDENTIFY_STR, OLD_IL_GPS_DB_KEY, OLD_NZ_GPS_DB_KEY, OLD_US_GPS_DB_KEY,
    SOUND_DB_KEY,
};
use crate::util::{alter_length, CursorHelper};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::{fs, io, path};

#[derive(Clone, Copy)]
struct FileInfoBase {
    length: i32,
    offset: i32,
    version: i32,
}

#[derive(Clone, Copy)]
struct GpsDbFileInfo {
    info: FileInfoBase,
    poi: i32,
    file_type: GpsDbType,
    country: Option<GpsDbCountry>,
}

#[derive(Clone, Copy)]
pub enum FileInfo {
    Base(FileInfoBase),
    GpsDb(GpsDbFileInfo),
}

pub enum FWFileKind {
    UiNu(FileInfo),
    UiStm(FileInfo),
    UiNu2(FileInfo),
    DspNu(FileInfo),
    DspStm(FileInfo),
    DspNu2(FileInfo),
    DspNu3(FileInfo),
    GpsNu(FileInfo),
    GpsStm(FileInfo),
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

impl FWFileKind {
    pub fn to_file_name(&self) -> String {
        match self {
            FWFileKind::UiNu(_) => "ui_nu.bin".into(),
            FWFileKind::UiStm(_) => "ui_stm.bin".into(),
            FWFileKind::UiNu2(_) => "ui_nu2.bin".into(),
            FWFileKind::DspNu(_) => "dsp_nu.bin".into(),
            FWFileKind::DspStm(_) => "dsp_stm.bin".into(),
            FWFileKind::DspNu2(_) => "dsp_nu2.bin".into(),
            FWFileKind::DspNu3(_) => "dsp_nu3.bin".into(),
            FWFileKind::GpsNu(_) => "gps_nu.bin".into(),
            FWFileKind::GpsStm(_) => "gps_stm.bin".into(),
            FWFileKind::GpsNu2(_) => "gps_nu2.bin".into(),
            FWFileKind::GpsNu3(_) => "gps_nu3.bin".into(),
            FWFileKind::SoundDbnu(_) => "sound_dbnu.bin".into(),
            FWFileKind::SoundDbla1(_) => "sound_dbla1.bin".into(),
            FWFileKind::SoundDbla2(_) => "sound_dbla2.bin".into(),
            FWFileKind::GpsDb(_) => "gps_db.bin".into(),
            FWFileKind::GpsDbSecond(_) => "gps_db_second.bin".into(),
            FWFileKind::Ble(_) => "ble.bin".into(),
            FWFileKind::Keypad(_) => "keypad.bin".into(),
            FWFileKind::LaserIf(_) => "laser_if.bin".into(),
        }
    }
}

pub struct FWFile {
    pub(crate) kind: FWFileKind,
    pub(crate) info: FileInfo,
}

pub fn handle_gpsdb_file_info(file: &FWFileKind) -> Option<&GpsDbFileInfo> {
    match file {
        FWFileKind::GpsDb(gps_db_file_info) | FWFileKind::GpsDbSecond(gps_db_file_info) => {
            Some(gps_db_file_info)
        }
        _ => None,
    }
}

pub fn handle_file_info(file: &FWFileKind) -> Option<&FileInfo> {
    match file {
        FWFileKind::UiNu(file_info)
        | FWFileKind::UiStm(file_info)
        | FWFileKind::UiNu2(file_info)
        | FWFileKind::DspNu(file_info)
        | FWFileKind::DspStm(file_info)
        | FWFileKind::DspNu2(file_info)
        | FWFileKind::DspNu3(file_info)
        | FWFileKind::GpsNu(file_info)
        | FWFileKind::GpsStm(file_info)
        | FWFileKind::GpsNu2(file_info)
        | FWFileKind::GpsNu3(file_info)
        | FWFileKind::SoundDbnu(file_info)
        | FWFileKind::SoundDbla1(file_info)
        | FWFileKind::SoundDbla2(file_info)
        | FWFileKind::Ble(file_info)
        | FWFileKind::Keypad(file_info)
        | FWFileKind::LaserIf(file_info) => Some(file_info),
        _ => None,
    }
}

pub struct FWMetadata {
    pub model: RDModel,
    pub format_version: i32,
    pub new_merge_file: bool,
}

/// (offset, version, end string)
fn parse_file_basic(cursor: &mut Cursor<&Vec<u8>>, length: i32) -> io::Result<(i32, i32, String)> {
    let offset = cursor.position() as i32;
    cursor.seek(length as u64);

    let arr = cursor.read_n(9)?;
    let version = rd_version(i16::from_le_bytes(arr[0..2].try_into().unwrap())) as i32;
    let end_string = String::from_utf8(arr[2..].to_vec()).unwrap();
    Ok((offset, version, end_string))
}

macro_rules! stfu {
    ($ex:expr) => {
        String::from_utf8($ex.to_vec()).unwrap()
    };
}

pub struct UnidenFirmware {
    pub metadata: Option<FWMetadata>,
    pub(crate) files: Vec<FWFile>,
    buffer: Vec<u8>,
}

impl UnidenFirmware {
    pub fn from(file_path: &PathBuf) -> Result<UnidenFirmware, String> {
        let buffer = fs::read(file_path).map_err(|e| e.to_string())?;

        Ok(Self {
            metadata: None,
            files: vec![],
            buffer,
        })
    }

    pub fn read_buffer(&mut self) -> io::Result<()> {
        let mut files = Vec::new();
        let mut metadata = FWMetadata {
            model: RDModel::Unknown,
            format_version: 0,
            new_merge_file: false,
        };

        let mut cursor = Cursor::new(&self.buffer);

        let first_element = i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap());

        let ui_nu_len = alter_length(first_element & 0xFFFFFF);
        let flag_includes_sound_db = (first_element >> 0x18) & 0x1;

        let dsp_nu_len = alter_length(i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap()));
        let gps_nu_len = alter_length(i32::from_le_bytes(cursor.read_n(4)?.try_into().unwrap()));

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
            let info = FileInfo::Base(FileInfoBase {
                length: ui_nu_len,
                offset: ui_nu_offset as i32,
                version: ui_nu_version as i32,
            });
            files.push(FWFile {
                kind: FWFileKind::UiNu(info),
                info,
            });
        }

        if dsp_nu_len != 0 {
            let (offset, version, endstr) = parse_file_basic(&mut cursor, dsp_nu_len)?;
            if endstr != "DRSWDSP" {
                panic!("Wrong format.");
            }

            let info = FileInfo::Base(FileInfoBase {
                length: dsp_nu_len,
                offset,
                version,
            });
            files.push(FWFile {
                kind: FWFileKind::DspNu(info),
                info,
            });
        }

        if gps_nu_len != 0 {
            let (offset, version, endstr) = parse_file_basic(&mut cursor, gps_nu_len)?;
            if endstr != "DRSWSUB" {
                panic!("Wrong format.");
            }

            let info = FileInfo::Base(FileInfoBase {
                length: dsp_nu_len,
                offset,
                version,
            });
            files.push(FWFile {
                kind: FWFileKind::GpsNu(info),
                info,
            });
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

            let info = FileInfo::Base(FileInfoBase {
                length: sound_db_nu_len,
                offset,
                version,
            });
            files.push(FWFile {
                kind: FWFileKind::SoundDbnu(info),
                info,
            });
        }

        if cursor.position() == cursor.get_ref().len() as u64 {
            return Ok(());
        }

        while cursor.position() != cursor.get_ref().len() as u64 {
            let arr = cursor.read_n(12)?;
            let switch = String::from_utf8(arr[0..4].to_vec()).unwrap();
            let current_length = i32::from_le_bytes(arr[8..].try_into().unwrap());
            let current_offset = cursor.position();

            match switch.as_ref() {
                "GPSD" | "GASD" => {
                    cursor.seek(current_length as u64 - 12);
                    let arr = cursor.read_n(12)?;
                    let gps_db = stfu!(arr[8..]);
                    let mut file = GpsDbFileInfo {
                        info: FileInfoBase {
                            length: current_length,
                            offset: current_offset as i32,
                            version: 0,
                        },
                        poi: 0,
                        file_type: GpsDbType::Unknown,
                        country: None,
                    };
                    if OLD_FILE_GPS_DB_IDENTIFY_STR.contains(&&*gps_db) {
                        file.file_type = GpsDbType::GpsDbOldEnc;
                        let (key, country) = match gps_db.as_ref() {
                            "LRDB" => (OLD_US_GPS_DB_KEY, GpsDbCountry::Us),
                            "DFDB" => (OLD_NZ_GPS_DB_KEY, GpsDbCountry::Nz),
                            "IRDB" => (OLD_IL_GPS_DB_KEY, GpsDbCountry::Il),
                            _ => unreachable!(),
                        };
                        file.country = Some(country);
                        file.poi = i32::from_le_bytes(
                            decode_old_model(key, &arr, 0, 4).try_into().unwrap(),
                        );
                    } else if NEW_FILE_GPS_DB_IDENTIFY_STR.contains(&&*gps_db) {
                        file.file_type = GpsDbType::GpsDbAes128;
                        let country = match gps_db.as_ref() {
                            "AEUS" => GpsDbCountry::Us,
                            "AENZ" => GpsDbCountry::Nz,
                            "AEIL" => GpsDbCountry::Il,
                            "AEEU" => GpsDbCountry::Eu,
                            _ => unreachable!(),
                        };
                        file.country = Some(country);
                        file.poi = i32::from_le_bytes(arr[0..4].try_into().unwrap());
                    } else {
                        panic!("Malformed GPS DB File Info!");
                    }

                    file.info.version = i32::from_le_bytes(arr[4..8].try_into().unwrap());

                    if switch == "GASD" {
                        cursor.seek(2);
                    }

                    let term_string = stfu!(cursor.read_n(7)?);
                    if (term_string != "DRSWGDB" && switch == "GPSD")
                        || (term_string != "DRSWGAE" && switch == "GASD")
                    {
                        panic!("Error found on termination string.");
                    }

                    files.push(match switch.as_ref() {
                        "GPSD" => FWFile {
                            kind: FWFileKind::GpsDb(file),
                            info: FileInfo::GpsDb(file),
                        },
                        "GASD" => FWFile {
                            kind: FWFileKind::GpsDbSecond(file),
                            info: FileInfo::GpsDb(file),
                        },
                        _ => unreachable!(),
                    });
                }
                "BLES" | "KEYS" | "LSRS" | "STUI" | "STDS" | "STGP" | "N2UI" | "N2DS" | "N3DS"
                | "N2GP" | "N3GP" => {
                    let length_modifier = if switch == "BLES" { 1024 } else { 512 };
                    let length = (current_length / length_modifier + 1) * length_modifier;
                    let expected_termstr = format!("DRSW{}", &switch[0..3]);

                    let (offset, version, term_str) = parse_file_basic(&mut cursor, length)?;

                    if expected_termstr != term_str {
                        panic!("Wrong termination sequence: {}", switch)
                    }

                    let file = FileInfoBase {
                        length,
                        offset,
                        version,
                    };

                    files.push(match switch.as_ref() {
                        "BLES" => FWFile {
                            kind: FWFileKind::Ble(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "KEYS" => FWFile {
                            kind: FWFileKind::Keypad(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "LSRS" => FWFile {
                            kind: FWFileKind::LaserIf(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "STUI" => FWFile {
                            kind: FWFileKind::UiStm(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "STDS" => FWFile {
                            kind: FWFileKind::DspStm(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "STGP" => FWFile {
                            kind: FWFileKind::GpsStm(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "N2UI" => FWFile {
                            kind: FWFileKind::UiNu2(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "N2DS" => FWFile {
                            kind: FWFileKind::DspNu2(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "N3DS" => FWFile {
                            kind: FWFileKind::DspNu3(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "N2GP" => FWFile {
                            kind: FWFileKind::GpsNu2(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "N3GP" => FWFile {
                            kind: FWFileKind::GpsNu3(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        _ => unreachable!(),
                    });
                }
                "STSD" | "SUSD" => {
                    let expected_termstr = format!("DRSW{}", &switch[0..3]);
                    cursor.seek(current_length as u64 - 12);

                    let arr = cursor.read_n(12)?;
                    let vbuf = decode_old_model(SOUND_DB_KEY, &arr, 0, 4);
                    let version = rd_version(i32::from_le_bytes(vbuf.try_into().unwrap()) as i16);

                    if switch == "SUSD" {
                        cursor.seek(2);
                    }

                    let termstr = stfu!(cursor.read_n(7)?);
                    if expected_termstr != termstr {
                        panic!("Wrong termination sequence: {}", switch)
                    }

                    let file = FileInfoBase {
                        length: current_length,
                        offset: current_offset as i32,
                        version: version as i32,
                    };

                    files.push(match switch.as_ref() {
                        "STSD" => FWFile {
                            kind: FWFileKind::SoundDbla1(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        "SUSD" => FWFile {
                            kind: FWFileKind::SoundDbla2(FileInfo::Base(file)),
                            info: FileInfo::Base(file),
                        },
                        _ => unreachable!(),
                    });
                }
                "NMGF" => {
                    if cursor.position() == cursor.get_ref().len() as u64 {
                        metadata.new_merge_file = true;
                        metadata.format_version = i32::from_le_bytes(arr[8..12].try_into().unwrap())
                    }
                }
                _ => {
                    if switch[2..4].to_string() == "SD" {
                        cursor.seek(current_length as u64 + 9);
                    } else {
                        cursor.seek(alter_length(current_length) as u64 + 9);
                    }
                }
            }
        }

        self.files = files;
        self.metadata = Some(metadata);

        Ok(())
    }

    pub fn extract_to(&self, directory: &path::Path) {
        for file in &self.files {
            // let content = &self.buffer[]
            let mut fpath = path::PathBuf::from(directory);
            fpath.push(file.kind.to_file_name());
            let mut f = fs::File::create(&fpath)
                .unwrap_or_else(|_| panic!("Couldn't create output file: {}", fpath.display()));
            if let FileInfo::Base(fib) = file.info {
                f.write_all(
                    &self.buffer[fib.offset as usize..fib.offset as usize + fib.length as usize],
                )
                .unwrap_or_else(|_| panic!("Couldn't write output file: {}", fpath.display()));
            } else if let FileInfo::GpsDb(figps) = file.info {
                f.write_all(
                    &self.buffer[figps.info.offset as usize
                        ..figps.info.offset as usize + figps.info.length as usize],
                )
                .unwrap_or_else(|_| panic!("Couldn't write output file: {}", fpath.display()));
            }
        }
    }
}
