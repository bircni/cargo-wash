use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

/// Represents a size in bytes with a unit
#[derive(Clone, Copy)]
pub struct Size {
    value: f64,
    unit: SizeUnit,
}

impl Size {
    const fn new(value: f64, unit: SizeUnit) -> Self {
        Self { value, unit }
    }

    /// Returns the size in bytes
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

#[derive(Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub size: Size,
    pub language: Language,
}

impl Project {
    pub fn new<P: AsRef<Path>>(name: &str, path: P, size: u64, lang: Language) -> Self {
        Self {
            name: name.to_string(),
            path: path.as_ref().to_path_buf(),
            size: Size::to_size(size),
            language: lang,
        }
    }
}

#[derive(strum_macros::Display, Clone, Copy)]
enum SizeUnit {
    B,
    KB,
    MB,
    GB,
}

#[derive(strum_macros::Display, Clone, Copy)]
pub enum Language {
    Rust,
    NodeJS,
}
