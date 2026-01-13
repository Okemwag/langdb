use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use thiserror::Error;
use super::table::{Table, Row, Schema, TableMetadata};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Table not found: {0}")]
    TableNotFound(String),
    #[error("Table already exists: {0}")]
    TableAlreadyExists(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Concurrency Error: {0}")]
    ConcurrencyError(String),
    #[error("Validation Error: {0}")]
    ValidationError(#[from] crate::utils::helpers::TypeError),
}

#[derive(Debug, Clone)]
pub struct Database {
    pub tables: Arc<RwLock<HashMap<String, Table>>>,
}

impl Database {
    pub fn new() -> Self {
        Self { tables: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn save_to_disk(&self, filename: &str) -> Result<(), StorageError> {
        let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        let json = serde_json::to_string_pretty(&*tables)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_from_disk(filename: &str) -> Result<Self, StorageError> {
        if !std::path::Path::new(filename).exists() { return Ok(Self::new()); }
        let json = std::fs::read_to_string(filename)?;
        let tables: HashMap<String, Table> = serde_json::from_str(&json)?;
        Ok(Self { tables: Arc::new(RwLock::new(tables)) })
    }

    pub fn create_table(&self, name: String, schema: Schema) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        if tables.contains_key(&name) { return Err(StorageError::TableAlreadyExists(name)); }
        tables.insert(name.clone(), Table::new(name, schema));
        Ok(())
    }

    pub fn get_table_names(&self) -> Result<Vec<String>, StorageError> {
        let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        Ok(tables.keys().cloned().collect())
    }

    pub fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata, StorageError> {
        let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        let table = tables.get(table_name).ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;
        Ok(table.metadata.clone())
    }

    pub fn insert(&self, table_name: &str, row: Row) -> Result<(), StorageError> {
        let mut tables = self.tables.write().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        let table = tables.get_mut(table_name).ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;
        table.insert(row)?;
        Ok(())
    }

    pub fn scan(&self, table_name: &str) -> Result<Vec<Row>, StorageError> {
         let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
         let table = tables.get(table_name).ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;
         Ok(table.scan())
    }

    pub fn get_row_count(&self, table_name: &str) -> Result<usize, StorageError> {
        let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        let table = tables.get(table_name).ok_or_else(|| StorageError::TableNotFound(table_name.to_string()))?;
        Ok(table.rows.len())
    }

    pub fn table_exists(&self, table_name: &str) -> Result<bool, StorageError> {
        let tables = self.tables.read().map_err(|e| StorageError::ConcurrencyError(e.to_string()))?;
        Ok(tables.contains_key(table_name))
    }
}
