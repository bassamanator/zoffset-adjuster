// use serde::{Deserialize, Serialize};
use std::{fs, io, path};

pub const GCODE_DIR: &str = "./gcode";
pub const GCODE_EXT: &str = "gcode";
pub fn get_gcode_files() -> Result<Vec<path::PathBuf>, io::Error> {
    let gcode_files = fs::read_dir(GCODE_DIR)?
        .filter_map(|result| {
            // Map over directory entries, returning None if there's an error
            result.ok().and_then(|e| {
                let path = e.path();
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
    Ok(gcode_files)
}
