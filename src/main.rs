#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]
mod helpers;
mod tests;
use colorize::AnsiColor;
use inquire::InquireError;
use std::io::{self, BufRead, Write};
use std::{fs, path, process};

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

    let file = fs::File::open(&response.filename).expect("Failed to open file");
    let reader = io::BufReader::new(file);

    let out_file = fs::File::create(response.get_output_filename())?;
    let mut writer = io::BufWriter::new(out_file);

    struct GCode;
    impl GCode {
        const LAYER_CHANGE: &'static str = ";LAYER_CHANGE";
        const CURRENT_PRINT_HEIGHT: &'static str = ";Z:"; // NOTE the print is now at this height
        const CURRENT_LAYER_HEIGHT: &'static str = ";HEIGHT:"; // NOTE print height of the current layer
    }
    // first layer height: 0.26
    // layer height: 0.10
    // NOTE first layer
    // ;LAYER_CHANGE
    // ;Z:0.26
    // ;HEIGHT:0.26
    // NOTE third layer
    // ;LAYER_CHANGE
    // ;Z:0.5
    // ;HEIGHT:0.12

    let insert_at = 2u8;
    let mut insert_at_line: Vec<u32> = vec![];
    let mut first_gcode_insertion = false;

    let mut was_as_layer_change = false;
    let mut was_as_current_z = false;

    let mut locations: Vec<u32> = vec![];
    let mut layer_change_counter = 0u32;

    let mut placeholder_count = 0u32;

    let mut capture_current_print_height: f32 = 0.0;
    let mut capture_current_layer_height: f32 = 0.0;
    for (current_line_position, line) in reader.lines().enumerate() {
        let line = line?;

        // NOTE just count for verification purposes; can be deleted
        if line.trim() == GCode::LAYER_CHANGE {
            layer_change_counter += 1;
            locations.push(current_line_position.try_into().unwrap());
        }

        writeln!(writer, "{}", line);

        if line.trim() == GCode::LAYER_CHANGE {
            was_as_layer_change = true;
            // writeln!(writer, "{}", response.adjust_z_offset_code());
        }
        if line.contains(GCode::CURRENT_PRINT_HEIGHT) && was_as_layer_change {
            was_as_current_z = true;
            if let Some((_, value)) = line.split_once(':') {
                capture_current_print_height = value.trim().parse().unwrap();
                println!(
                    "capture_current_print_height: {}",
                    capture_current_print_height
                );
            }
        }

        if line.contains(GCode::CURRENT_LAYER_HEIGHT) && was_as_layer_change && was_as_current_z {
            was_as_layer_change = false;
            was_as_current_z = false;

            if let Some((_, value)) = line.split_once(':') {
                capture_current_layer_height = value.trim().parse().unwrap();
                println!(
                    "capture_current_layer_height: {}",
                    capture_current_layer_height
                );
            }
            if !first_gcode_insertion {
                first_gcode_insertion = true;
                writeln!(writer, "{}", response.adjust_z_offset_code());
                println!("{}", "write adjustment!".green());
            } else {
                if capture_current_print_height == response.revert_z_offset_at_height() {
                    writeln!(writer, "{}", response.revert_z_offset_code());
                    println!("{}", "write revertion!".green());
                }
            }
            // let v: u32 = current_line_position.try_into().unwrap();
            // insert_at_line.push(v + 1);
        }

        // NOTE for verification
        // if line.contains(";Z:") {
        //     if let Some((_, value)) = line.split_once(':') {
        //         let z: f32 = value.trim().parse().unwrap();
        //         println!("{}", z);
        //     }
        // }
    }
    println!("{}", layer_change_counter);
    println!("locations: {:?}", locations);
    Ok(())
}
