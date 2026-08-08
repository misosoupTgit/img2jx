pub mod decode;
pub mod encode;
pub mod parallel;
pub mod schema;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use decode::decode_image;
pub use encode::encode_image;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cpu,
    Gpu,
}

impl std::str::FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cpu" => Ok(Self::Cpu),
            "gpu" => Ok(Self::Gpu),
            _ => Err(format!("unknown backend '{s}'; expected cpu or gpu")),
        }
    }
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("unsupported img2jx_version: {0}")]
    UnsupportedVersion(String),
    #[error("unsupported color_space: {0} (expected sRGB)")]
    UnsupportedColorSpace(String),
    #[error("unsupported channels: {0} (expected RGBA)")]
    UnsupportedChannels(String),
    #[error("row count mismatch: expected {expected} rows, got {actual}")]
    RowCountMismatch { expected: u32, actual: usize },
    #[error("row {row_index} column count mismatch: expected {expected}, got {actual}")]
    ColumnCountMismatch {
        row_index: usize,
        expected: u32,
        actual: usize,
    },
    #[error("pixel at row {row}, col {col}: x={x} y={y} does not match position")]
    PixelPositionMismatch {
        row: usize,
        col: usize,
        x: u32,
        y: u32,
    },
    #[error("invalid hex color '{hex}' at row {row}, col {col}")]
    InvalidHex {
        hex: String,
        row: usize,
        col: usize,
    },
}
