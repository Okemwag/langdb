use crate::{
    parser::{
        sql_parser::{CreateTableStatement, InsertStatement, Operator as ParserOperator, SelectStatement, Statement, WhereClause}, // Use ParserOperator to avoid conflict or just import types
    },
    storage::{Database, StorageError},
    storage::table::{Column, ResultSet, Row, Schema},
    utils::{Value, Operator as TypesOperator},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("Execution error: {0}")]
    ExecutionFailed(String),
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    #[error("Invalid value: {0}")]
    #[allow(dead_code)]
    InvalidValue(String),
    #[error("Unsupported operation: {0}")]
    #[allow(dead_code)]
    UnsupportedOperation(String),
}

#[derive(Clone)]
pub struct QueryExecutor {
    storage: Database,
}

impl QueryExecutor {
    pub fn new(storage: Database) -> Self {
        Self { storage }
    }

    pub fn execute(&self, statement: Statement) -> Result<ResultSet, ExecutionError> {
        match statement {
            Statement::CreateTable(create) => self.execute_create_table(create),
            Statement::Insert(insert) => self.execute_insert(insert),
            Statement::Select(select) => self.execute_select(select),
        }
    }

    fn execute_create_table(&self, stmt: CreateTableStatement) -> Result<ResultSet, ExecutionError> {
        let columns: Vec<Column> = stmt.columns.into_iter()
            .map(|col_def| Column::new(col_def.name, col_def.data_type, col_def.nullable))
            .collect();
        let schema = Schema::new(columns);
        self.storage.create_table(stmt.table_name, schema)?;
        Ok(ResultSet::empty(Schema::new(vec![])))
    }

    fn execute_insert(&self, stmt: InsertStatement) -> Result<ResultSet, ExecutionError> {
        let metadata = self.storage.get_table_metadata(&stmt.table_name)?;
        let schema = metadata.schema;

        if let Some(column_names) = stmt.columns {
            for name in &column_names {
                if schema.get_column(name).is_none() {
                    return Err(ExecutionError::ColumnNotFound(name.clone()));
                }
            }
            for values in stmt.values {
                if values.len() != column_names.len() {
                    return Err(ExecutionError::ExecutionFailed(format!("Column count ({}) does not match value count ({})", column_names.len(), values.len())));
                }
                let mut row_values = vec![Value::Null; schema.columns.len()];
                for (i, col_name) in column_names.iter().enumerate() {
                    let col_idx = schema.get_column_index(col_name).ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?;
                    row_values[col_idx] = values[i].clone();
                }
                self.storage.insert(&stmt.table_name, Row::new(row_values))?;
            }
        } else {
            for values in stmt.values {
                if values.len() != schema.columns.len() {
                    return Err(ExecutionError::ExecutionFailed(format!("Column count ({}) does not match value count ({})", schema.columns.len(), values.len())));
                }
                self.storage.insert(&stmt.table_name, Row::new(values))?;
            }
        }

        let count = self.storage.get_row_count(&stmt.table_name)?;
        Ok(ResultSet::empty(Schema::new(vec![])))
    }

