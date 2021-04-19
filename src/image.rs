use std::path::Path;

use libheif_rs::HeifContext;
use anyhow::Result;
use crate::util::png;
use crate::schema::xml::{Background, Image::{Static, Transition}};
use colored::*;
use crate::serializer::GnomeXMLBackgroundSerializer;
use crate::DAY_SECS;
use std::io::BufWriter;

pub struct ImagePoint<'a> {
    pub image_ctx: &'a HeifContext,
    pub img_id: u32,
    pub index: usize,
    pub background: &'a mut Background,
    pub parent_directory: &'a Path,
    pub start_time: f32,
    pub time: f32,
    pub next_time: f32,
}

pub fn process_img(pt: ImagePoint) -> Result<()> {
    let prim_image = pt.image_ctx.image_handle(pt.img_id).unwrap();
    let number_of_images = pt.image_ctx.number_of_top_level_images();
    png::write_png(
        format!("{}/{}.png", pt.parent_directory.to_string_lossy(), pt.index).as_str(),
        prim_image,
    )?;

    // Add to Background Structure
    pt.background.images.push(Static {
        duration: 1 as f32,
        file: format!("{}/{}.png", pt.parent_directory.to_string_lossy(), pt.index),
        idx: pt.index,
    });

    pt.background.images.push(Transition {
        kind: "overlay".to_string(),
        duration: {
            if pt.index < number_of_images - 1 {
                (pt.time - pt.next_time).abs() * DAY_SECS - 1.0
            } else {
                (((pt.time - 1.0).abs() + pt.start_time) * DAY_SECS - 1.0).ceil()
            }
        },
        from: format!("{}/{}.png", pt.parent_directory.to_string_lossy(), pt.index),
        to: format!("{}/{}.png", pt.parent_directory.to_string_lossy(), {
            if pt.index < number_of_images - 1 {
                pt.index + 1
            } else {
                0
            }
        }),
        idx: pt.index,
    });

    Ok(())
}

pub fn save_xml(xml: &mut Background, parent_directory: &Path) -> Result<()> {
    println!("{}: {}", "Conversion".yellow(), "Done!");

    println!("{}: {}", "Conversion".green(), "Creating xml description for new wallpaper");
    xml.images.sort_by(|a, b| match (a, b) {
        (
            Static {
                idx: static_idx, ..
            },
            Transition {
                idx: transition_idx,
                ..
            },
        ) => static_idx.cmp(&transition_idx),
        (
            Static {
                idx: static_idx, ..
            },
            Static {
                idx: transition_idx,
                ..
            },
        ) => static_idx.cmp(&transition_idx),
        (
            Transition {
                idx: static_idx, ..
            },
            Static {
                idx: transition_idx,
                ..
            },
        ) => static_idx.cmp(&transition_idx),
        (
            Transition {
                idx: static_idx, ..
            },
            Transition {
                idx: transition_idx,
                ..
            },
        ) => static_idx.cmp(&transition_idx),
    });

    println!("{}: {}", "Conversion".green(), "Writing wallpaper description");
    let result_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!(
            "{}/default.xml",
            parent_directory.to_string_lossy()
        ))?;
    let mut result = BufWriter::new(result_file);
    let mut ser = GnomeXMLBackgroundSerializer::new(&mut result);
    ser.serialize(&xml)?;
    println!("{}", "Done".green());
    Ok(())
}