use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Context;
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;

use crate::schema::{self, ImageJson, VERSION};
use crate::{Backend, SchemaError};

#[cfg(feature = "gpu")]
use crate::gpu;

pub fn decode_image(input: &Path, output: &Path, backend: Backend) -> anyhow::Result<()> {
    let doc = read_json(input)?;
    validate(&doc)?;

    let width = doc.width;
    let height = doc.height;

    let buffer = match backend {
        Backend::Cpu => decode_buffer_cpu(&doc)?,
        Backend::Gpu => {
            #[cfg(feature = "gpu")]
            {
                gpu::decode_buffer(&doc)?
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

    image::save_buffer(output, &buffer, width, height, image::ColorType::Rgba8)
        .with_context(|| format!("failed to save image: {}", output.display()))?;

    Ok(())
}

fn read_json(input: &Path) -> anyhow::Result<ImageJson> {
    let file = File::open(input)
        .with_context(|| format!("failed to open JSON: {}", input.display()))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).context("failed to parse JSON")
}

pub fn validate(doc: &ImageJson) -> Result<(), SchemaError> {
    if doc.img2jx_version != VERSION {
        return Err(SchemaError::UnsupportedVersion(doc.img2jx_version.clone()));
    }
    if doc.color_space != schema::COLOR_SPACE {
        return Err(SchemaError::UnsupportedColorSpace(doc.color_space.clone()));
    }
    if doc.channels != schema::CHANNELS {
        return Err(SchemaError::UnsupportedChannels(doc.channels.clone()));
    }
    if doc.rows.len() != doc.height as usize {
        return Err(SchemaError::RowCountMismatch {
            expected: doc.height,
            actual: doc.rows.len(),
        });
    }

    for (row_index, row) in doc.rows.iter().enumerate() {
        if row.len() != doc.width as usize {
            return Err(SchemaError::ColumnCountMismatch {
                row_index,
                expected: doc.width,
                actual: row.len(),
            });
        }
        for (col_index, pixel) in row.iter().enumerate() {
            if pixel.x != col_index as u32 || pixel.y != row_index as u32 {
                return Err(SchemaError::PixelPositionMismatch {
                    row: row_index,
                    col: col_index,
                    x: pixel.x,
                    y: pixel.y,
                });
            }
            if schema::hex_to_rgba(&pixel.hex).is_none() {
                return Err(SchemaError::InvalidHex {
                    hex: pixel.hex.clone(),
                    row: row_index,
                    col: col_index,
                });
            }
        }
    }

    Ok(())
}

fn decode_buffer_cpu(doc: &ImageJson) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let width = doc.width;
    let height = doc.height;

    let row_bytes: Vec<Vec<u8>> = doc
        .rows
        .par_iter()
        .enumerate()
        .map(|(row_index, row)| -> anyhow::Result<Vec<u8>> {
            let mut bytes = vec![0u8; width as usize * 4];
            for (col_index, pixel) in row.iter().enumerate() {
                let rgba = schema::hex_to_rgba(&pixel.hex).ok_or_else(|| {
                    anyhow::anyhow!("invalid hex at row {row_index}, col {col_index}")
                })?;
                let offset = col_index * 4;
                bytes[offset] = rgba[0];
                bytes[offset + 1] = rgba[1];
                bytes[offset + 2] = rgba[2];
                bytes[offset + 3] = rgba[3];
            }
            Ok(bytes)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let buffer: Vec<u8> = row_bytes.into_iter().flatten().collect();

    ImageBuffer::from_raw(width, height, buffer)
        .ok_or_else(|| anyhow::anyhow!("failed to build image buffer"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Pixel;

    fn sample_doc() -> ImageJson {
        ImageJson::new(
            2,
            2,
            vec![
                vec![
                    Pixel {
                        x: 0,
                        y: 0,
                        hex: "#FF8000FF".into(),
                    },
                    Pixel {
                        x: 1,
                        y: 0,
                        hex: "#0A141EFF".into(),
                    },
                ],
                vec![
                    Pixel {
                        x: 0,
                        y: 1,
                        hex: "#3C5064FF".into(),
                    },
                    Pixel {
                        x: 1,
                        y: 1,
                        hex: "#C8D2DCFF".into(),
                    },
                ],
            ],
        )
    }

    #[test]
    fn validate_accepts_valid_doc() {
        assert!(validate(&sample_doc()).is_ok());
    }

    #[test]
    fn validate_rejects_row_mismatch() {
        let mut doc = sample_doc();
        doc.rows.pop();
        assert!(matches!(
            validate(&doc),
            Err(SchemaError::RowCountMismatch { .. })
        ));
    }
}
