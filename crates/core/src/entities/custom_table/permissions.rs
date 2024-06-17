use std::{fmt, hash::Hash};

use regex::Regex;
use sea_query::{all, any, Alias, Cond, Expr, SimpleExpr};
use serde::{Deserialize, Serialize};

use crate::{entities::AnyUser, error::Error};

use super::schema::CustomTableSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub strict: bool,
    pub view: Option<String>,
    pub create: Option<String>,
    pub update: Option<String>,
    pub delete: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Permission {
    Expression(Box<Expression>),
    Clause(Clause),
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Expression {
    operator: ExpressionOperator,
    operands: [Permission; 2],
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Clause {
    operator: ClauseOperator,
    operands: [Symbol; 2],
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
    Value(Value),
}

#[derive(Debug, Clone, Hash, PartialEq)]
enum Value {
    String(String),
    Number(i64),
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

impl Permission {
    pub fn parse(schema: &CustomTableSchema, value: String) -> Result<Self, Error> {
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

        let mut permission = None;
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

            permission = Some(Permission::Expression(Box::new(Expression {
                operator: symbol.clone(),
                operands: [
                    match permission {
                        Some(permission) => permission,
                        None => Permission::parse(schema, left.replace(['(', ')'], "").clone())?,
                    },
                    Permission::parse(schema, right.replace(['(', ')'], "").clone())?,
                ],
            })))
        }

        if permission.is_none() {
            permission = Some(Permission::Clause(Clause::parse(schema, value)?))
        }

        permission.ok_or(Error::InternalServerError(
            "Unable to construct expression".into(),
        ))
    }
}

impl Clause {
    fn parse(schema: &CustomTableSchema, value: String) -> Result<Self, Error> {
        let regex = Regex::new(&ClauseOperator::regex_symbols().join("|")).unwrap();
        let mat = regex
            .find(&value)
            .ok_or(Error::BadRequest("Invalid expression".into()))?;

        Ok(Clause {
            operator: ClauseOperator::try_from(mat.as_str().to_string())?,
            operands: [
                Symbol::parse(schema, value[0..mat.start()].to_string())?,
                Symbol::parse(schema, value[mat.end()..value.len()].to_string())?,
            ],
        })
    }
}

impl Symbol {
    fn parse(schema: &CustomTableSchema, value: String) -> Result<Self, Error> {
        Ok(if value.starts_with("@") {
            Symbol::Builtin(BuiltinSymbol::try_from(value.replace('@', ""))?)
        } else {
            match schema.fields.iter().find(|f| f.name == value) {
                Some(_) => Symbol::Database(value),
                None => {
                    let string_regex = Regex::new(r"'.+'").unwrap();
                    if string_regex.find(&value).is_some() {
                        return Ok(Symbol::Value(Value::String(value.replace('\'', ""))));
                    }

                    let number_regex = Regex::new(r"-?\d+").unwrap();
                    if number_regex.find(&value).is_some() {
                        return Ok(Symbol::Value(Value::Number(value.parse().map_err(
                            |_| Error::InternalServerError("Unable to parse number".into()),
                        )?)));
                    }

                    return Err(Error::BadRequest("Invalid value type".into()));
                }
            }
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

impl Permission {
    pub fn to_sql_cond(&self, user: &AnyUser) -> Cond {
        match self {
            Self::Expression(expression) => match expression.operator {
                ExpressionOperator::And => {
                    all![
                        expression.operands[0].to_sql_cond(user),
                        expression.operands[1].to_sql_cond(user)
                    ]
                }
                ExpressionOperator::Or => {
                    any![
                        expression.operands[0].to_sql_cond(user),
                        expression.operands[1].to_sql_cond(user)
                    ]
                }
            },
            Self::Clause(clause) => {
                let database_alias = clause
                    .operands
                    .iter()
                    .filter_map(|o| match o {
                        Symbol::Database(alias) => Some(alias),
                        _ => None,
                    })
                    .next()
                    .unwrap();
                let other = clause
                    .operands
                    .iter()
                    .filter_map(|o| match o {
                        Symbol::Database(_) => None,
                        Symbol::Builtin(builtin) => match builtin {
                            BuiltinSymbol::RequestUser => Some(SimpleExpr::from(user.id.clone())),
                        },
                        Symbol::Value(value) => match value {
                            Value::String(value) => Some(SimpleExpr::from(value)),
                            Value::Number(value) => Some(SimpleExpr::from(*value)),
                        },
                    })
                    .next()
                    .unwrap();

                let expr = Expr::col(Alias::new(database_alias));
                all![match clause.operator {
                    ClauseOperator::Equal => expr.eq(other),
                    ClauseOperator::NotEqual => expr.ne(other),
                }]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::custom_table::fields::{Field, FieldInfo};

    use super::*;

    #[test]
    fn simple_parse() {
        let schema = CustomTableSchema {
            fields: vec![Field {
                name: "user_id".into(),
                info: FieldInfo::Boolean,
            }],
            ..Default::default()
        };

        let result = Permission::parse(&schema, "@request.user == user_id".to_string());
        assert_eq!(
            result,
            Ok(Permission::Clause(Clause {
                operator: ClauseOperator::Equal,
                operands: [
                    Symbol::Builtin(BuiltinSymbol::RequestUser),
                    Symbol::Database("user_id".to_string())
                ]
            }))
        );
    }

    #[test]
    fn complex_parse() {
        let schema = CustomTableSchema {
            fields: vec![
                Field {
                    name: "user_id".into(),
                    info: FieldInfo::Boolean,
                },
                Field {
                    name: "name".into(),
                    info: FieldInfo::Boolean,
                },
                Field {
                    name: "age".into(),
                    info: FieldInfo::Boolean,
                },
            ],
            ..Default::default()
        };

        let result = Permission::parse(
            &schema,
            "@request.user == user_id && (name == 'Sam' || 17 != age)".to_string(),
        );
        assert_eq!(
            result,
            Ok(Permission::Expression(Box::new(Expression {
                operator: ExpressionOperator::And,
                operands: [
                    Permission::Clause(Clause {
                        operator: ClauseOperator::Equal,
                        operands: [
                            Symbol::Builtin(BuiltinSymbol::RequestUser),
                            Symbol::Database("user_id".to_string())
                        ]
                    }),
                    Permission::Expression(Box::new(Expression {
                        operator: ExpressionOperator::Or,
                        operands: [
                            Permission::Clause(Clause {
                                operator: ClauseOperator::Equal,
                                operands: [
                                    Symbol::Database("name".to_string()),
                                    Symbol::Value(Value::String("Sam".into()))
                                ]
                            }),
                            Permission::Clause(Clause {
                                operator: ClauseOperator::NotEqual,
                                operands: [
                                    Symbol::Value(Value::Number(17)),
                                    Symbol::Database("age".to_string()),
                                ]
                            })
                        ]
                    }))
                ]
            })))
        );
    }

    #[test]
    fn simple_cond() {
        let schema = CustomTableSchema {
            fields: vec![Field {
                name: "user_id".into(),
                info: FieldInfo::Boolean,
            }],
            ..Default::default()
        };
        let permission = Permission::parse(&schema, "@request.user == user_id".to_string());

        let user = AnyUser {
            id: "test_user".into(),
            ..Default::default()
        };

        let result = permission.map(|p| p.to_sql_cond(&user));
        assert_eq!(
            result,
            Ok(Cond::all().add(Expr::col(Alias::new("user_id")).eq(user.id)))
        )
    }

    #[test]
    fn complex_cond() {
        let schema = CustomTableSchema {
            fields: vec![
                Field {
                    name: "user_id".into(),
                    info: FieldInfo::Boolean,
                },
                Field {
                    name: "name".into(),
                    info: FieldInfo::Boolean,
                },
                Field {
                    name: "age".into(),
                    info: FieldInfo::Boolean,
                },
            ],
            ..Default::default()
        };
        let permission = Permission::parse(
            &schema,
            "@request.user == user_id && (name == 'Sam' || 17 != age)".to_string(),
        );

        let user = AnyUser {
            id: "test_user".into(),
            ..Default::default()
        };

        let result = permission.map(|p| p.to_sql_cond(&user));
        assert_eq!(
            result,
            Ok(all![
                Expr::col(Alias::new("user_id")).eq(user.id),
                any![
                    Expr::col(Alias::new("name")).eq("Sam"),
                    Expr::col(Alias::new("age")).ne(17_i64)
                ]
            ])
        )
    }
}
