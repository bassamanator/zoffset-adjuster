mod helpers;
use colorize::AnsiColor;
use inquire::InquireError;
use std::io::{self, BufRead};
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
    // response.
    let adjustment_code_line = format!(
        "SET_GCODE_OFFSET Z_ADJUST={} MOVE=1",
        response.z_offset_signed()
    );

    println!("adjustment_code_line: {:?}", adjustment_code_line);

    let file = fs::File::open(response.filename).expect("Failed to open file");
    let reader = io::BufReader::new(file);

    let mut locations: Vec<u32> = vec![];
    let mut count = 0i32;
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        // println!("{:?}", line);
        if line.contains(";LAYER_CHANGE") {
            count += 1;
            locations.push(i.try_into().unwrap());
        }
    }
    println!("{}", count);
    println!("locations: {:#?}", locations);
    Ok(())
}