    fn execute_select(&self, stmt: SelectStatement) -> Result<ResultSet, ExecutionError> {
        let left_metadata = self.storage.get_table_metadata(&stmt.table_name)?;
        let left_schema = left_metadata.schema;
        let left_rows = self.storage.scan(&stmt.table_name)?;

        let (mut working_schema, mut working_rows) = if let Some(join) = stmt.join {
            let right_metadata = self.storage.get_table_metadata(&join.table_name)?;
            let right_schema = right_metadata.schema;
            let right_rows = self.storage.scan(&join.table_name)?;

            let find_col_idx = |schema: &Schema, table_name: &str, col_ref: &str| -> Result<usize, ExecutionError> {
                if let Some(idx) = schema.get_column_index(col_ref) { return Ok(idx); }
                if col_ref.contains('.') {
                    let parts: Vec<&str> = col_ref.split('.').collect();
                    if parts.len() == 2 && parts[0] == table_name {
                        if let Some(idx) = schema.get_column_index(parts[1]) { return Ok(idx); }
                    }
                }
                if !col_ref.contains('.') {
                     if let Some(idx) = schema.get_column_index(col_ref) { return Ok(idx); }
                }
                Err(ExecutionError::ColumnNotFound(format!("{} in table {}", col_ref, table_name)))
            };

            let left_col_idx = find_col_idx(&left_schema, &stmt.table_name, &join.on_left)
                .or_else(|_| find_col_idx(&left_schema, &stmt.table_name, &join.on_right))?;
            
            let right_col_idx = find_col_idx(&right_schema, &join.table_name, &join.on_right)
                .or_else(|_| find_col_idx(&right_schema, &join.table_name, &join.on_left))?;

            let mut merged_columns = Vec::new();
            for col in &left_schema.columns {
                 let mut c = col.clone();
                 c.name = format!("{}.{}", stmt.table_name, c.name);
                 merged_columns.push(c);
            }
            for col in &right_schema.columns {
                 let mut c = col.clone();
                 c.name = format!("{}.{}", join.table_name, c.name);
                 merged_columns.push(c);
            }
            let merged_schema = Schema::new(merged_columns);
            let mut joined_rows = Vec::new();

            for left_row in &left_rows {
                for right_row in &right_rows {
                    let left_val = left_row.get_value(left_col_idx).unwrap();
                    let right_val = right_row.get_value(right_col_idx).unwrap();
                    if left_val.compare(&TypesOperator::Eq, right_val).unwrap_or(false) {
                        let mut merged_values = left_row.values.clone();
                        merged_values.extend(right_row.values.clone());
                        joined_rows.push(Row::new(merged_values));
                    }
                }
            }
            (merged_schema, joined_rows)
        } else {
            (left_schema, left_rows)
        };

        if let Some(where_clause) = stmt.where_clause {
            working_rows = self.filter_rows(working_rows, &where_clause, &working_schema)?;
        }

        let result_schema = if stmt.columns.contains(&"*".to_string()) {
            working_schema.clone()
        } else {
            let mut columns = Vec::new();
            for col_name in &stmt.columns {
                let found_col = working_schema.columns.iter().find(|c| c.name == *col_name);
                let col = if let Some(c) = found_col { c } else {
                     let matches: Vec<&Column> = working_schema.columns.iter()
                        .filter(|c| c.name.ends_with(&format!(".{}", col_name)))
                        .collect();
                     if matches.len() == 1 { matches[0] } else if matches.is_empty() {
                         return Err(ExecutionError::ColumnNotFound(col_name.clone()));
                     } else {
                         return Err(ExecutionError::ExecutionFailed(format!("Ambiguous column: {}", col_name)));
                     }
                };
                columns.push(col.clone());
            }
            Schema::new(columns)
        };

        let result_rows = if stmt.columns.contains(&"*".to_string()) {
            working_rows
        } else {
            let mut projected_rows = Vec::new();
            for row in working_rows {
                let mut values = Vec::new();
                for col_name in &stmt.columns {
                     let col_idx = if let Some(idx) = working_schema.get_column_index(col_name) { idx } else {
                           working_schema.columns.iter().position(|c| c.name.ends_with(&format!(".{}", col_name)))
                             .ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?
                     };
                    let value = row.get_value(col_idx).ok_or_else(|| ExecutionError::ExecutionFailed(format!("Missing value for column {}", col_name)))?;
                    values.push(value.clone());
                }
                projected_rows.push(Row::new(values));
            }
            projected_rows
        };

        Ok(ResultSet::new(result_schema, result_rows))
    }

    fn filter_rows(&self, rows: Vec<Row>, where_clause: &WhereClause, schema: &Schema) -> Result<Vec<Row>, ExecutionError> {
        let convert_operator = |op: &ParserOperator| -> TypesOperator {
            match op {
                ParserOperator::Equals => TypesOperator::Eq,
                ParserOperator::NotEquals => TypesOperator::NotEq,
                ParserOperator::GreaterThan => TypesOperator::Gt,
                ParserOperator::LessThan => TypesOperator::Lt,
                ParserOperator::GreaterThanOrEqual => TypesOperator::GtEq,
                ParserOperator::LessThanOrEqual => TypesOperator::LtEq,
            }
        };

        let mut filtered_rows = rows;
        for condition in &where_clause.conditions {
            let col_idx = schema.get_column_index(&condition.column).ok_or_else(|| ExecutionError::ColumnNotFound(condition.column.clone()))?;
            let op = convert_operator(&condition.operator);
            filtered_rows = filtered_rows.into_iter().filter(|row| {
                if let Some(value) = row.get_value(col_idx) {
                    value.compare(&op, &condition.value).unwrap_or(false)
                } else { false }
            }).collect();
        }
        Ok(filtered_rows)
    }

    pub fn get_storage(&self) -> Database {
        self.storage.clone()
    }
}
