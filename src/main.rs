mod helpers;
mod tests;
use clap::Parser;
use colorize::AnsiColor;
use inquire::InquireError;
use std::io::{self, BufRead, Write};
use std::{fs, path, process};

#[derive(Parser)]
#[command(name = "zoffset-adjuster")]
#[command(about = "Adjusts z offset in gcode files for early layers.")]
#[command(
    long_about = "Adjusts z offset in gcode files for early layers. E.g., if you prefer more\nlayer squish for the first layer, and then normal layer squish for subsequent\nlayers. This is especially useful for users with a warped bed, or with a bed\nthat poor layer squish in particular spots consistently."
)]
struct Args {
    /// Path to gcode file
    #[arg(index = 1)]
    file: Option<String>,

    /// Path to gcode file
    #[arg(short, long)]
    input: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = args.input.or(args.file);

    let file: Option<std::path::PathBuf> = match file {
        Some(f) => {
            let p = path::Path::new(&f);
            if !(p.is_file()
                && p.extension().and_then(|ext| ext.to_str()) == Some(helpers::GCODE_EXT))
            {
                println!("❌ Invalid input. Only `.gcode` files are permissible.");
                process::exit(0)
            }
            Some(p.to_path_buf())
        }
        None => None,
    };

    // let mut response: helpers::ZOffsetAdjustmentParams;
    let mut gcodes_list: Vec<String> = vec![];

    if let Some(file) = &file {
        gcodes_list.push(file.display().to_string());
    } else {
        gcodes_list = helpers::get_gcode_files().expect("Failed to get gcode list");
    }

    let response: helpers::ZOffsetAdjustmentParams = match helpers::ask_user(gcodes_list) {
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

    let file = fs::File::open(&response.filename).expect("Failed to open file");
    let reader = io::BufReader::new(file);

    let out_path = response.get_output_filename();
    let out_file = fs::File::create(&out_path)?;
    // let out_file = fs::File::create(response.get_output_filename())?;
    let mut writer = io::BufWriter::new(out_file);

    struct GCode;
    impl GCode {
        const LAYER_CHANGE: &'static str = ";LAYER_CHANGE";
        const CURRENT_PRINT_HEIGHT: &'static str = ";Z:";
        const CURRENT_LAYER_HEIGHT: &'static str = ";HEIGHT:";
    }
    // NOTE example sequence found in OS 2.3.2
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

    let mut first_gcode_insertion = false;
    let mut second_gcode_insertion = false;

    let mut was_as_layer_change = false;
    let mut was_as_current_z = false;

    let mut layer_change_counter = 0u32;

    let mut capture_current_print_height: f32 = 0.0;

    for (current_line_position, line) in reader.lines().enumerate() {
        let line = line?;

        if line.trim() == GCode::LAYER_CHANGE {
            layer_change_counter += 1;
        }

        let _ = writeln!(writer, "{}", line);

        // NOTE first cue
        if line.trim() == GCode::LAYER_CHANGE {
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
                first_gcode_insertion = true;
                let _ = writeln!(writer, "{}", response.adjust_z_offset_code());
                println!(
                    "\n{} {}",
                    "Inserting z_offset adjustment at line".italic().b_magenta(),
                    current_line_position + 1 + 1
                );
            } else {
                if capture_current_print_height == response.revert_z_offset_at_height() {
                    second_gcode_insertion = true;
                    let _ = writeln!(writer, "{}", response.revert_z_offset_code());
                    println!(
                        "{} {}",
                        "Inserting z_offset reversion at line".italic().b_magenta(),
                        current_line_position + 1 + 1
                    );
                }
            }
        }
    }

    println!(
        "{}",
        format!("There were {} layer changes", layer_change_counter).magenta()
    );

    if !second_gcode_insertion || !first_gcode_insertion {
        println!(
            "\n🚨 {}❌ {}{}",
            "The z_offset adjustment and, or, reversion entry was never added.\n".red(),
            "Do not use the generated gcode!\n".red(),
            "⏰ The inputs were likely incorrect, try again.".yellow()
        );
        drop(writer);
        let new_path = out_path.replace(".gcode", "-DO-NOT-USE.gcode");
        fs::rename(&out_path, &new_path)?;
    } else {
        println!("\n{} {}", out_path.b_blue(), "generated!".cyan());
        println!("{}😀", "Goodbye! ".green());
    }
    Ok(())
}
