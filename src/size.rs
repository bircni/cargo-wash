use core::{fmt, iter::Sum, str::FromStr};

use serde::Serialize;

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;

/// A size in bytes that is displayed in a human readable unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Size(pub u64);

impl Size {
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

impl Sum for Size {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(Self::bytes).sum())
    }
}

impl fmt::Display for Size {
    #[expect(
        clippy::cast_precision_loss,
        reason = "the value is only rounded for display"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (divisor, unit) = match self.0 {
            bytes if bytes >= GB => (GB, "GB"),
            bytes if bytes >= MB => (MB, "MB"),
            bytes if bytes >= KB => (KB, "KB"),
            _ => (1, "B"),
        };
        write!(f, "{:.2} {unit}", self.0 as f64 / divisor as f64)
    }
}

/// Parses sizes such as `1024`, `500MB` or `1.5 gb`, units are powers of 1024.
impl FromStr for Size {
    type Err = String;

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "the value is checked to be finite and not negative, and saturates"
    )]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let unit_start = input
            .find(|c: char| c.is_ascii_alphabetic())
            .unwrap_or(input.len());
        let (number, unit) = input.split_at(unit_start);
        let multiplier = match unit.to_ascii_lowercase().as_str() {
            "" | "b" => 1,
            "k" | "kb" | "kib" => KB,
            "m" | "mb" | "mib" => MB,
            "g" | "gb" | "gib" => GB,
            _ => return Err(format!("unknown unit `{unit}`, expected B, KB, MB or GB")),
        };
        let value: f64 = number
            .trim()
            .parse()
            .map_err(|_| format!("`{}` is not a number", number.trim()))?;
        if !value.is_finite() || value < 0.0 {
            return Err("the size must be a positive number".to_owned());
        }
        Ok(Self((value * multiplier as f64) as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::{GB, KB, MB, Size};

    #[test]
    fn displays_the_largest_fitting_unit() {
        assert_eq!(Size(0).to_string(), "0.00 B");
        assert_eq!(Size(500).to_string(), "500.00 B");
        assert_eq!(Size(KB).to_string(), "1.00 KB");
        assert_eq!(Size(MB + MB / 2).to_string(), "1.50 MB");
        assert_eq!(Size(GB - 1).to_string(), "1024.00 MB");
        assert_eq!(Size(5 * GB).to_string(), "5.00 GB");
    }

    #[test]
    fn parses_sizes_with_units() {
        let parse = |input: &str| input.parse::<Size>().map(Size::bytes);
        assert_eq!(parse("1024"), Ok(1024));
        assert_eq!(parse("10b"), Ok(10));
        assert_eq!(parse("2K"), Ok(2 * KB));
        assert_eq!(parse("500MB"), Ok(500 * MB));
        assert_eq!(parse(" 1.5 gb "), Ok(GB + GB / 2));
        assert_eq!(parse("1GiB"), Ok(GB));
        assert!(parse("").is_err(), "empty");
        assert!(parse("GB").is_err(), "missing number");
        assert!(parse("-1MB").is_err(), "negative");
        assert!(parse("1XB").is_err(), "unknown unit");
        assert!(parse("inf").is_err(), "not a size");
    }

    #[test]
    fn sums_without_losing_bytes() {
        let total: Size = [Size(1), Size(KB), Size(GB)].into_iter().sum();
        assert_eq!(total.bytes(), 1 + KB + GB);
    }
}
