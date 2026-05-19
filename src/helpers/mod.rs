// use serde::{Deserialize, Serialize};
use std::{error::Error, fs, io, path};

pub const GCODE_DIR: &str = "./logs";
pub const GCODE_EXT: &str = "gcode";

pub fn get_logs() -> Result<Vec<path::PathBuf>, io::Error> {
    // if path::
    // fs::create_dir_all(GCODE_DIR)?; // Ensure the directory exists

    let log_files = fs::read_dir(GCODE_DIR)?
        .filter_map(|result| {
            // Map over directory entries, returning None if there's an error
            result.ok().and_then(|e| {
                let path = e.path();
                // Only include files, files with a .gcode extension
                if path.is_file()
                    && path.extension().and_then(|ext| ext.to_str()) == Some(GCODE_EXT)
                {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    Ok(log_files)
}
