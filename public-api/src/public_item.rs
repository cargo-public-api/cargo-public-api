use std::fmt::Display;

use crate::intermediate_public_item::IntermediatePublicItem;
use crate::render::RenderingContext;
use crate::tokens::tokens_to_string;
use crate::tokens::Token;

/// Each public item (except `impl`s) have a path that is displayed like
/// `first::second::third`. Internally we represent that with a `vec!["first",
/// "second", "third"]`. This is a type alias for that internal representation
/// to make the code easier to read.
pub(crate) type PublicItemPath = Vec<String>;

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate. Implements [`Display`] so it can be printed. It
/// also implements [`Ord`], but how items are ordered are not stable yet, and
/// will change in later versions.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PublicItem {
    /// The "your_crate::mod_a::mod_b" part of an item. Split by "::"
    pub(crate) path: PublicItemPath,

    /// The rendered item as a stream of [`Token`]s
    pub(crate) tokens: Vec<Token>,
}

impl PublicItem {
    pub(crate) fn from_intermediate_public_item(
        context: &RenderingContext,
        public_item: &IntermediatePublicItem<'_>,
    ) -> PublicItem {
        PublicItem {
            path: public_item.path_vec(),
            tokens: public_item.render_token_stream(context),
        }
    }

    /// The rendered item as a stream of [`Token`]s
    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.tokens.iter()
    }
}

/// We want pretty-printing (`"{:#?}"`) of [`crate::diff::PublicApiDiff`] to print
/// each public item as `Display`, so implement `Debug` with `Display`.
impl std::fmt::Debug for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// One of the basic uses cases is printing a sorted `Vec` of `PublicItem`s. So
/// we implement `Display` for it.
impl Display for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tokens_to_string(&self.tokens))
    }
}

impl PartialOrd for PublicItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
