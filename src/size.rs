use core::{fmt, iter::Sum};

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;

/// A size in bytes that is displayed in a human readable unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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
    fn sums_without_losing_bytes() {
        let total: Size = [Size(1), Size(KB), Size(GB)].into_iter().sum();
        assert_eq!(total.bytes(), 1 + KB + GB);
    }
}
