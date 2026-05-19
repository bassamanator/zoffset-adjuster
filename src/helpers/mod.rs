use inquire::{CustomType, InquireError, Select, Text, validator::Validation};

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

#[derive(Debug)]
pub struct ZOffsetAdjustmentParams {
    pub filename: String,
    pub z_offset: f32,
    pub first_layer_height: f32,
    pub layer_height: f32,
    pub revert_z_offset_at_layer: i32,
}

impl ZOffsetAdjustmentParams {
    pub fn new(
        filename: String,
        z_offset: f32,
        first_layer_height: f32,
        layer_height: f32,
        revert_z_offset_at_layer: i32,
    ) -> Self {
        Self {
            filename,
            z_offset,
            first_layer_height,
            layer_height,
            revert_z_offset_at_layer,
        }
    }

    pub fn z_offset_signed(&self) -> String {
        if self.z_offset >= 0.0 {
            format!("+{:.3}", self.z_offset)
        } else {
            format!("{:.3}", self.z_offset)
        }
    }
    pub fn get_output_filename(&self) -> String {
        let parts: Vec<&str> = self.filename.split(".gcode").collect();
        let new = format!("{}-{}.gcode", parts[0], get_timestamp(),);
        new
    }

    pub fn revert_z_offset_at_height(&self) -> f32 {
        let result = ((self.revert_z_offset_at_layer - 1) as f32) * self.layer_height
            + self.first_layer_height;
        (result * 1000.0).round() / 1000.0 // round to 3 decimal places
    }
}

pub fn ask_user(gcodes_list: Vec<String>) -> Result<ZOffsetAdjustmentParams, InquireError> {
    let menu_options: Vec<&str> = vec!["Select a gcode file to adjust", "Exit"];

    let filename = Select::new("Select a gcode file", gcodes_list).prompt()?;

    let z_offset_min: f32 = -0.400;
    let z_offset_max: f32 = 0.400;

    let z_offset = CustomType::<f32>::new("How much to adjust z_offset by?")
        .with_starting_input("-0.015")
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
    Ok(ZOffsetAdjustmentParams {
        filename,
        z_offset,
        first_layer_height,
        layer_height,
        revert_z_offset_at_layer: at_what_layer_to_revert_z_offset,
    })
}

fn get_timestamp() -> String {
    chrono::Local::now()
        .format("%I%M%S")
        .to_string()
        .to_uppercase()
}
