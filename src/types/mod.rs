use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use thiserror::Error;

/// Errors related to data types and value conversions
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type conversion error: {0}")]
    ConversionError(String),
    #[error("Unsupported data type: {0}")]
    UnsupportedType(String),
    #[error("Invalid value for type {0}: {1}")]
    InvalidValue(String, String),
    #[error("Value comparison error: {0}")]
    ComparisonError(String),
}

/// Supported SQL data types
///
/// Currently supports:
/// - INTEGER: Signed 64-bit integer
/// - TEXT: UTF-8 string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// 64-bit signed integer
    Integer,
    /// UTF-8 string
    Text,
    // Can be extended with more types later (e.g., FLOAT, BOOLEAN, DATE, etc.)
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
///
/// Values can be:
/// - Integer: 64-bit signed integer
/// - Text: UTF-8 string
/// - Null: SQL NULL value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// 64-bit signed integer value
    Integer(i64),
    /// UTF-8 string value
    Text(String),
    /// SQL NULL value
    Null,
}

impl Value {
    /// Check if the value is NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert value to expected type if possible
    #[allow(dead_code)]
    pub fn as_type(&self, data_type: &DataType) -> Result<Value, TypeError> {
        match (self, data_type) {
            // Already correct type
            (Value::Integer(_), DataType::Integer)
            | (Value::Text(_), DataType::Text)
            | (Value::Null, _) => Ok(self.clone()),

            // Conversion from Text to Integer
            (Value::Text(s), DataType::Integer) => match s.parse::<i64>() {
                Ok(i) => Ok(Value::Integer(i)),
                Err(_) => Err(TypeError::ConversionError(format!(
                    "Cannot convert '{}' to INTEGER",
                    s
                ))),
            },

            // Conversion from Integer to Text
            (Value::Integer(i), DataType::Text) => Ok(Value::Text(i.to_string())),
        }
    }

    /// Compare two values
    pub fn compare(&self, op: &Operator, other: &Value) -> Result<bool, TypeError> {
        match (self, other) {
            // NULL comparisons always return false (except IS NULL which is handled separately)
            (Value::Null, _) | (_, Value::Null) => Ok(false),

            // Integer comparisons
            (Value::Integer(a), Value::Integer(b)) => match op {
                Operator::Eq => Ok(a == b),
                Operator::NotEq => Ok(a != b),
                Operator::Gt => Ok(a > b),
                Operator::Lt => Ok(a < b),
                Operator::GtEq => Ok(a >= b),
                Operator::LtEq => Ok(a <= b),
            },

            // Text comparisons
            (Value::Text(a), Value::Text(b)) => match op {
                Operator::Eq => Ok(a == b),
                Operator::NotEq => Ok(a != b),
                Operator::Gt => Ok(a > b),
                Operator::Lt => Ok(a < b),
                Operator::GtEq => Ok(a >= b),
                Operator::LtEq => Ok(a <= b),
            },

            // Mixed type comparisons - convert to compatible type if possible
            (Value::Integer(a), Value::Text(b)) => match b.parse::<i64>() {
                Ok(b_int) => match op {
                    Operator::Eq => Ok(a == &b_int),
                    Operator::NotEq => Ok(a != &b_int),
                    Operator::Gt => Ok(a > &b_int),
                    Operator::Lt => Ok(a < &b_int),
                    Operator::GtEq => Ok(a >= &b_int),
                    Operator::LtEq => Ok(a <= &b_int),
                },
                Err(_) => Err(TypeError::ComparisonError(format!(
                    "Cannot compare INTEGER with TEXT: {} and '{}'",
                    a, b
                ))),
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
                Err(_) => Err(TypeError::ComparisonError(format!(
                    "Cannot compare TEXT with INTEGER: '{}' and {}",
                    a, b
                ))),
            },
        }
    }
}

/// Comparison operators for WHERE clauses
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    /// Equals (=)
    Eq,
    /// Not equals (<> or !=)
    NotEq,
    /// Greater than (>)
    Gt,
    /// Less than (<)
    Lt,
    /// Greater than or equal (>=)
    GtEq,
    /// Less than or equal (<=)
    LtEq,
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

/// Column definition in a table schema
///
/// Represents a column with name, data type, and nullability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: DataType,
    /// Whether the column can contain NULL values
    pub nullable: bool,
}

