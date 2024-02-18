use proc_macro2::{Ident, TokenTree};
use syn::{Attribute, Meta};

#[derive(PartialEq)]
pub enum TokenName {
    Join,
    Find,
    Json,
    Unique,
    Rename,
    Relation,
    Validated,
    Transform,
    JoinIdent,
    Unknown,
}

impl From<Ident> for TokenName {
    fn from(value: Ident) -> Self {
        match value.to_string().as_str() {
            "join" => Self::Join,
            "find" => Self::Find,
            "json" => Self::Json,
            "unique" => Self::Unique,
            "rename" => Self::Rename,
            "relation" => Self::Relation,
            "validated" => Self::Validated,
            "transform" => Self::Transform,
            "join_ident" => Self::JoinIdent,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Join,
    Find,
    Json,
    Unique,
    Validated,
    Rename(String),
    Relation(Ident),
    Transform(Ident),
    JoinIdent(String),
}

impl Token {
    pub fn name(&self) -> TokenName {
        match self {
            Self::Join => TokenName::Join,
            Self::Find => TokenName::Find,
            Self::Json => TokenName::Json,
            Self::Unique => TokenName::Unique,
            Self::Validated => TokenName::Validated,
            Self::Rename(_) => TokenName::Rename,
            Self::Relation(_) => TokenName::Relation,
            Self::Transform(_) => TokenName::Transform,
            Self::JoinIdent(_) => TokenName::JoinIdent,
        }
    }
}

#[derive(Debug)]
pub struct AttributeTokens(Vec<Token>);

impl AttributeTokens {
    pub fn get(&self, token: TokenName) -> Option<&Token> {
        self.0.iter().find(|it| it.name() == token)
    }
}

impl From<Vec<Attribute>> for AttributeTokens {
    fn from(value: Vec<Attribute>) -> Self {
        let list = value.into_iter().find_map(|it| {
            let Meta::List(list) = it.meta else {
                return None;
            };

            if list.path.segments.first().unwrap().ident != "adrastos" {
                return None;
            }

            Some(list)
        });

        let Some(list) = list else {
            return Self(vec![]);
        };

        let tokens = list
            .tokens
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(index, it)| {
                let TokenTree::Ident(ident) = it else {
                    return None;
                };

                match ident.into() {
                    TokenName::Join => Some(Token::Join),
                    TokenName::Find => Some(Token::Find),
                    TokenName::Json => Some(Token::Json),
                    TokenName::Unique => Some(Token::Unique),
                    TokenName::Validated => Some(Token::Validated),
                    TokenName::Rename => {
                        let Some(TokenTree::Literal(literal)) =
                            list.tokens.clone().into_iter().nth(index + 2)
                        else {
                            return None;
                        };

                        let renamed = literal
                            .to_string()
                            .parse::<String>()
                            .unwrap()
                            .replace('\"', "");
                        Some(Token::Rename(renamed))
                    }
                    TokenName::Relation => {
                        let Some(TokenTree::Ident(ident)) =
                            list.tokens.clone().into_iter().nth(index + 2)
                        else {
                            return None;
                        };

                        Some(Token::Relation(ident))
                    }
                    TokenName::Transform => {
                        let Some(TokenTree::Ident(ident)) =
                            list.tokens.clone().into_iter().nth(index + 2)
                        else {
                            return None;
                        };

                        Some(Token::Transform(ident))
                    }
                    TokenName::JoinIdent => {
                        let Some(TokenTree::Literal(literal)) =
                            list.tokens.clone().into_iter().nth(index + 2)
                        else {
                            return None;
                        };

                        let renamed = literal
                            .to_string()
                            .parse::<String>()
                            .unwrap()
                            .replace('\"', "");
                        Some(Token::JoinIdent(renamed))
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        Self(tokens)
    }
}
