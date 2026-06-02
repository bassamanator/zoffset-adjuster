mod helpers;
mod tests;
use clap::Parser;
use colorize::AnsiColor;
use env_logger::Env;
use inquire::InquireError;
use std::io::{self, BufRead, Write};
use std::{fs, path, process};

#[derive(Parser, Debug)]
#[command(allow_negative_numbers = true)]
#[command(name = "zoffset-adjuster")]
#[command(about = "Adjusts the `z_offset` in gcode files for early layers.")]
#[command(
    long_about = "Adjusts the `z_offset` in gcode files for early layers. E.g., if you prefer more\nlayer squish for the first layer, and then normal layer squish for subsequent layers.

This is especially useful for users with a warped bed, or with a bed that has poor layer\nsquish, or adhesion, in particular spots.

Negative (-) values lower the nozzle, reducing the gap between the nozzle and the bed,\nresulting in `more` layer squish.
Positive (+) values elevate the nozzle, increasing the gap between the nozzle and the bed,\nresulting in `less` layer squish.
"
)]
struct Args {
    /// Path to gcode file
    #[arg(index = 1)]
    file: Option<String>,

    /// Path to gcode file
    #[arg(short, long)]
    input: Option<String>,

    /// Silent mode (no prompts, uses defaults or provided CLI args)
    #[arg(short, long)]
    silent: bool,

    /// Z offset adjustment
    #[arg(short, long)]
    z_offset: Option<f32>,

    /// First layer height
    // #[arg(short, long, value_parser = positive_float )]
    #[arg(short, long)]
    first_layer_height: Option<f32>,

    /// Layer height
    #[arg(short, long)]
    layer_height: Option<f32>,

    /// Revert z_offset at layer
    #[arg(short, long)]
    revert_z_offset_at_layer: Option<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("RUST_LOG", "off")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let args = Args::parse();

    let mut cli_args = validate_args(&args);

    if args.silent {
        if cli_args.filename.is_none() {
            println!("❌ Silent mode requires a gcode file to be provided via `--input`.");
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        };
    } else {
        let mut gcodes_list: Vec<String> = vec![];

        if let Some(filename) = &cli_args.filename {
            gcodes_list.push(filename.clone());
        } else {
            gcodes_list = helpers::get_gcode_files().expect("Failed to get gcode list");
        }

        if gcodes_list.len() == 0 {
            println!(
                "{}\n{}",
                "❌ No gcode file provided and none were found in the current directory".red(),
                "🗒️ Run this program with `--help` for instructions.".yellow(),
            );
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        }
        cli_args = match helpers::ask_user(gcodes_list, &cli_args) {
            Ok(choice) => choice,
            Err(InquireError::OperationCanceled) => {
                println!("{}😀", "Goodbye! ".blue());
                process::exit(0);
            }
            Err(err) => {
                eprintln!("❌ {}", err);
                process::exit(1);
            }
        };
    }

    adjust_gcode(cli_args)?;
    Ok(())
}

