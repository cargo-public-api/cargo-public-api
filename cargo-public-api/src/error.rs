use public_api::{PublicItem, diff::ChangedPublicItem};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The API diff is not allowed as per --deny: {0}")]
    DiffDenied(Violations),
}

#[derive(Debug)]
pub struct Violations {
    /// These items were added to the API, but no items may be added to the API
    added: Vec<PublicItem>,

    /// These items were changed in the API, but no items may be changed in the API
    changed: Vec<ChangedPublicItem>,

    /// These items were removed from the API, but no items may be removed from the API
    removed: Vec<PublicItem>,
}

impl Violations {
    pub const fn new() -> Self {
        Self {
            added: Vec::new(),
            changed: Vec::new(),
            removed: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty()
    }

    pub fn extend_added<I: Iterator<Item = PublicItem>>(&mut self, added: I) {
        self.added.extend(added);
    }

    pub fn extend_changed<I: Iterator<Item = ChangedPublicItem>>(&mut self, changed: I) {
        self.changed.extend(changed);
    }

    pub fn extend_removed<I: Iterator<Item = PublicItem>>(&mut self, removed: I) {
        self.removed.extend(removed);
    }
}

impl std::fmt::Display for Violations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.added.is_empty() {
            write!(f, "Added items not allowed: {:?} ", self.added)?;
        }

        if !self.changed.is_empty() {
            write!(f, "Changed items not allowed: {:?} ", self.changed)?;
        }

        if !self.removed.is_empty() {
            write!(f, "Removed items not allowed: {:?} ", self.removed)?;
        }

        Ok(())
    }
}
