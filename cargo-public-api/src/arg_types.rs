use std::io::IsTerminal;

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum)]
#[value(rename_all = "lower")]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum)]
#[value(rename_all = "lower")]
pub enum Color {
    /// Colors will be used if stdout is a terminal. Colors will not be used if
    /// stdout is a regular file.
    Auto,

    /// Colors will never be used.
    Never,

    /// Colors will always be used.
    Always,
}

impl Color {
    pub fn active(self) -> bool {
        match self {
            Self::Auto => std::io::stdout().is_terminal(), // We should not assume Stdout here, but good enough for now
            Self::Never => false,
            Self::Always => true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum)]
#[value(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)] // We might add support for omitting other things in the future
pub enum Omit {
    /// Omit items that belong to Blanket Implementations such as `impl<T> Any
    /// for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where
    /// U: From<T>`.
    BlanketImpls,

    /// Omit items that belong to Auto Trait Implementations such as `impl Send
    /// for ...`, `impl Sync for ...`, and `impl Unpin for ...`.
    AutoTraitImpls,

    /// Omit items that belong to Auto Derived Implementations such as `Clone`,
    /// `Debug`, and `Eq`.
    AutoDerivedImpls,
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
