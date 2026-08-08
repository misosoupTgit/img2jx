use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use image::RgbaImage;
use rayon::prelude::*;

use crate::schema::{self, ImageJson, Pixel};
use crate::Backend;

#[cfg(feature = "gpu")]
use crate::gpu;

pub fn encode_image(
    input: &Path,
    output: &Path,
    pretty: bool,
    backend: Backend,
) -> anyhow::Result<()> {
    let img = image::open(input)
        .with_context(|| format!("failed to open image: {}", input.display()))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let rows = match backend {
        Backend::Cpu => encode_rows_cpu(&rgba, width, height),
        Backend::Gpu => {
            #[cfg(feature = "gpu")]
            {
                gpu::encode_rows(&rgba)?
            }
            #[cfg(not(feature = "gpu"))]
            {
                anyhow::bail!(
                    "GPU backend requested but this binary was built without the 'gpu' feature.\n\
                     Rebuild with: cargo build --release --features gpu"
                );
            }
        }
    };

    let doc = ImageJson::new(width, height, rows);
    write_json(output, &doc, pretty)
}

fn encode_rows_cpu(rgba: &RgbaImage, width: u32, height: u32) -> Vec<Vec<Pixel>> {
    (0..height)
        .into_par_iter()
        .map(|y| {
            (0..width)
                .map(|x| {
                    let pixel = rgba.get_pixel(x, y);
                    Pixel {
                        x,
                        y,
                        hex: schema::rgba_to_hex(pixel[0], pixel[1], pixel[2], pixel[3]),
                    }
                })
                .collect()
        })
        .collect()
}

fn write_json(output: &Path, doc: &ImageJson, pretty: bool) -> anyhow::Result<()> {
    let file = File::create(output)
        .with_context(|| format!("failed to create output file: {}", output.display()))?;
    let writer = BufWriter::new(file);

    if pretty {
        serde_json::to_writer_pretty(writer, doc).context("failed to write JSON")?;
    } else {
        serde_json::to_writer(writer, doc).context("failed to write JSON")?;
    }

    Ok(())
}
