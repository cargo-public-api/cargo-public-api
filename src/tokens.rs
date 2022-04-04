#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Symbol(String),
    Qualifier(String),
    Kind(String),
    Whitespace,
    Identifier(String),
    Self_(String),
    Function(String),
    Lifetime(String),
    Keyword(String),
    Generic(String),
    Primitive(String),
    Type(String),
}

#[macro_export]
macro_rules! ws {
    () => {
        Token::Whitespace
    };
}

impl Token {
    pub fn symbol(text: impl Into<String>) -> Self {
        Token::Symbol(text.into())
    }
    pub fn qualifier(text: impl Into<String>) -> Self {
        Token::Qualifier(text.into())
    }
    pub fn kind(text: impl Into<String>) -> Self {
        Token::Kind(text.into())
    }
    pub fn identifier(text: impl Into<String>) -> Self {
        Token::Identifier(text.into())
    }
    pub fn self_(text: impl Into<String>) -> Self {
        Token::Self_(text.into())
    }
    pub fn function(text: impl Into<String>) -> Self {
        Token::Function(text.into())
    }
    pub fn lifetime(text: impl Into<String>) -> Self {
        Token::Lifetime(text.into())
    }
    pub fn keyword(text: impl Into<String>) -> Self {
        Token::Keyword(text.into())
    }
    pub fn generic(text: impl Into<String>) -> Self {
        Token::Generic(text.into())
    }
    pub fn primitive(text: impl Into<String>) -> Self {
        Token::Primitive(text.into())
    }
    pub fn type_(text: impl Into<String>) -> Self {
        Token::Type(text.into())
    }
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
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
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

    pub(crate) fn align_score(&self, other: &Self) -> isize {
        let cmp = |a, b| if a == b { 1 } else { -2 };
        match (self, other) {
            (Self::Symbol(a), Self::Symbol(b)) => cmp(a, b),
            (Self::Qualifier(a), Self::Qualifier(b)) => cmp(a, b),
            (Self::Kind(a), Self::Kind(b)) => cmp(a, b),
            (Self::Whitespace, Self::Whitespace) => 0,
            (Self::Identifier(a) | Self::Function(a), Self::Identifier(b) | Self::Function(b)) => {
                cmp(a, b)
            }
            (Self::Lifetime(a), Self::Lifetime(b)) => cmp(a, b),
            (Self::Keyword(a), Self::Keyword(b)) => cmp(a, b),
            (Self::Generic(a), Self::Generic(b)) => cmp(a, b),
            (Self::Primitive(a), Self::Primitive(b)) => cmp(a, b),
            (Self::Type(a), Self::Type(b)) => cmp(a, b),
            _ => -2,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}

impl TokenStream {
    pub fn extend(&mut self, tokens: impl Into<TokenStream>) {
        self.tokens.extend(tokens.into().tokens)
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn remove_from_back(&mut self, len: usize) {
        self.tokens
            .resize(self.tokens.len() - len, Token::Whitespace)
    }

    pub fn tokens(&self) -> impl Iterator<Item = &Token> + '_ {
        self.tokens.iter()
    }

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
    fn from(tokens: Vec<Token>) -> TokenStream {
        TokenStream { tokens }
    }
}

impl From<&[Token]> for TokenStream {
    fn from(tokens: &[Token]) -> TokenStream {
        TokenStream {
            tokens: tokens.to_vec(),
        }
    }
}

impl From<Token> for TokenStream {
    fn from(token: Token) -> TokenStream {
        TokenStream {
            tokens: vec![token],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ChangedToken {
    Same(Token),
    Inserted(Token),
    Removed(Token),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangedTokenStream {
    Same(TokenStream),
    Changed {
        removed: TokenStream,
        inserted: TokenStream,
    },
}

impl ChangedTokenStream {
    pub(crate) fn new(tokens: Vec<ChangedToken>) -> Vec<Self> {
        let mut output = Vec::new();
        let mut recent_change = false;
        let mut stream_a = TokenStream::default();
        let mut stream_b = TokenStream::default();

        for token in tokens {
            match token {
                ChangedToken::Same(t) => {
                    if !recent_change {
                        stream_a.push(t);
                    } else if !stream_a.is_empty() || !stream_b.is_empty() {
                        output.push(ChangedTokenStream::Changed {
                            removed: stream_a,
                            inserted: stream_b,
                        });
                        stream_a = t.into();
                        stream_b = TokenStream::default();
                        recent_change = false;
                    }
                }
                ChangedToken::Inserted(t) => {
                    if recent_change {
                        stream_b.push(t);
                    } else if !stream_a.is_empty() {
                        output.push(ChangedTokenStream::Same(stream_a));
                        stream_a = TokenStream::default();
                        stream_b = t.into();
                        recent_change = true;
                    }
                }
                ChangedToken::Removed(t) => {
                    if recent_change {
                        stream_a.push(t);
                    } else if !stream_a.is_empty() {
                        output.push(ChangedTokenStream::Same(stream_a));
                        stream_a = t.into();
                        stream_b = TokenStream::default();
                        recent_change = true;
                    }
                }
            }
        }
        if !stream_a.is_empty() || !stream_b.is_empty() {
            if recent_change {
                output.push(ChangedTokenStream::Changed {
                    removed: stream_a,
                    inserted: stream_b,
                });
            } else {
                output.push(ChangedTokenStream::Same(stream_a));
            }
        }
        output
    }
}
