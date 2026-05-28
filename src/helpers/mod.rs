use colorize::AnsiColor;
use inquire::{CustomType, InquireError, Select, validator::Validation};
use log::warn;
use serde::{Deserialize, Serialize};
use std::{fs, io};

const SETTINGS_FILE: &str = "./settings.toml";
pub const GCODE_DIR: &str = "./";
pub const GCODE_EXT: &str = "gcode";
pub const Z_OFFSET_MIN: f32 = -0.400;
pub const Z_OFFSET_MAX: f32 = 0.400;
pub const LAYER_HEIGHT_MIN: f32 = 0.0;
pub const LAYER_HEIGHT_MAX: f32 = 0.5;

pub fn get_gcode_files() -> Result<Vec<String>, io::Error> {
    let gcode_files = fs::read_dir(GCODE_DIR)?
        .filter_map(|result| {
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
        .collect::<Vec<_>>()
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    Ok(gcode_files)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub z_offset: f32,
    pub first_layer_height: f32,
    pub layer_height: f32,
    pub revert_z_offset_at_layer: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            z_offset: -0.015,
            first_layer_height: 0.26,
            layer_height: 0.2,
            revert_z_offset_at_layer: 2,
        }
    }
}

pub fn load_settings() -> Settings {
    let content = match fs::read_to_string(SETTINGS_FILE) {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => {
            warn!(
                "{} exists but is empty — using default Settings.",
                SETTINGS_FILE
            );
            return Settings::default();
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                println!("Creating {} with default settings.", SETTINGS_FILE);

                if let Ok(serialized) = toml::to_string_pretty(&Settings::default()) {
                    let _ = fs::write(SETTINGS_FILE, serialized);
                }
                return Settings::default();
            }
            _ => {
                warn!(
                    "Failed to read {}: {} — using default Settings.",
                    SETTINGS_FILE, e
                );
                return Settings::default();
            }
        },
    };

    match toml::from_str::<Settings>(&content) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "Failed to parse {} as TOML, you should delete it. Using default settings.",
                SETTINGS_FILE
            );
            warn!("{}", e);
            Settings::default()
        }
    }
}

#[derive(Debug)]
pub struct ZOffsetAdjustmentParams {
    pub filename: String,
    pub z_offset: f32,
    pub first_layer_height: f32,
    pub layer_height: f32,
    pub revert_z_offset_at_layer: u32,
}

impl ZOffsetAdjustmentParams {
    pub fn new(
        filename: String,
        z_offset: f32,
        first_layer_height: f32,
        layer_height: f32,
        revert_z_offset_at_layer: u32,
    ) -> Self {
        Self {
            filename,
            z_offset,
            first_layer_height,
            layer_height,
            revert_z_offset_at_layer,
        }
    }

    fn z_offset_signed(&self) -> String {
        if self.z_offset >= 0.0 {
            format!("+{:.3}", self.z_offset)
        } else {
            format!("{:.3}", self.z_offset)
        }
    }

    fn get_signed(&self, value: f32) -> String {
        if value >= 0.0 {
            format!("+{:.3}", value)
        } else {
            format!("{:.3}", value)
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

    pub fn adjust_z_offset_code(&self) -> String {
        format!(
            "SET_GCODE_OFFSET Z_ADJUST={} MOVE=1",
            self.z_offset_signed()
        )
    }

    pub fn revert_z_offset_code(&self) -> String {
        format!(
            "SET_GCODE_OFFSET Z_ADJUST={} MOVE=1",
            self.get_signed(self.z_offset * (-1 as f32))
        )
    }
}

pub fn ask_user(gcodes_list: Vec<String>) -> Result<ZOffsetAdjustmentParams, InquireError> {
    let settings = load_settings();
    // println!("settings: {:#?}", settings);

    let filename = if gcodes_list.len() == 1 {
        println!(
            "{} {} {}",
            format!(">").faint(),
            format!("Selected file:"),
            format!("{}", gcodes_list[0]).cyan()
        );
        gcodes_list[0].clone()
    } else {
        Select::new("Select a gcode file", gcodes_list).prompt()?
    };

    let z_offset = CustomType::<f32>::new("How much to adjust z_offset by?")
        .with_starting_input(&settings.z_offset.to_string())
        // .with_starting_input("-0.015")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("Range: -0.400 to +0.400.\n E.g., +0.01, 0.012, -0.015, etc.\n Note: negative values lower the nozzle.")
        .with_validator(move |val: &f32| {
            if *val < Z_OFFSET_MIN {
                Ok(Validation::Invalid(format!("Value must be greater than {Z_OFFSET_MIN}").into()))
            } else if *val > Z_OFFSET_MAX {
                Ok(Validation::Invalid(format!("Value must be less than {Z_OFFSET_MAX}").into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let first_layer_height = CustomType::<f32>::new("What is the height of the first layer?")
        .with_starting_input(&settings.first_layer_height.to_string())
        // .with_starting_input("0.2")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("E.g., 0.2, 0.26, 0.3, etc.")
        .with_validator(move |val: &f32| {
            if *val > LAYER_HEIGHT_MAX {
                Ok(Validation::Invalid(
                    format!("Value must be less than {LAYER_HEIGHT_MAX}").into(),
                ))
            } else if *val < LAYER_HEIGHT_MIN {
                Ok(Validation::Invalid(
                    format!("Value must be greater than {LAYER_HEIGHT_MIN} mm").into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let layer_height = CustomType::<f32>::new("What is the height of the other layers?")
        .with_starting_input(&settings.layer_height.to_string())
        // .with_starting_input("0.2")
        .with_formatter(&|i| format!("{i:.3} mm"))
        .with_error_message("Please type a valid number")
        .with_help_message("E.g., 0.2, 0.26, 0.3, etc.")
        .with_validator(move |val: &f32| {
            if *val > LAYER_HEIGHT_MAX {
                Ok(Validation::Invalid(
                    format!("Value must be less than {LAYER_HEIGHT_MAX}").into(),
                ))
            } else if *val < LAYER_HEIGHT_MIN {
                Ok(Validation::Invalid(
                    format!("Value must be greater than {LAYER_HEIGHT_MIN} mm").into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let at_what_layer_to_revert_z_offset = CustomType::<u32>::new(
        "At the start of what layer do you want to undo the Z offset adjustment?",
    )
    .with_starting_input(&settings.revert_z_offset_at_layer.to_string())
    // .with_starting_input("2")
    .with_formatter(&|i| format!("{i}"))
    .with_error_message("Please type a valid integer")
    .with_help_message("Enter an integer value greater than 2")
    .with_validator(|val: &u32| {
        if *val < 2 {
            Ok(Validation::Invalid("Value must be 2 or greater".into()))
        } else {
            Ok(Validation::Valid)
        }
    })
    .prompt()?;

    Ok(ZOffsetAdjustmentParams::new(
        filename,
        z_offset,
        first_layer_height,
        layer_height,
        at_what_layer_to_revert_z_offset,
    ))
}

fn get_timestamp() -> String {
    chrono::Local::now()
        .format("%I%M%S")
        .to_string()
        .to_uppercase()
}
