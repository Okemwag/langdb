# Changelog

All notable changes to LangDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-11

### Added
- Initial release of LangDB
- In-memory SQL database implementation in Rust
- SQL parser using nom combinator library
- Support for basic SQL statements:
  - CREATE TABLE with column definitions
  - INSERT with single and multiple row support
  - SELECT with column projection and WHERE clauses
- Data types: INTEGER, TEXT, and NULL
- REPL interface with interactive SQL prompt
- Special commands:
  - `.help` - Display help information
  - `.exit` / `.quit` - Exit the program
  - `.tables` - List all tables
- Thread-safe database operations using Arc<RwLock<>>
- Comprehensive error handling with custom error types
- Schema validation for all data operations
- WHERE clause support with comparison operators (=, <>, >, <, >=, <=)
- Column projection (SELECT specific columns or *)
- Multi-line SQL input support (statements ending with semicolon)
- Integration test suite
- MIT License
- Comprehensive README with usage examples

### Features
- **Parser Module**: SQL parsing with lexical analysis and AST generation
- **Types Module**: Core data structures (Value, DataType, Schema, Row, ResultSet)
- **Storage Module**: In-memory table management with concurrent access
- **Executor Module**: Query execution engine with filtering and projection
- **REPL Interface**: Interactive command-line interface

### Technical Details
- Built with Rust 2024 edition
- Dependencies:
  - nom 7.1.3 (parser combinators)
  - thiserror 1.0.49 (error handling)
  - chrono 0.4.31 (date/time support)
  - serde 1.0.189 (serialization)
  - serde_json 1.0.107 (JSON support)
  - anyhow 1.0.75 (error context)

### Known Limitations
- No persistent storage (in-memory only)
- No JOIN operations
- No aggregate functions (SUM, COUNT, AVG, etc.)
- No ORDER BY or GROUP BY clauses
- No complex WHERE expressions (AND/OR)
- Limited data types (INTEGER and TEXT only)
- No indexing or query optimization
- No transaction support
- No UPDATE or DELETE statements

## [Unreleased]

### Planned Features
- Persistent storage (file-based)
- Additional SQL statements (UPDATE, DELETE)
- JOIN operations (INNER, LEFT, RIGHT)
- Aggregate functions (COUNT, SUM, AVG, MIN, MAX)
- GROUP BY and HAVING clauses
- ORDER BY with ASC/DESC
- More data types (FLOAT, BOOLEAN, DATE, TIMESTAMP)
- Complex WHERE expressions with AND/OR
- Indexing for query optimization
- Transaction support (BEGIN, COMMIT, ROLLBACK)
- LIMIT and OFFSET for pagination
- ALTER TABLE support
- Subqueries
- Views
- Constraints (PRIMARY KEY, FOREIGN KEY, UNIQUE)

---

## Version History

- **0.1.0** (2024-12-11): Initial release with core SQL functionality
