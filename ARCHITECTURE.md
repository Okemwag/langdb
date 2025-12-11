# LangDB Architecture

This document provides a detailed overview of LangDB's architecture, design decisions, and implementation details.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Layers](#architecture-layers)
3. [Data Flow](#data-flow)
4. [Module Details](#module-details)
5. [Design Patterns](#design-patterns)
6. [Concurrency Model](#concurrency-model)
7. [Error Handling](#error-handling)
8. [Future Improvements](#future-improvements)

## Overview

LangDB is built using a layered architecture pattern, where each layer has a specific responsibility and communicates with adjacent layers through well-defined interfaces. This design promotes:

- **Separation of Concerns**: Each module handles a specific aspect of database functionality
- **Testability**: Modules can be tested independently
- **Maintainability**: Changes in one layer don't affect others
- **Extensibility**: New features can be added without major refactoring

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Layer 1: REPL                         │
│                  User Interface Layer                    │
│  - Command-line interface                                │
│  - Input/output handling                                 │
│  - Result formatting                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Layer 2: Parser                        │
│                  Syntax Analysis Layer                   │
│  - Lexical analysis                                      │
│  - Syntax parsing                                        │
│  - AST generation                                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  Layer 3: Executor                       │
│                 Query Execution Layer                    │
│  - Query planning                                        │
│  - Execution logic                                       │
│  - Result generation                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  Layer 4: Storage                        │
│                   Data Access Layer                      │
│  - Table management                                      │
│  - Data persistence                                      │
│  - Concurrency control                                   │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Layer 5: Types                         │
│                 Foundation Layer                         │
│  - Data structures                                       │
│  - Type system                                           │
│  - Validation logic                                      │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Query Execution Flow

```
┌──────────┐
│   User   │
└────┬─────┘
     │ SQL String
     ▼
┌──────────────────┐
│   REPL (main)    │
│  - Read input    │
│  - Route command │
└────┬─────────────┘
     │ SQL String
     ▼
┌──────────────────┐
│     Parser       │
│  - Tokenize      │
│  - Parse         │
│  - Validate      │
└────┬─────────────┘
     │ Statement (AST)
     ▼
┌──────────────────┐
│    Executor      │
│  - Plan query    │
│  - Execute ops   │
│  - Filter/Project│
└────┬─────────────┘
     │ Storage Operations
     ▼
┌──────────────────┐
│    Storage       │
│  - Read/Write    │
│  - Validate      │
│  - Return data   │
└────┬─────────────┘
     │ Rows
     ▼
┌──────────────────┐
│    Executor      │
│  - Format result │
└────┬─────────────┘
     │ ResultSet
     ▼
┌──────────────────┐
│      REPL        │
│  - Display       │
└────┬─────────────┘
     │ Formatted Output
     ▼
┌──────────┐
│   User   │
└──────────┘
```

### Example: SELECT Query Flow

```sql
SELECT name, age FROM users WHERE age > 25;
```

1. **REPL Layer**: Receives input string, detects semicolon, sends to parser
2. **Parser Layer**: 
   - Tokenizes: `SELECT`, `name`, `,`, `age`, `FROM`, `users`, `WHERE`, `age`, `>`, `25`
   - Parses into `SelectStatement`:
     ```rust
     SelectStatement {
         columns: vec!["name", "age"],
         table_name: "users",
         where_clause: Some(WhereClause {
             conditions: vec![Condition {
                 column: "age",
                 operator: GreaterThan,
                 value: Integer(25)
             }]
         })
     }
     ```
3. **Executor Layer**:
   - Retrieves table schema from storage
   - Scans all rows from `users` table
   - Filters rows where `age > 25`
   - Projects only `name` and `age` columns
   - Creates `ResultSet` with filtered/projected data
4. **Storage Layer**:
   - Acquires read lock on tables HashMap
   - Returns rows from `users` table
   - Releases lock
5. **Executor Layer**: Returns `ResultSet` to REPL
6. **REPL Layer**: Formats and displays results as ASCII table

## Module Details

### 1. REPL Module (`main.rs`)

**Responsibilities:**
- User interaction
- Command routing
- Multi-line input handling
- Result display

**Key Functions:**
- `run_repl()`: Main REPL loop
- `process_special_command()`: Handle `.` commands
- `process_sql_command()`: Execute SQL statements
- `print_welcome()`: Display welcome message

**Design Decisions:**
- Uses `std::io::BufRead` for efficient line reading
- Accumulates input until semicolon is found
- Separates special commands from SQL commands

### 2. Parser Module (`parser/mod.rs`)

**Responsibilities:**
- Lexical analysis
- Syntax parsing
- AST generation
- Error reporting

**Key Components:**
- `Statement`: Enum representing different SQL statements
- `parse_sql()`: Main entry point for parsing
- Parser combinators for each SQL construct

**Design Decisions:**
- Uses `nom` parser combinator library for composability
- Case-insensitive keyword matching
- Whitespace-tolerant parsing
- Detailed error messages with context

**Parsing Strategy:**
```rust
// Example: Parsing SELECT statement
parse_select = 
    keyword("SELECT") >>
    columns >>
    keyword("FROM") >>
    table_name >>
    optional(where_clause) >>
    return SelectStatement
```

### 3. Executor Module (`executor/mod.rs`)

**Responsibilities:**
- Query execution
- Result generation
- Filtering and projection
- Error handling

**Key Components:**
- `QueryExecutor`: Main execution engine
- `execute()`: Routes statements to handlers
- `execute_select()`: SELECT query execution
- `execute_insert()`: INSERT execution
- `execute_create_table()`: CREATE TABLE execution

**Execution Strategies:**

**CREATE TABLE:**
```
1. Parse column definitions
2. Create Schema object
3. Call storage.create_table()
4. Return empty ResultSet
```

**INSERT:**
```
1. Get table schema from storage
2. Validate column count
3. Map values to columns (if columns specified)
4. Validate data types
5. Call storage.insert()
6. Return empty ResultSet
```

**SELECT:**
```
1. Get table schema from storage
2. Scan all rows from table
3. Apply WHERE clause filters
4. Project requested columns
5. Create ResultSet with filtered/projected data
6. Return ResultSet
```

### 4. Storage Module (`storage/mod.rs`)

**Responsibilities:**
- Data persistence
- Table management
- Concurrency control
- Schema validation

**Key Components:**
- `Database`: Main storage engine
- `Table`: Individual table structure
- `TableMetadata`: Table schema information

**Data Structures:**
```rust
Database {
    tables: Arc<RwLock<HashMap<String, Table>>>
}

Table {
    metadata: TableMetadata,
    rows: Vec<Row>
}

TableMetadata {
    name: String,
    schema: Schema
}
```

**Concurrency Model:**
- Uses `Arc<RwLock<>>` for thread-safe access
- Multiple readers can access simultaneously
- Writers get exclusive access
- Prevents data races at compile time

### 5. Types Module (`types/mod.rs`)

**Responsibilities:**
- Core data structures
- Type system
- Validation logic
- Value operations

**Key Types:**

**DataType:**
```rust
enum DataType {
    Integer,  // i64
    Text,     // String
}
```

**Value:**
```rust
enum Value {
    Integer(i64),
    Text(String),
    Null,
}
```

**Schema:**
```rust
struct Schema {
    columns: Vec<Column>
}
```

**Row:**
```rust
struct Row {
    values: Vec<Value>
}
```

**ResultSet:**
```rust
struct ResultSet {
    schema: Schema,
    rows: Vec<Row>
}
```

## Design Patterns

### 1. Layered Architecture
Each layer depends only on the layer below it, promoting loose coupling.

### 2. Parser Combinator Pattern
Small, composable parsing functions that can be combined to parse complex structures.

### 3. Repository Pattern
Storage layer abstracts data access, allowing for different storage implementations.

### 4. Visitor Pattern
Executor "visits" different statement types and executes them accordingly.

### 5. Builder Pattern
Schema and Row construction uses builder-like patterns.

### 6. Type State Pattern
Rust's type system ensures operations are valid at compile time.

## Concurrency Model

### Thread Safety

LangDB uses Rust's ownership system and synchronization primitives to ensure thread safety:

```rust
Arc<RwLock<HashMap<String, Table>>>
 │    │      │
 │    │      └─ Data structure
 │    └─ Read-Write lock
 └─ Atomic Reference Counting
```

**Benefits:**
- Multiple readers can access data simultaneously
- Writers get exclusive access
- No data races (guaranteed by Rust compiler)
- Deadlock prevention through lock ordering

### Lock Granularity

Currently, LangDB uses a single lock for the entire database:
- **Pros**: Simple, no deadlocks
- **Cons**: Limited concurrency

**Future Improvement**: Per-table locks for better concurrency.

## Error Handling

LangDB uses Rust's `Result` type and the `thiserror` crate for error handling:

### Error Types

```rust
ParseError      // Syntax errors
ExecutionError  // Query execution errors
StorageError    // Data access errors
TypeError       // Type validation errors
```

### Error Propagation

```rust
fn execute_query() -> Result<ResultSet, ExecutionError> {
    let stmt = parse_sql(sql)?;  // ? operator propagates errors
    let result = executor.execute(stmt)?;
    Ok(result)
}
```

### Error Context

Uses `anyhow` for adding context to errors:

```rust
storage.create_table(name, schema)
    .context("Failed to create users table")?
```

## Future Improvements

### 1. Performance Optimizations
- **Indexing**: B-tree indexes for faster lookups
- **Query Optimization**: Cost-based query planning
- **Caching**: Query result caching
- **Parallel Execution**: Multi-threaded query execution

### 2. Storage Enhancements
- **Persistent Storage**: File-based storage with WAL
- **Compression**: Row-level compression
- **Transactions**: ACID transaction support
- **Backup/Restore**: Database backup functionality

### 3. SQL Features
- **JOIN Operations**: INNER, LEFT, RIGHT, FULL joins
- **Aggregations**: COUNT, SUM, AVG, MIN, MAX
- **Grouping**: GROUP BY and HAVING clauses
- **Sorting**: ORDER BY with ASC/DESC
- **Subqueries**: Nested SELECT statements
- **Views**: Virtual tables

### 4. Type System
- **More Types**: FLOAT, BOOLEAN, DATE, TIMESTAMP, BLOB
- **Type Constraints**: CHECK constraints
- **Default Values**: Column defaults
- **Auto Increment**: Automatic ID generation

### 5. Advanced Features
- **Stored Procedures**: User-defined functions
- **Triggers**: Event-driven actions
- **Constraints**: PRIMARY KEY, FOREIGN KEY, UNIQUE
- **Permissions**: User access control
- **Replication**: Master-slave replication

## Conclusion

LangDB's architecture is designed to be simple, maintainable, and extensible. The layered approach allows for easy understanding and modification, while Rust's type system ensures safety and correctness. The current implementation provides a solid foundation for future enhancements and serves as an excellent educational resource for understanding database internals.
