use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// MAX_SEGMENT_SIZE is the number of bytes a segment can contain before we have to roll over to a
/// new one.  Right now, this is non-configurable and is set to 1 GiB.
const MAX_SEGMENT_SIZE: usize = 1_073_741_824;

/// A Segment is a set of two files, a DataFile and an IndexFile.
/// The combination of these store all the data and where the
#[derive(Debug)]
pub struct Segment {
    /// The timestamp of the earliest message contained in the segment. This doubles as the name of
    /// the DataFile and IndexFile before by their respective extensions.
    timestamp: u64,
    format_version: FormatVersion,

    data_file: DataFile,
    index_file: IndexFile,
}

impl Segment {
    pub fn get_active_for(path: PathBuf) -> Segment {
        let mut files: Vec<String> = std::fs::read_dir(&path)
            .expect("could not read segment directory")
            .map(|entry| {
                entry
                    .expect("got error reading dir entry")
                    .file_name()
                    .into_string()
                    .expect("segment file contains non-utf8 string")
            })
            .filter(|entry| entry.split(".").last().unwrap() == "dat")
            .collect();

        // New log with no segments. Make a new one.
        if files.len() == 0 {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            return Segment::create(path, timestamp);
        }

        files.sort();

        let dat_path = files.first().unwrap();
        let timestamp: u64 = dat_path[0..dat_path.len() - 4].parse().unwrap();
        let mut idx_path = dat_path[0..dat_path.len() - 4].to_string();
        idx_path.push_str("idx");

        Segment {
            timestamp,
            format_version: FormatVersion::Uncompressed,
            data_file: DataFile {
                file: File::open(dat_path).unwrap(),
            },
            index_file: IndexFile {
                file: File::open(idx_path).unwrap(),
            },
        }
    }

    fn create(path: PathBuf, timestamp: u64) -> Segment {
        let dat = format!("{:020}.dat", timestamp);
        let idx = format!("{:020}.idx", timestamp);

        let mut dat_path = path.clone();
        dat_path.push(dat);

        let mut idx_path = path.clone();
        idx_path.push(idx);

        Segment {
            timestamp,
            format_version: FormatVersion::Uncompressed,
            data_file: DataFile::create(dat_path),
            index_file: IndexFile {
                file: File::create(idx_path).unwrap(),
            },
        }
    }

    fn add_msg(&self, msg: Vec<u8>) {
        //self.r
    }
}

// Magic numbers are derived from the order of the Monster Group.
// The Hex representation of the group is
//     86fa3f510644e13fdc4c5673c27c78c31400000000000
//  We can just take 32 bits at a time for each 32bit magic number.
//
//  DataFile  Header Number - 0x86FA3F51
//  IndexFile Header Number - 0x0644E13F

/// TODO: Add message to segment

// Format:
//
// The beginning of the Datafile has a header that includes:
//   - A 32bit magic number.
//   - An 8bit number corresponding to the FormatVersion of the Segment
//
// After the Header is a list of entries for each message.
// Each message entry contains:
//   - A 32bit CRC of the message payload.
//   - The variable byte message payload.
#[derive(Debug)]
struct DataFile {
    file: File,
}

impl DataFile {
    const MAGIC_NUMBER: &'static [u8] = &[0x86, 0xFA, 0x3F, 0x51];

    fn create(path: PathBuf) -> Self {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("could not create datafile");

        file.write_all(Self::MAGIC_NUMBER)
            .expect("could not write initial bytes to datafile");

        file.write_all(&(0x00 as u8).to_le_bytes());
        //.expect("could not write initial bytes to datafile");

        Self { file }
    }
}

// Format:
//
// The IndexFile begins with a header that includes:
//   - A 32bit magic number.
//   - A 64bit Unix timestamp in milliseconds representing the Segment's Epoch.
//
// After the Header is a list of entries for each messages
// Each message entry has:
//   - A 32bit integer representing the number of milliseconds since the Segment's Epoch.
//   - A 32bit integer representing the id of the message.
//   - A 32bit integer representing the byte offset of the message in the DataFile.
//
// The first is the offset, the second is the timestamp of ingestion, and the third is the byte
// position in the DataFile.
#[derive(Debug)]
struct IndexFile {
    file: File,
}

impl IndexFile {
    const MAGIC_NUMBER: &'static [u8] = &[0x06, 0x44, 0xE1, 0x3F];

    fn create(path: PathBuf, ts: u64) -> Self {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("could not create datafile");

        file.write_all(Self::MAGIC_NUMBER)
            .expect("could not write initial bytes to datafile");

        file.write_all(&ts.to_le_bytes())
            .expect("could not write initial bytes to datafile");

        Self { file }
    }
}

#[repr(u8)]
#[derive(Debug)]
enum FormatVersion {
    Uncompressed = 0x00,
}
