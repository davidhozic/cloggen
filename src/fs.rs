//! Module related to files

use std::io::{self, Read};
use std::path::Path;
use std::fs::File;
use encoding_rs;


/// Encoding aware file read.
pub fn read_file_universal<T: AsRef<Path>>(filepath: T) -> io::Result<String> {
    let mut file = File::open(filepath)?;
    let fdata = {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        match encoding_rs::Encoding::for_bom(&buf) {
            Some((encoding, _)) => {
                let (decoded, _, _) = encoding.decode(&buf);  // Decode based on BOM (byte order mark)
                decoded.to_string()
            }
            None => String::from_utf8(buf).unwrap()  // BOM not found, assume utf-8
        }
    };

    Ok(fdata)
}
