use crate::{
    parser::{
        CreateTableStatement, InsertStatement, Operator, SelectStatement, Statement, WhereClause,
    },
    storage::{Database, StorageError},
    types::{Column, ResultSet, Row, Schema, Value},
};
use thiserror::Error;

/// Error types for query execution
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Error from the storage layer
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    /// Query execution failed
    #[error("Execution error: {0}")]
    ExecutionFailed(String),

    /// Invalid column reference
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Invalid value for operation
    #[error("Invalid value: {0}")]
    #[allow(dead_code)]
    InvalidValue(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    #[allow(dead_code)]
    UnsupportedOperation(String),
}

/// Query executor handles executing SQL statements
#[derive(Clone)]
pub struct QueryExecutor {
    /// Database storage engine
    storage: Database,
}

impl QueryExecutor {
    /// Create a new query executor with the given storage
    pub fn new(storage: Database) -> Self {
        Self { storage }
    }

    /// Execute an SQL statement and return results
    pub fn execute(&self, statement: Statement) -> Result<ResultSet, ExecutionError> {
        match statement {
            Statement::CreateTable(create) => self.execute_create_table(create),
            Statement::Insert(insert) => self.execute_insert(insert),
            Statement::Select(select) => self.execute_select(select),
        }
    }

    /// Execute a CREATE TABLE statement
    fn execute_create_table(
        &self,
        stmt: CreateTableStatement,
    ) -> Result<ResultSet, ExecutionError> {
        // Convert column definitions to our schema format
        let columns: Vec<Column> = stmt
            .columns
            .into_iter()
            .map(|col_def| Column::new(col_def.name, col_def.data_type, col_def.nullable))
            .collect();

        // Create schema from columns
        let schema = Schema::new(columns);

        // Create the table
        self.storage.create_table(stmt.table_name, schema)?;

        // Return empty result set
        Ok(ResultSet::empty(Schema::new(vec![])))
    }

    /// Execute an INSERT statement
    fn execute_insert(&self, stmt: InsertStatement) -> Result<ResultSet, ExecutionError> {
        // Get table metadata to validate the insert
        let metadata = self.storage.get_table_metadata(&stmt.table_name)?;
        let schema = metadata.schema;

        // If columns are specified, we need to map values to the right columns
        if let Some(column_names) = stmt.columns {
            // Validate that specified columns exist in the table
            for name in &column_names {
                if schema.get_column(name).is_none() {
                    return Err(ExecutionError::ColumnNotFound(name.clone()));
                }
            }

            // Process each row to insert
            for values in stmt.values {
                // Make sure we have the right number of values
                if values.len() != column_names.len() {
                    return Err(ExecutionError::ExecutionFailed(format!(
                        "Column count ({}) does not match value count ({})",
                        column_names.len(),
                        values.len()
                    )));
                }

                // Create a full row with NULL values for unspecified columns
                let mut row_values = vec![Value::Null; schema.columns.len()];

                // Fill in the specified values
                for (i, col_name) in column_names.iter().enumerate() {
                    let col_idx = schema
                        .get_column_index(col_name)
                        .ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?;

                    row_values[col_idx] = values[i].clone();
                }

                // Insert the row
                let row = Row::new(row_values);
                self.storage.insert(&stmt.table_name, row)?;
            }
        } else {
            // No columns specified, insert values as-is
            for values in stmt.values {
                // Make sure we have the right number of values
                if values.len() != schema.columns.len() {
                    return Err(ExecutionError::ExecutionFailed(format!(
                        "Column count ({}) does not match value count ({})",
                        schema.columns.len(),
                        values.len()
                    )));
                }

                // Insert the row
                let row = Row::new(values);
                self.storage.insert(&stmt.table_name, row)?;
            }
        }

        // Return empty result set with count of rows affected
        let count = self.storage.get_row_count(&stmt.table_name)?;
        let result = ResultSet::empty(Schema::new(vec![]));

        // Create a simple message about the operation
        println!("Inserted rows. Total rows: {}", count);

        Ok(result)
    }

    /// Execute a SELECT statement
    fn execute_select(&self, stmt: SelectStatement) -> Result<ResultSet, ExecutionError> {
        // Get table metadata and verify table exists
        let metadata = self.storage.get_table_metadata(&stmt.table_name)?;
        let table_schema = metadata.schema;

        // Get all rows from the table initially
        let mut rows = self.storage.scan(&stmt.table_name)?;

        // Apply WHERE clause filter if present
        if let Some(where_clause) = stmt.where_clause {
            rows = self.filter_rows(rows, &where_clause, &table_schema)?;
        }

        // Handle column projection
        let result_schema = if stmt.columns.contains(&"*".to_string()) {
            // Select all columns
            table_schema.clone()
        } else {
            // Project only requested columns
            let mut columns = Vec::new();

            for col_name in &stmt.columns {
                let col = table_schema
                    .get_column(col_name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?;
                columns.push(col.clone());
            }

            // Create a new schema with only the selected columns
            Schema::new(columns)
        };

        // Project rows to include only requested columns
        let result_rows = if stmt.columns.contains(&"*".to_string()) {
            // Keep all columns
            rows
        } else {
            // Project only requested columns
            let mut projected_rows = Vec::new();

            for row in rows {
                let mut values = Vec::new();

                for col_name in &stmt.columns {
                    let col_idx = table_schema
                        .get_column_index(col_name)
                        .ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?;

                    let value = row.get_value(col_idx).ok_or_else(|| {
                        ExecutionError::ExecutionFailed(format!(
                            "Missing value for column {}",
                            col_name
                        ))
                    })?;

                    values.push(value.clone());
                }

                projected_rows.push(Row::new(values));
            }

            projected_rows
        };

        // Return the result set
        Ok(ResultSet::new(result_schema, result_rows))
    }

    /// Filter rows based on WHERE clause conditions
    fn filter_rows(
        &self,
        rows: Vec<Row>,
        where_clause: &WhereClause,
        schema: &Schema,
    ) -> Result<Vec<Row>, ExecutionError> {
        // Convert parser Operator to types Operator
        let convert_operator = |op: &Operator| -> crate::types::Operator {
            match op {
                Operator::Equals => crate::types::Operator::Eq,
                Operator::NotEquals => crate::types::Operator::NotEq,
                Operator::GreaterThan => crate::types::Operator::Gt,
                Operator::LessThan => crate::types::Operator::Lt,
                Operator::GreaterThanOrEqual => crate::types::Operator::GtEq,
                Operator::LessThanOrEqual => crate::types::Operator::LtEq,
            }
        };

        // For each condition in the WHERE clause, filter the rows
        let mut filtered_rows = rows;

        for condition in &where_clause.conditions {
            // Get column index
            let col_idx = schema
                .get_column_index(&condition.column)
                .ok_or_else(|| ExecutionError::ColumnNotFound(condition.column.clone()))?;

            // Convert operator
            let op = convert_operator(&condition.operator);

            // Filter rows
            filtered_rows = filtered_rows
                .into_iter()
                .filter(|row| {
                    if let Some(value) = row.get_value(col_idx) {
                        match value.compare(&op, &condition.value) {
                            Ok(true) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                })
                .collect();
        }

        Ok(filtered_rows)
    }

    /// Helper method to get the database instance
    pub fn get_storage(&self) -> Database {
        self.storage.clone()
    }
}
