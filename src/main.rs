#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]
mod helpers;
use colorize::AnsiColor;
use inquire::InquireError;
use std::io::{self, BufRead, Write};
use std::{fs, path, process};
// fn main() -> io::Result<()> {
//     let file = fs::File::open("top-left_Overture PLA+ Pro - Grey Blue_1h59m-OS.gcode")?;
//     let reader = io::BufReader::new(file);

//     for line in reader.lines() {
//         let line = line?;
//         println!("{}", line);
//     }

//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = fs::create_dir_all(path::Path::new(helpers::GCODE_DIR)) {
        eprintln!("Error creating directory '{}': {}", helpers::GCODE_DIR, e);
    }
    let gcodes_list: Vec<path::PathBuf> =
        helpers::get_gcode_files().expect("Failed to get gcode list");
    let gcodes_list: Vec<String> = gcodes_list
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    let response = match helpers::ask_user(gcodes_list) {
        Ok(choice) => choice,
        Err(InquireError::OperationCanceled) => {
            println!("{}😀", "Goodbye! ".blue());
            process::exit(1);
        }
        Err(err) => {
            eprintln!("❌ {}", err);
            process::exit(1);
        }
    };
    println!("{}", format!("{:#?}", response).yellow());

    let adjustment_code_line = format!(
        "SET_GCODE_OFFSET Z_ADJUST={} MOVE=1",
        response.z_offset_signed()
    );

    let file = fs::File::open(&response.filename).expect("Failed to open file");
    let reader = io::BufReader::new(file);

    let out_file = fs::File::create(response.get_output_filename())?;
    let mut writer = io::BufWriter::new(out_file);

    let insert_at = 4u8;
    let mut locations: Vec<u32> = vec![];
    let mut count = 0i32;
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        writeln!(writer, "{}", line);
        // writeln!(writer, "{}", adjustment_code_line);
        // println!("{:?}", line);
        if line.trim() == ";LAYER_CHANGE" && count == 0 {
            count += 1;
            locations.push(i.try_into().unwrap());
        }
    }
    println!("{}", count);
    println!("locations: {:#?}", locations);
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_revert_at_layer_height_4_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            4,
        );
        assert_eq!(params.revert_z_offset_at_height(), 0.86);
    }

    #[test]
    fn test_revert_at_layer_height_6_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            6,
        );
        assert_eq!(params.revert_z_offset_at_height(), 1.26);
    }

    #[test]
    fn test_revert_at_layer_height_2_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            2,
        );
        assert_eq!(params.revert_z_offset_at_height(), 0.46);
    }
}
