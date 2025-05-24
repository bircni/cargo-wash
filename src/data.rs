use core::fmt::{self, Display};
use std::path::{Path, PathBuf};

/// Represents the size unit
/// B = Bytes
/// KB = Kilobytes
/// MB = Megabytes
/// GB = Gigabytes
#[derive(strum_macros::Display, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum SizeUnit {
    B,
    GB,
    KB,
    MB,
}

/// Represents a size in bytes with a unit
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Size {
    pub unit: SizeUnit,
    pub value: f64,
}

impl Size {
    pub const fn new(value: f64, unit: SizeUnit) -> Self {
        Self { unit, value }
    }

    /// Returns the size in bytes
    #[expect(clippy::cast_possible_truncation, reason = "Ok here")]
    #[expect(clippy::cast_sign_loss, reason = "Ok here")]
    pub fn size_in_bytes(&self) -> u64 {
        let multiplier = match self.unit {
            SizeUnit::B => 1,
            SizeUnit::KB => 1024,
            SizeUnit::MB => 1024 * 1024,
            SizeUnit::GB => 1024 * 1024 * 1024,
        };
        (self.value * f64::from(multiplier)) as u64
    }

    /// Converts a size in bytes to a `Size` struct
    #[expect(clippy::cast_precision_loss, reason = "Ok here")]
    pub fn to_size(bytes: u64) -> Self {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            Self::new(bytes as f64 / GB as f64, SizeUnit::GB)
        } else if bytes >= MB {
            Self::new(bytes as f64 / MB as f64, SizeUnit::MB)
        } else if bytes >= KB {
            Self::new(bytes as f64 / KB as f64, SizeUnit::KB)
        } else {
            Self::new(bytes as f64, SizeUnit::B)
        }
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

/// Represents a Rust project
/// with its name, path, and size
#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub size: Size,
}

impl Project {
    pub fn new<P: AsRef<Path>>(name: &str, path: P, size: u64) -> Self {
        Self {
            name: name.to_owned(),
            path: path.as_ref().to_path_buf(),
            size: Size::to_size(size),
        }
    }
}
