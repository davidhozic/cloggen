//! Module related to files

use std::io::{self, Read};
use std::path::Path;
use std::fs::File;
use encoding_rs;


/// Encoding aware file read.
/// This function reads the file at ``filepath`` and then tries to decode it,
/// assuming multiple possible encodings.
pub fn read_file_universal<T: AsRef<Path>>(filepath: T) -> io::Result<String> {
    let filepath = filepath.as_ref();
    let mut file = File::open(filepath)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    match encoding_rs::Encoding::for_bom(&buf) {
        Some((encoding, _)) => {
            let (decoded, _, _) = encoding.decode(&buf);  // Decode based on BOM (byte order mark)
            return Ok(decoded.to_string());
        }
        None => {
            // Encoding could not be determined through byte-order-marker
            // => Try to decode with various standards.
            let mut decoded = None;
            for encoding in ENCODINGS {
                // It's a bit inefficient to decode the entire file in the presence decoding errors,
                // but currently the ``encoding_rs`` library lacks early stopping.
                let (new, _, errors) = encoding.decode(&buf);
                if !errors {
                    decoded = Some(new);
                    break;
                }
            }

            // Returned the decoded data if found, else InvalidData error.
            if let Some(data) = decoded {
                return Ok(data.to_string());
            }
            else {
                return Err(io::Error::from(io::ErrorKind::InvalidData))
            }
        }
    }
}


/// Table of encodings to try (in order) when the BOM is not present. 
static ENCODINGS: [&'static encoding_rs::Encoding; 11] = [
    encoding_rs::UTF_8,
    encoding_rs::WINDOWS_1250,
    encoding_rs::WINDOWS_1251,
    encoding_rs::WINDOWS_1252,
    encoding_rs::WINDOWS_1253,
    encoding_rs::WINDOWS_1254,
    encoding_rs::WINDOWS_1255,
    encoding_rs::WINDOWS_1256,
    encoding_rs::WINDOWS_1257,
    encoding_rs::WINDOWS_1258,
    encoding_rs::WINDOWS_874,
];
