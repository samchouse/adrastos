use std::{fmt, hash::Hash};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub view: Option<String>,
    pub create: Option<String>,
    pub update: Option<String>,
    pub delete: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Expression {
    operator: ExpressionOperator,
    operands: [Operand; 2],
}

#[derive(Debug, Clone, Hash, PartialEq)]
struct Clause {
    operator: ClauseOperator,
    operands: [Symbol; 2],
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum Operand {
    Expression(Box<Expression>),
    Clause(Clause),
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum ExpressionOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum ClauseOperator {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum Symbol {
    Builtin(BuiltinSymbol),
    Database(String),
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum BuiltinSymbol {
    RequestUser,
}

#[derive(Debug, Clone)]
enum ParserType {
    Operator(ExpressionOperator),
    Clause(String),
}

impl fmt::Display for ExpressionOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::And => "&&",
                Self::Or => "||",
            }
        )
    }
}

impl fmt::Display for ClauseOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Equal => "==",
                Self::NotEqual => "!=",
            }
        )
    }
}

impl TryFrom<String> for ExpressionOperator {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value {
            _ if Self::And.to_string() == value => Ok(Self::And),
            _ if Self::Or.to_string() == value => Ok(Self::Or),
            _ => Err(Error::BadRequest("Invalid expression was provided".into())),
        }
    }
}

impl TryFrom<String> for ClauseOperator {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value {
            _ if Self::Equal.to_string() == value => Ok(Self::Equal),
            _ if Self::NotEqual.to_string() == value => Ok(Self::NotEqual),
            _ => Err(Error::BadRequest("Invalid expression was provided".into())),
        }
    }
}

impl Symbols for ExpressionOperator {
    fn symbols() -> Vec<String> {
        vec![Self::And.to_string(), Self::Or.to_string()]
    }
}

impl Symbols for ClauseOperator {
    fn symbols() -> Vec<String> {
        vec![Self::Equal.to_string(), Self::NotEqual.to_string()]
    }
}

trait Symbols {
    fn symbols() -> Vec<String>;
    fn regex_symbols() -> Vec<String> {
        Self::symbols()
            .into_iter()
            .map(|s| regex::escape(&s))
            .collect::<Vec<_>>()
    }
}

impl TryFrom<String> for Expression {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.replace(' ', "");

        let mut prev_end = 0;
        let mut result = vec![];
        for mat in Regex::new(&format!(
            r#"{}|\(.[^\(\)]+\)"#,
            ExpressionOperator::regex_symbols().join("|")
        ))
        .unwrap()
        .find_iter(&value)
        {
            if mat.start() > prev_end {
                result.push(ParserType::Clause(value[prev_end..mat.start()].to_string()));
            }
            prev_end = mat.end();

            let mat = mat.as_str().to_string();
            if mat.starts_with("(") {
                result.push(ParserType::Clause(mat))
            } else {
                result.push(ParserType::Operator(ExpressionOperator::try_from(mat)?));
            }
        }

        if prev_end < value.len() {
            result.push(ParserType::Clause(value[prev_end..value.len()].to_string()));
        }

        let mut expression = None;
        for idx in 0..(result.len() - 1) / 2 {
            let actual_index = idx * 2 + 1;

            let Some(ParserType::Operator(symbol)) = result.get(actual_index) else {
                return Err(Error::BadRequest("Improper expression".to_string()));
            };
            let Some(ParserType::Clause(left)) = result.get(actual_index - 1) else {
                return Err(Error::BadRequest("Improper expression".to_string()));
            };
            let Some(ParserType::Clause(right)) = result.get(actual_index + 1) else {
                return Err(Error::BadRequest("Improper expression".to_string()));
            };

            expression = Some(Expression {
                operator: symbol.clone(),
                operands: [
                    match expression {
                        Some(expression) => Operand::Expression(Box::new(expression)),
                        None => Expression::try_from(left.replace(['(', ')'], "").clone())
                            .ok()
                            .map(|e| Operand::Expression(Box::new(e)))
                            .unwrap_or(Operand::Clause(Clause::try_from(left.clone())?)),
                    },
                    Expression::try_from(right.replace(['(', ')'], "").clone())
                        .ok()
                        .map(|e| Operand::Expression(Box::new(e)))
                        .unwrap_or(Operand::Clause(Clause::try_from(right.clone())?)),
                ],
            })
        }

        expression.ok_or(Error::InternalServerError(
            "Unable to construct expression".into(),
        ))
    }
}

impl TryFrom<String> for Clause {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let regex = Regex::new(&ClauseOperator::regex_symbols().join("|")).unwrap();
        let mat = regex
            .find(&value)
            .ok_or(Error::BadRequest("Invalid expression".into()))?;

        Ok(Clause {
            operator: ClauseOperator::try_from(mat.as_str().to_string())?,
            operands: [
                Symbol::try_from(value[0..mat.start()].to_string())?,
                Symbol::try_from(value[mat.end()..value.len()].to_string())?,
            ],
        })
    }
}

impl TryFrom<String> for Symbol {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(if value.starts_with("@") {
            Symbol::Builtin(BuiltinSymbol::try_from(value.replace('@', ""))?)
        } else {
            Symbol::Database(value)
        })
    }
}

impl TryFrom<String> for BuiltinSymbol {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "request.user" => Ok(BuiltinSymbol::RequestUser),
            _ => Err(Error::BadRequest(
                "Invalid builtin symbol was provided".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_parse() {
        let result = Expression::try_from(
            "@request.user == user_id && (name == last_name || last_name != name)".to_string(),
        );
        assert_eq!(
            result,
            Ok(Expression {
                operator: ExpressionOperator::And,
                operands: [
                    Operand::Clause(Clause {
                        operator: ClauseOperator::Equal,
                        operands: [
                            Symbol::Builtin(BuiltinSymbol::RequestUser),
                            Symbol::Database("user_id".to_string())
                        ]
                    }),
                    Operand::Expression(Box::new(Expression {
                        operator: ExpressionOperator::Or,
                        operands: [
                            Operand::Clause(Clause {
                                operator: ClauseOperator::Equal,
                                operands: [
                                    Symbol::Database("name".to_string()),
                                    Symbol::Database("last_name".to_string())
                                ]
                            }),
                            Operand::Clause(Clause {
                                operator: ClauseOperator::NotEqual,
                                operands: [
                                    Symbol::Database("last_name".to_string()),
                                    Symbol::Database("name".to_string())
                                ]
                            })
                        ]
                    }))
                ]
            })
        );
    }
}
