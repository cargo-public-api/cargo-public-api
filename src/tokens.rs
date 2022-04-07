//! The module tp contain all token handling logic.
#[cfg(doc)]
use crate::item_iterator::PublicItem;

/// A token in a rendered [`PublicItem`], used to apply syntax colouring in downstream applications.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    /// A symbol, like `=` or `::<`
    Symbol(String),
    /// A qualifier, like `pub` or `const`
    Qualifier(String),
    /// The kind of an item, like `function` or `trait`
    Kind(String),
    /// Whitespace, a single space
    Whitespace,
    /// An identifier, like variable names or parts of the path of an item
    Identifier(String),
    /// The identifier self, the text can be `self` or `Self`
    Self_(String),
    /// The identifier for a function, like `fn_arg` in `comprehensive_api::functions::fn_arg`
    Function(String),
    /// A lifetime including the apostrophe `'`, like `'a`
    Lifetime(String),
    /// A keyword, like `impl`
    Keyword(String),
    /// A generic, like `T`
    Generic(String),
    /// A primitive type, like `usize`
    Primitive(String),
    /// A type, like `Iterator`
    Type(String),
}

/// A simple macro to write `Token::Whitespace` in less characters.
#[macro_export]
macro_rules! ws {
    () => {
        Token::Whitespace
    };
}

impl Token {
    /// A symbol, like `=` or `::<`
    pub(crate) fn symbol(text: impl Into<String>) -> Self {
        Self::Symbol(text.into())
    }
    /// A qualifier, like `pub` or `const`
    pub(crate) fn qualifier(text: impl Into<String>) -> Self {
        Self::Qualifier(text.into())
    }
    /// The kind of an item, like `function` or `trait`
    pub(crate) fn kind(text: impl Into<String>) -> Self {
        Self::Kind(text.into())
    }
    /// An identifier, like variable names or parts of the path of an item
    pub(crate) fn identifier(text: impl Into<String>) -> Self {
        Self::Identifier(text.into())
    }
    /// The identifier self, the text can be `self` or `Self`
    pub(crate) fn self_(text: impl Into<String>) -> Self {
        Self::Self_(text.into())
    }
    /// The identifier for a function, like `fn_arg` in `comprehensive_api::functions::fn_arg`
    pub(crate) fn function(text: impl Into<String>) -> Self {
        Self::Function(text.into())
    }
    /// A lifetime including the apostrophe `'`, like `'a`
    pub(crate) fn lifetime(text: impl Into<String>) -> Self {
        Self::Lifetime(text.into())
    }
    /// A keyword, like `impl`
    pub(crate) fn keyword(text: impl Into<String>) -> Self {
        Self::Keyword(text.into())
    }
    /// A generic, like `T`
    pub(crate) fn generic(text: impl Into<String>) -> Self {
        Self::Generic(text.into())
    }
    /// A primitive type, like `usize`
    pub(crate) fn primitive(text: impl Into<String>) -> Self {
        Self::Primitive(text.into())
    }
    /// A type, like `Iterator`
    pub(crate) fn type_(text: impl Into<String>) -> Self {
        Self::Type(text.into())
    }
    /// Give the length of the inner text of this token
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Self::Symbol(l)
            | Self::Qualifier(l)
            | Self::Kind(l)
            | Self::Identifier(l)
            | Self::Self_(l)
            | Self::Function(l)
            | Self::Lifetime(l)
            | Self::Keyword(l)
            | Self::Generic(l)
            | Self::Primitive(l)
            | Self::Type(l) => l.len(),
            Self::Whitespace => 1,
        }
    }
    /// Get the inner text of this token
    pub fn text(&self) -> &str {
        match self {
            Self::Symbol(l)
            | Self::Qualifier(l)
            | Self::Kind(l)
            | Self::Identifier(l)
            | Self::Self_(l)
            | Self::Function(l)
            | Self::Lifetime(l)
            | Self::Keyword(l)
            | Self::Generic(l)
            | Self::Primitive(l)
            | Self::Type(l) => l,
            Self::Whitespace => " ",
        }
    }
}

/// A sequence of Tokens with nice helper functions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd)]
pub struct TokenStream {
    /// The tokens
    pub tokens: Vec<Token>,
}

impl TokenStream {
    /// Extend this [`TokenStream`] with extra [`Token`]s.
    pub fn extend(&mut self, tokens: impl Into<Self>) {
        self.tokens.extend(tokens.into().tokens);
    }

    /// Push a single [`Token`] to the end of this sequence.
    pub(crate) fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Get the number of tokens in this [`TokenStream`].
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if there are no [`Token`]s in this sequence.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Remove the specified number of [`Token`]s from the end of this sequence.
    pub(crate) fn remove_from_back(&mut self, len: usize) {
        self.tokens
            .resize(self.tokens.len() - len, Token::Whitespace);
    }

    /// Get access to the tokens with an Iterator.
    pub fn tokens(&self) -> impl Iterator<Item = &Token> + '_ {
        self.tokens.iter()
    }

    /// Get the total length of all [`Token`]s in this sequence, see [`Token::len`].
    pub fn tokens_len(&self) -> usize {
        self.tokens().map(Token::len).sum()
    }
}

impl std::fmt::Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tokens().map(Token::text).collect::<String>())
    }
}

//impl<T: Iterator<Item = Token>> From<T> for TokenStream {
//    fn from(tokens: T) -> TokenStream {
//        TokenStream {
//            tokens: tokens.collect(),
//        }
//    }
//}

impl From<Vec<Token>> for TokenStream {
    fn from(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
}

impl From<&[Token]> for TokenStream {
    fn from(tokens: &[Token]) -> Self {
        Self {
            tokens: tokens.to_vec(),
        }
    }
}

impl From<Token> for TokenStream {
    fn from(token: Token) -> Self {
        Self {
            tokens: vec![token],
        }
    }
}
