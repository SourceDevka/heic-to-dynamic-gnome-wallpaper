use anyhow::Result;
use colored::*;
use libheif_rs::HeifContext;
use std::path::Path;

use clap::{App, Arg};

mod image;
mod metadata;
mod schema;
mod serializer;
mod solar;
mod timebased;
mod util;

const INPUT: &str = "IMAGE";
const DIR: &str = "DIR";
const DAY_SECS: f32 = 86400.0;

fn main() -> Result<()> {
    let matches = App::new("heic-to-dynamic-gnome-wallpaper")
        .arg(Arg::with_name(INPUT)
             .help("Image which should be transformed")
             .takes_value(true)
             .value_name(INPUT)
             .required(true)
        )

.arg(Arg::with_name(DIR)
             .help("Into which directory the output images and schema should be written to.")
             .long_help("Specifies into which directory created images should be written to. Default is the parent directory of the given image.")
             .short("d")
             .long("dir")
             .takes_value(true)
             .value_name(DIR)
        )
        .get_matches();
    let path = matches
        .value_of(INPUT)
        .ok_or(anyhow::Error::msg("Could not read INPUT"))?;

    let parent_directory;
    if matches.is_present(DIR) {
        let nu_path = std::path::Path::new(matches.value_of(DIR).unwrap()).to_path_buf();
        if !nu_path.exists() {
            std::fs::create_dir_all(&nu_path)?
        }
        parent_directory = nu_path.canonicalize()?;
    } else {
        let p = std::path::Path::new(path)
            .ancestors()
            .nth(1);

        if p.is_none() {
            return Err(anyhow::Error::msg("Cannot get parent of given image path."));
        }
        let pd = p.unwrap()
            .canonicalize();

        if let Err(e) = pd {
            let msg =  format!("Cannot get absolute path: {}", e);
            return Err(anyhow::Error::msg(msg));
        }
        parent_directory = pd.unwrap();
    }
    let image_ctx_opt = HeifContext::read_from_file(path);

    if let Err(e) = image_ctx_opt {
        let msg = format!("{}", e.message);
        return Err(anyhow::Error::msg(msg))
    }
    let image_ctx = image_ctx_opt.unwrap();

    // FETCH file wide metadata
    println!(
        "{}: {}",
        "Preparation".bright_blue(),
        "Fetch metadata from image"
    );
    let base64plist = metadata::get_wallpaper_metadata(&image_ctx);

    if base64plist.is_none() {
        return Err(anyhow::Error::msg("No valid metadata found describing wallpaper! Please check if the mime field is available and carries an apple_desktop:h24 and/or apple_desktop:solar value"));
    }

    let image_name = Path::new(path)
        .file_stem()
        .expect("Could not get file name of path")
        .to_string_lossy();

    println!(
        "{}: {}",
        "Preparation".bright_blue(),
        "Detecting wallpaper description kind"
    );
    match base64plist.unwrap() {
        metadata::WallPaperMode::H24(content) => {
            println!(
                "{}: {}",
                "Preparation".bright_blue(),
                "Detected time-based wallpaper"
            );
            timebased::compute_time_based_wallpaper(
                image_ctx,
                content,
                &parent_directory,
                &image_name,
            )
        }
        metadata::WallPaperMode::Solar(content) => {
            println!(
                "{}: {}",
                "Preparation".bright_blue(),
                "Detected solar-based wallpaper"
            );
            solar::compute_solar_based_wallpaper(image_ctx, content, &parent_directory, &image_name)
        }
    }
}
