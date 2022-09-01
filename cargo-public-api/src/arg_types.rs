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
    pub(crate) fn deny_added(self) -> bool {
        std::matches!(self, DenyMethod::All | DenyMethod::Added)
    }

    pub(crate) fn deny_changed(self) -> bool {
        std::matches!(self, DenyMethod::All | DenyMethod::Changed)
    }

    pub(crate) fn deny_removed(self) -> bool {
        std::matches!(self, DenyMethod::All | DenyMethod::Removed)
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
            "auto" => Ok(Color::Auto),
            "never" => Ok(Color::Never),
            "always" => Ok(Color::Always),
            _ => Err(anyhow!("See --help")),
        }
    }
}

impl Color {
    pub fn active(&self) -> bool {
        match self {
            Color::Auto => atty::is(atty::Stream::Stdout), // We should not assume Stdout here, but good enough for now
            Color::Never => false,
            Color::Always => true,
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
