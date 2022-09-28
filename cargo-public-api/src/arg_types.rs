use anyhow::anyhow;

use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ArgEnum)]
#[clap(rename_all = "lower")]
pub enum DenyMethod {
    /// All forms of API diffs are denied: additions, changes, deletions.
    All,

    /// Deny added things in API diffs
    Added,

    /// Deny changed things in API diffs
    Changed,

    /// Deny removed things in API diffs
    Removed,
}

impl DenyMethod {
    pub(crate) const fn deny_added(self) -> bool {
        std::matches!(self, Self::All | Self::Added)
    }

    pub(crate) const fn deny_changed(self) -> bool {
        std::matches!(self, Self::All | Self::Changed)
    }

    pub(crate) const fn deny_removed(self) -> bool {
        std::matches!(self, Self::All | Self::Removed)
    }
}

#[derive(Debug)]
pub enum Color {
    Auto,
    Never,
    Always,
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "never" => Ok(Self::Never),
            "always" => Ok(Self::Always),
            _ => Err(anyhow!("See --help")),
        }
    }
}

impl Color {
    pub fn active(&self) -> bool {
        match self {
            Self::Auto => atty::is(atty::Stream::Stdout), // We should not assume Stdout here, but good enough for now
            Self::Never => false,
            Self::Always => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DenyMethod;
    use std::ops::Not;

    #[test]
    fn test_deny_added() {
        assert!(DenyMethod::Added.deny_added());
        assert!(DenyMethod::All.deny_added());

        assert!(DenyMethod::Changed.deny_added().not());
        assert!(DenyMethod::Removed.deny_added().not());
    }

    #[test]
    fn test_deny_changed() {
        assert!(DenyMethod::Changed.deny_changed());
        assert!(DenyMethod::All.deny_changed());

        assert!(DenyMethod::Added.deny_changed().not());
        assert!(DenyMethod::Removed.deny_changed().not());
    }

    #[test]
    fn test_deny_removed() {
        assert!(DenyMethod::Removed.deny_removed());
        assert!(DenyMethod::All.deny_removed());

        assert!(DenyMethod::Added.deny_removed().not());
        assert!(DenyMethod::Changed.deny_removed().not());
    }
}