fn adjust_gcode(
    response: helpers::ZOffsetAdjustmentParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(&response.filename.as_ref().expect("Filename is required"))
        .expect("Failed to open file");
    let reader = io::BufReader::new(file);

    let out_path = response.get_output_filename();
    let out_file = fs::File::create(&out_path)?;
    let mut writer = io::BufWriter::new(out_file);

    struct GCode;
    impl GCode {
        const LAYER_CHANGE: &'static str = ";LAYER_CHANGE";
        const CURRENT_PRINT_HEIGHT: &'static str = ";Z:";
        const CURRENT_LAYER_HEIGHT: &'static str = ";HEIGHT:";
    }

    let mut first_gcode_insertion = false;
    let mut second_gcode_insertion = false;

    let mut was_as_layer_change = false;
    let mut was_as_current_z = false;

    let mut capture_current_print_height: f32 = 0.0;
    let mut layer_counter: u32 = 0;

    for (current_line_position, line) in reader.lines().enumerate() {
        let inserting_cmd_at_line = current_line_position + 1 + 1;
        let inserting_2nd_cmd_at_line = current_line_position + 1 + 1 + 1;
        let line = line?;

        let _ = writeln!(writer, "{}", line);

        // NOTE first cue
        if line.trim() == GCode::LAYER_CHANGE {
            layer_counter += 1;
            was_as_layer_change = true;
        }

        // NOTE second cue
        if line.contains(GCode::CURRENT_PRINT_HEIGHT) && was_as_layer_change {
            was_as_current_z = true;
            if let Some((_, value)) = line.split_once(':') {
                capture_current_print_height = value.trim().parse().unwrap();
            }
        }

        // NOTE third cue
        if line.contains(GCode::CURRENT_LAYER_HEIGHT) && was_as_layer_change && was_as_current_z {
            was_as_layer_change = false;
            was_as_current_z = false;

            if !first_gcode_insertion {
                if capture_current_print_height == response.first_layer_height {
                    first_gcode_insertion = true;
                    let _ = writeln!(writer, "{}", response.adjust_z_offset_code());
                    println!(
                        "\n{} {}",
                        "Inserting z_offset adjustment at line".italic().b_magenta(),
                        inserting_cmd_at_line
                    );
                }
            } else {
                if capture_current_print_height == response.revert_z_offset_at_height()
                    && layer_counter == response.revert_z_offset_at_layer
                {
                    second_gcode_insertion = true;
                    let _ = writeln!(writer, "{}", response.revert_z_offset_code());
                    println!(
                        "{} {}",
                        "Inserting z_offset reversion at line".italic().b_magenta(),
                        inserting_2nd_cmd_at_line
                    );
                }
            }
        }
    }

    if !second_gcode_insertion || !first_gcode_insertion {
        drop(writer);
        let new_path = out_path.replace(".gcode", "-DO-NOT-USE.gcode");
        fs::rename(&out_path, &new_path)?;
        println!(
            "\n🚨 {}❌ {} {}\n{}",
            "The z_offset adjustment and, or, reversion entry was never added.\n".red(),
            "Do not use the generated gcode!".red(),
            new_path.b_blue(),
            "🤔 The inputs were likely incorrect, double check them and try again.".yellow()
        );
    } else {
        println!("\n{} {}", out_path.b_blue(), "generated!".cyan());
        println!("{}😀", "Goodbye! ".green());
    }
    Ok(())
}

fn validate_args(args: &Args) -> helpers::ZOffsetAdjustmentParams {
    let file = &args.input.clone().or(args.file.clone());
    let file: Option<String> = match file {
        Some(f) => {
            let p = path::Path::new(&f);
            if !(p.is_file()
                && p.extension().and_then(|ext| ext.to_str()) == Some(helpers::GCODE_EXT))
            {
                println!("❌ Invalid input. Only `.gcode` files are permissible.");
                process::exit(0)
            }
            Some(f.to_string())
        }
        None => None,
    };
    if let Some(value) = args.z_offset {
        if value < helpers::Z_OFFSET_MIN || value > helpers::Z_OFFSET_MAX {
            println!(
                "❌ Invalid z_offset value. Must be between {} and {}.",
                helpers::Z_OFFSET_MIN,
                helpers::Z_OFFSET_MAX
            );
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        }
    }
    if let Some(value) = args.first_layer_height {
        if value < helpers::LAYER_HEIGHT_MIN || value > helpers::LAYER_HEIGHT_MAX {
            println!(
                "❌ Invalid first_layer_height value. Must be between {} and {}.",
                helpers::LAYER_HEIGHT_MIN,
                helpers::LAYER_HEIGHT_MAX
            );
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        }
    }
    if let Some(value) = args.layer_height {
        if value < helpers::LAYER_HEIGHT_MIN || value > helpers::LAYER_HEIGHT_MAX {
            println!(
                "❌ Invalid layer_height value. Must be between {} and {}.",
                helpers::LAYER_HEIGHT_MIN,
                helpers::LAYER_HEIGHT_MAX
            );
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        }
    }
    if let Some(value) = args.revert_z_offset_at_layer {
        if value < 2 {
            println!(
                "❌ Invalid revert_z_offset_at_layer value. Must be an integer greater than or equal to 2."
            );
            println!("{}😀", "Goodbye! ".green());
            process::exit(0)
        }
    }

    let settings = helpers::load_settings();

    let response = helpers::ZOffsetAdjustmentParams {
        filename: file,
        z_offset: args.z_offset.unwrap_or(settings.z_offset),
        first_layer_height: args
            .first_layer_height
            .unwrap_or(settings.first_layer_height),
        layer_height: args.layer_height.unwrap_or(settings.layer_height),
        revert_z_offset_at_layer: args
            .revert_z_offset_at_layer
            .unwrap_or(settings.revert_z_offset_at_layer),
    };
    response
}
