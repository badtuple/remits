use std::fs::File;
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

    data_file: File,
    index_file: File,
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
            data_file: File::open(dat_path).unwrap(),
            index_file: File::open(idx_path).unwrap(),
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
            data_file: File::create(dat_path).unwrap(),
            index_file: File::create(idx_path).unwrap(),
        }
    }
}

//struct DataFile {}
//struct IndexFile {}

#[repr(u8)]
#[derive(Debug)]
enum FormatVersion {
    Uncompressed = 0x00,
}