impl Column {
    /// Create a new column definition
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            data_type,
            nullable,
        }
    }

    /// Validate that a value matches this column's type
    pub fn validate_value(&self, value: &Value) -> Result<(), TypeError> {
        // NULL check
        if value.is_null() && !self.nullable {
            return Err(TypeError::InvalidValue(
                self.name.clone(),
                "NULL value not allowed for non-nullable column".to_string(),
            ));
        }

        // Type check (skip for NULL values)
        if !value.is_null() {
            match (&self.data_type, value) {
                (DataType::Integer, Value::Integer(_)) => Ok(()),
                (DataType::Text, Value::Text(_)) => Ok(()),
                _ => Err(TypeError::InvalidValue(
                    self.name.clone(),
                    format!(
                        "Value {:?} does not match column type {:?}",
                        value, self.data_type
                    ),
                )),
            }
        } else {
            Ok(())
        }
    }
}

/// Table schema definition
///
/// Defines the structure of a table with columns and their types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// List of columns in the schema
    pub columns: Vec<Column>,
}

impl Schema {
    /// Create a new schema with the given columns
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    /// Get a column by name
    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|col| col.name == name)
    }

    /// Get the index of a column by name
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }

    /// Validate that a row matches this schema
    pub fn validate_row(&self, row: &Row) -> Result<(), TypeError> {
        // Check number of values
        if row.values.len() != self.columns.len() {
            return Err(TypeError::InvalidValue(
                "row".to_string(),
                format!(
                    "Expected {} values, got {}",
                    self.columns.len(),
                    row.values.len()
                ),
            ));
        }

        // Validate each value against its column
        for (i, value) in row.values.iter().enumerate() {
            let column = &self.columns[i];
            column.validate_value(value)?;
        }

        Ok(())
    }
}

/// Represents a row of data in a table
///
/// A row contains a list of values corresponding to columns in a schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    /// Values in the row (one per column)
    pub values: Vec<Value>,
}

impl Row {
    /// Create a new row with the given values
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Get a value by column index
    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

/// Result set from a query
///
/// Contains the schema and rows returned by a query.
#[derive(Debug, Clone)]
pub struct ResultSet {
    /// Schema defining the structure of the result
    pub schema: Schema,
    /// Rows in the result set
    pub rows: Vec<Row>,
}

impl ResultSet {
    /// Create a new result set
    pub fn new(schema: Schema, rows: Vec<Row>) -> Self {
        Self { schema, rows }
    }

    /// Create an empty result set with the given schema
    pub fn empty(schema: Schema) -> Self {
        Self {
            schema,
            rows: vec![],
        }
    }

    /// Get the number of rows in the result set
    #[allow(dead_code)]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Format the result set as a string table
    pub fn to_string(&self) -> String {
        if self.schema.columns.is_empty() {
            return "Empty result set".to_string();
        }

        let mut result = String::new();

        // Column headers
        let headers: Vec<String> = self
            .schema
            .columns
            .iter()
            .map(|col| col.name.clone())
            .collect();

        // Calculate column widths
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

        // Update column widths based on data
        for row in &self.rows {
            for (i, val) in row.values.iter().enumerate() {
                let val_str = format!("{}", val);
                if i < col_widths.len() && val_str.len() > col_widths[i] {
                    col_widths[i] = val_str.len();
                }
            }
        }

        // Header row
        for (i, header) in headers.iter().enumerate() {
            result.push_str("| ");
            result.push_str(&format!("{:width$}", header, width = col_widths[i]));
            result.push_str(" ");
        }
        result.push_str("|\n");

        // Separator row
        for width in &col_widths {
            result.push_str("+");
            result.push_str(&"-".repeat(width + 2));
        }
        result.push_str("+\n");

        // Data rows
        for row in &self.rows {
            for (i, val) in row.values.iter().enumerate() {
                result.push_str("| ");
                let val_str = format!("{}", val);
                result.push_str(&format!("{:width$}", val_str, width = col_widths[i]));
                result.push_str(" ");
            }
            result.push_str("|\n");
        }

        // Row count
        result.push_str(&format!("\n{} row(s) returned", self.rows.len()));

        result
    }

    /// Convert ResultSet to a vector of string vectors (for external processing)
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<Vec<String>> {
        let mut result = Vec::new();

        // Add header row
        let headers: Vec<String> = self
            .schema
            .columns
            .iter()
            .map(|col| col.name.clone())
            .collect();
        result.push(headers);

        // Add data rows
        for row in &self.rows {
            let row_strings: Vec<String> = row.values.iter().map(|v| format!("{}", v)).collect();
            result.push(row_strings);
        }

        result
    }

    /// Check if the result set is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
