mod helpers;
use std::io::{self, BufRead};
use std::{fs, path};

use inquire::InquireError;
use inquire::{
    CustomType, MultiSelect, Select, Text,
    error::{CustomUserError, InquireResult},
    required,
    ui::RenderConfig,
};

// fn main() -> io::Result<()> {
//     let file = fs::File::open("top-left_Overture PLA+ Pro - Grey Blue_1h59m-OS.gcode")?;
//     let reader = io::BufReader::new(file);

//     for line in reader.lines() {
//         let line = line?;
//         println!("{}", line);
//     }

//     Ok(())
// }

fn main() {
    if let Err(e) = fs::create_dir_all(path::Path::new(helpers::GCODE_DIR)) {
        eprintln!("Error creating directory '{}': {}", helpers::GCODE_DIR, e);
    }
    let gcodes_list: Vec<path::PathBuf> =
        helpers::get_gcode_files().expect("Failed to get gcode list");
    let gcodes_list: Vec<String> = gcodes_list
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    println!("gcodes: {:?}", gcodes_list);

    let selected_file = Select::new("Select a gcode file", gcodes_list).prompt();

    match selected_file {
        Ok(file) => println!("Selected: {:?}", file),
        Err(_) => println!("There was an error or the user canceled"),
    }
}
