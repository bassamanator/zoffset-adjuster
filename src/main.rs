mod helpers;
use colorize::AnsiColor;
use inquire::validator::Validation;
use std::io::{self, BufRead};
use std::{fs, path, process};

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

    let response = ask_user(gcodes_list);
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

#[derive(Debug)]
struct ZOffsetResult {
    filename: String,
    z_offset: f32,
    first_layer_height: f32,
    layer_height: f32,
    revert_z_offset_at_layer: i32,
}

fn ask_user(gcodes_list: Vec<String>) -> Result<ZOffsetResult, InquireError> {
    let menu_options: Vec<&str> = vec!["Select a gcode file to adjust", "Exit"];

    // let response: Result<&str, InquireError> =
    //     Ok(inquire::Select::new("What would you like to do?", menu_options).prompt()?);

    let filename = Select::new("Select a gcode file", gcodes_list).prompt()?;

    let z_offset_min: f32 = -0.400;
    let z_offset_max: f32 = 0.400;

    let z_offset = CustomType::<f32>::new("How much to adjust z_offset by?")
        .with_starting_input("0.015")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("Range: -0.400 to +0.400.\n E.g., +0.01, 0.012, -0.015, etc.\n Note: negative values lower the nozzle.")
        .with_validator(move |val: &f32| {
            if *val < z_offset_min {
                Ok(Validation::Invalid(format!("Value must be greater than {z_offset_min}").into()))
            } else if *val > z_offset_max {
                Ok(Validation::Invalid(format!("Value must be less than {z_offset_max}").into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let layer_height_max: f32 = 0.5;
    let layer_height_min: f32 = 0.0;

    let first_layer_height = CustomType::<f32>::new("What is the height of the first layer?")
        .with_starting_input("0.2")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("E.g., 0.2, 0.26, 0.3, etc.")
        .with_validator(move |val: &f32| {
            if *val > layer_height_max {
                Ok(Validation::Invalid(
                    format!("Value must be less than {layer_height_max}").into(),
                ))
            } else if *val < layer_height_min {
                Ok(Validation::Invalid(
                    format!("Value must be greater than {layer_height_min} mm").into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let layer_height = CustomType::<f32>::new("What is the height of the other layers?")
        .with_starting_input("0.2")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("E.g., 0.2, 0.26, 0.3, etc.")
        .with_validator(move |val: &f32| {
            if *val > layer_height_max {
                Ok(Validation::Invalid(
                    format!("Value must be less than {layer_height_max}").into(),
                ))
            } else if *val < layer_height_min {
                Ok(Validation::Invalid(
                    format!("Value must be greater than {layer_height_min} mm").into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let at_what_layer_to_revert_z_offset = CustomType::<i32>::new(
        "At the start of what layer do you want to undo the Z offset adjustment?",
    )
    .with_starting_input("2")
    .with_formatter(&|i| format!("{i}"))
    .with_error_message("Please type a valid integer")
    .with_help_message("Enter an integer 2 or greater")
    .with_validator(|val: &i32| {
        if *val < 2 {
            Ok(Validation::Invalid("Value must be 2 or greater".into()))
        } else {
            Ok(Validation::Valid)
        }
    })
    .prompt()?;

    // println!("Selected: {:?}", selected_file);
    // match selected_file {
    //     Ok(file) => println!("Selected: {:?}", file),
    //     Err(_) => println!("There was an error or the user canceled"),
    // }
    Ok(ZOffsetResult {
        filename,
        z_offset,
        first_layer_height,
        layer_height,
        revert_z_offset_at_layer: at_what_layer_to_revert_z_offset,
    })
}

fn exit_goodbye() {
    println!("{}😀", "Goodbye! ".blue());
    process::exit(1);
}
