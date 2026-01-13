pub mod helpers;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use crate::utils::helpers::TypeError;

/// Supported SQL data types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Text,
}

impl Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Text => write!(f, "TEXT"),
        }
    }
}

impl FromStr for DataType {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "INTEGER" | "INT" => Ok(DataType::Integer),
            "TEXT" | "VARCHAR" | "STRING" | "CHAR" => Ok(DataType::Text),
            _ => Err(TypeError::UnsupportedType(s.to_string())),
        }
    }
}

/// Represents a SQL value of any supported type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Text(String),
    Null,
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn compare(&self, op: &Operator, other: &Value) -> Result<bool, TypeError> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => Ok(false),
            (Value::Integer(a), Value::Integer(b)) => match op {
                Operator::Eq => Ok(a == b),
                Operator::NotEq => Ok(a != b),
                Operator::Gt => Ok(a > b),
                Operator::Lt => Ok(a < b),
                Operator::GtEq => Ok(a >= b),
                Operator::LtEq => Ok(a <= b),
            },
            (Value::Text(a), Value::Text(b)) => match op {
                Operator::Eq => Ok(a == b),
                Operator::NotEq => Ok(a != b),
                Operator::Gt => Ok(a > b),
                Operator::Lt => Ok(a < b),
                Operator::GtEq => Ok(a >= b),
                Operator::LtEq => Ok(a <= b),
            },
            (Value::Integer(a), Value::Text(b)) => match b.parse::<i64>() {
                Ok(b_int) => match op {
                    Operator::Eq => Ok(a == &b_int),
                    Operator::NotEq => Ok(a != &b_int),
                    Operator::Gt => Ok(a > &b_int),
                    Operator::Lt => Ok(a < &b_int),
                    Operator::GtEq => Ok(a >= &b_int),
                    Operator::LtEq => Ok(a <= &b_int),
                },
                Err(_) => Err(TypeError::ComparisonError(format!("Cannot compare INTEGER with TEXT: {} and '{}'", a, b))),
            },
            (Value::Text(a), Value::Integer(b)) => match a.parse::<i64>() {
                Ok(a_int) => match op {
                    Operator::Eq => Ok(&a_int == b),
                    Operator::NotEq => Ok(&a_int != b),
                    Operator::Gt => Ok(&a_int > b),
                    Operator::Lt => Ok(&a_int < b),
                    Operator::GtEq => Ok(&a_int >= b),
                    Operator::LtEq => Ok(&a_int <= b),
                },
                Err(_) => Err(TypeError::ComparisonError(format!("Cannot compare TEXT with INTEGER: '{}' and {}", a, b))),
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Text(s) => write!(f, "'{}'", s),
            Value::Null => write!(f, "NULL"),
        }
    }
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
}
