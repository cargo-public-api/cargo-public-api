#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Symbol(String),
    Qualifier(String),
    Kind(String),
    Whitespace,
    Identifier(String),
    Function(String),
    Lifetime(String),
    Keyword(String),
    Generic(String),
    Primitive(String),
    Type(String),
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
pub enum ChangedToken {
    Same(Token),
    Inserted(Token),
    Removed(Token),
}
