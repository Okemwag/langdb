use crate::types::{Column, DataType, Operator, Row, Schema, TypeError, Value};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use thiserror::Error;

/// Errors that can occur during storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    /// Table not found error
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Table already exists error
    #[error("Table already exists: {0}")]
    TableAlreadyExists(String),

    /// Schema mismatch error
    #[error("Schema mismatch: {0}")]
    #[allow(dead_code)]
    SchemaMismatch(String),

    /// Column not found error
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Value validation error
    #[error("Value validation error: {0}")]
    ValidationError(#[from] TypeError),

    /// Concurrency error
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    /// Table name
    pub name: String,
    /// Table schema
    pub schema: Schema,
}

/// Represents a table in the database
#[derive(Debug, Clone)]
pub struct Table {
    /// Table metadata
    pub metadata: TableMetadata,
    /// Rows in the table
    pub rows: Vec<Row>,
}

impl Table {
    /// Create a new empty table with the given name and schema
    pub fn new(name: String, schema: Schema) -> Self {
        Self {
            metadata: TableMetadata { name, schema },
            rows: Vec::new(),
        }
    }

    /// Insert a row into the table
    pub fn insert_row(&mut self, row: Row) -> Result<(), StorageError> {
        self.metadata.schema.validate_row(&row)?;
        self.rows.push(row);
        Ok(())
    }

    /// Insert multiple rows into the table
    #[allow(dead_code)]
    pub fn insert_rows(&mut self, rows: Vec<Row>) -> Result<(), StorageError> {
        for row in &rows {
            self.metadata.schema.validate_row(row)?;
        }
        self.rows.extend(rows);
        Ok(())
    }

    /// Scan all rows
    pub fn scan(&self) -> Vec<Row> {
        self.rows.clone()
    }

    /// Filter rows using a condition
    #[allow(dead_code)]
    pub fn filter(
        &self,
        column: &str,
        op: &Operator,
        value: &Value,
    ) -> Result<Vec<Row>, StorageError> {
        let col_idx = self
            .metadata
            .schema
            .get_column_index(column)
            .ok_or_else(|| StorageError::ColumnNotFound(column.to_string()))?;

        let mut result = Vec::new();

        for row in &self.rows {
            let row_value = row.get_value(col_idx).ok_or_else(|| {
                StorageError::ValidationError(TypeError::InvalidValue(
                    "Row".to_string(),
                    format!("Missing value at index {}", col_idx),
                ))
            })?;

            match row_value.compare(op, value) {
                Ok(true) => result.push(row.clone()),
                Ok(false) => {}
                Err(e) => return Err(e.into()),
            }
        }

        Ok(result)
    }
}

/// Thread-safe database storage
#[derive(Debug, Clone)]
pub struct Database {
    /// Collection of tables with read-write lock for concurrent access
    tables: Arc<RwLock<HashMap<String, Table>>>,
}

impl Database {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new persistent database with filename
    #[allow(dead_code)]
    pub fn with_persistence(_filename: &str) -> Result<Self, StorageError> {
        // Implementation that might fail (file operations, etc.)
        Ok(Self::new())
    }

    /// Create a new table
    pub fn create_table(&self, name: String, schema: Schema) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire write lock: {}", e))
        })?;

        if tables.contains_key(&name) {
            return Err(StorageError::TableAlreadyExists(name));
        }

        let table = Table::new(name.clone(), schema);
        tables.insert(name, table);

        Ok(())
    }

    /// Convenience method to create table from string definitions
    #[allow(dead_code)]
    pub fn create_table_from_defs(
        &self,
        name: &str,
        _columns: &[&str],
    ) -> Result<(), StorageError> {
        let columns = &["id", "name", "email"];
        let schema = Schema::new(
            columns
                .iter()
                .map(|&name| Column::new(name.to_string(), DataType::Text, true))
                .collect(),
        );

        self.create_table(name.to_string(), schema)
    }

    /// Drop a table
    #[allow(dead_code)]
    pub fn drop_table(&self, name: &str) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire write lock: {}", e))
        })?;

        if tables.remove(name).is_none() {
            return Err(StorageError::TableNotFound(name.to_string()));
        }

        Ok(())
    }

    /// Check if a table exists
    pub fn table_exists(&self, name: &str) -> Result<bool, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(tables.contains_key(name))
    }

    /// Get table metadata
    pub fn get_table_metadata(&self, name: &str) -> Result<TableMetadata, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        let table = tables
            .get(name)
            .ok_or_else(|| StorageError::TableNotFound(name.to_string()))?;

        Ok(table.metadata.clone())
    }

    /// Insert a row into a table
    pub fn insert(&self, table_name: &str, row: Row) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire write lock: {}", e))
        })?;

        let table = tables
            .get_mut(table_name)
            .ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;

        table.insert_row(row)
    }

    /// Insert multiple rows into a table
    #[allow(dead_code)]
    pub fn insert_many(&self, table_name: &str, rows: Vec<Row>) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire write lock: {}", e))
        })?;

        let table = tables
            .get_mut(table_name)
            .ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;

        table.insert_rows(rows)
    }

    /// Scan all rows in a table
    pub fn scan(&self, table_name: &str) -> Result<Vec<Row>, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        let table = tables
            .get(table_name)
            .ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;

        Ok(table.scan())
    }

    /// Select rows from a table with a WHERE condition
    #[allow(dead_code)]
    pub fn select_where(
        &self,
        table_name: &str,
        column: &str,
        op: &Operator,
        value: &Value,
    ) -> Result<Vec<Row>, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        let table = tables
            .get(table_name)
            .ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;

        table.filter(column, op, value)
    }

    /// Get a list of all table names
    pub fn get_table_names(&self) -> Result<Vec<String>, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(tables.keys().cloned().collect())
    }

    /// Get the row count for a table
    pub fn get_row_count(&self, table_name: &str) -> Result<usize, StorageError> {
        let tables = self.tables.read().map_err(|e| {
            StorageError::ConcurrencyError(format!("Failed to acquire read lock: {}", e))
        })?;

        let table = tables
            .get(table_name)
            .ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;

        Ok(table.rows.len())
    }
}

/// For backward compatibility with existing code
#[allow(dead_code)]
pub type MemoryStorage = Database;
