mod helpers;
use colorize::AnsiColor;
use inquire::InquireError;

// use std::io::{self, BufRead};
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

    let response = helpers::ask_user(gcodes_list);
    match response {
        Ok(choice) => {
            // if choice == "Exit" {
            //     exit_goodbye()
            // }

            let output_message = format!("{:?}", choice);
            println!("{}", output_message.yellow());
        }
        Err(err) => match err {
            InquireError::OperationCanceled => exit_goodbye(),
            _ => {
                println!("{}", err);
                process::exit(1);
            }
        },
    }
}

fn exit_goodbye() {
    println!("{}😀", "Goodbye! ".blue());
    process::exit(1);
}
