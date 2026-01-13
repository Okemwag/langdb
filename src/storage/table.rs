use serde::{Deserialize, Serialize};
use crate::utils::{DataType, Value};
use crate::utils::helpers::TypeError;

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Column {
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Self { name, data_type, nullable }
    }

    pub fn validate_value(&self, value: &Value) -> Result<(), TypeError> {
        if value.is_null() && !self.nullable {
            return Err(TypeError::InvalidValue(self.name.clone(), "NULL value not allowed".to_string()));
        }
        if !value.is_null() {
            match (&self.data_type, value) {
                (DataType::Integer, Value::Integer(_)) => Ok(()),
                (DataType::Text, Value::Text(_)) => Ok(()),
                _ => Err(TypeError::InvalidValue(self.name.clone(), format!("Type mismatch: {:?} vs {:?}", value, self.data_type))),
            }
        } else {
            Ok(())
        }
    }
}

/// Table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|col| col.name == name)
    }

    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }

    pub fn validate_row(&self, row: &Row) -> Result<(), TypeError> {
        if row.values.len() != self.columns.len() {
            return Err(TypeError::InvalidValue("row".to_string(), format!("Values count mismatch: expected {}, got {}", self.columns.len(), row.values.len())));
        }
        for (i, value) in row.values.iter().enumerate() {
            self.columns[i].validate_value(value)?;
        }
        Ok(())
    }
}

/// Data row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

/// Result set
#[derive(Debug, Clone)]
pub struct ResultSet {
    pub schema: Schema,
    pub rows: Vec<Row>,
}

impl ResultSet {
    pub fn new(schema: Schema, rows: Vec<Row>) -> Self {
        Self { schema, rows }
    }

    pub fn empty(schema: Schema) -> Self {
        Self { schema, rows: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn to_string(&self) -> String {
        if self.schema.columns.is_empty() { return "Empty result set".to_string(); }
        
        let headers: Vec<String> = self.schema.columns.iter().map(|c| c.name.clone()).collect();
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        
        for row in &self.rows {
            for (i, val) in row.values.iter().enumerate() {
                let s = format!("{}", val);
                if i < col_widths.len() && s.len() > col_widths[i] { col_widths[i] = s.len(); }
            }
        }

        let mut out = String::new();
        for (i, h) in headers.iter().enumerate() {
            out.push_str(&format!("| {:width$} ", h, width = col_widths[i]));
        }
        out.push_str("|\n");
        for w in &col_widths {
            out.push_str(&format!("+{}+", "-".repeat(w + 2)));
        }
        out.push_str("\n");
        for row in &self.rows {
            for (i, val) in row.values.iter().enumerate() {
                out.push_str(&format!("| {:width$} ", format!("{}", val), width = col_widths[i]));
            }
            out.push_str("|\n");
        }
        out.push_str(&format!("\n{} row(s)", self.rows.len()));
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub name: String,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub metadata: TableMetadata,
    pub rows: Vec<Row>,
}

impl Table {
    pub fn new(name: String, schema: Schema) -> Self {
        Self {
            metadata: TableMetadata { name, schema },
            rows: Vec::new(),
        }
    }

    pub fn insert(&mut self, row: Row) -> Result<(), TypeError> {
        self.metadata.schema.validate_row(&row)?;
        self.rows.push(row);
        Ok(())
    }

    pub fn scan(&self) -> Vec<Row> {
        self.rows.clone()
    }
}
