# LangDB: A Simple SQL Database in Rust 

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

LangDB is an educational SQL database implementation written in Rust. It provides a minimal but functional in-memory SQL database with support for basic SQL operations. This project demonstrates core database concepts including SQL parsing, query execution, and data storage.
## Features

- 💾 **In-memory database** with table management
- 🔍 **SQL parser** built with the nom parsing library
- 📊 **Support for basic SQL statements**:
  - CREATE TABLE with column types
  - INSERT with value lists
  - SELECT with WHERE clauses
- 📋 **REPL interface** with special commands
- 🔒 **Thread-safe operations** for concurrent access
- 📝 **Data types**: INTEGER, TEXT, and NULL values
- ⚡ **Extensible architecture** for future enhancements

## Installation

### Prerequisites

- Rust and Cargo (1.70+ recommended)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Okemwag/langdb.git
   cd langdb
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run LangDB:
   ```bash
   cargo run --release
   ```

4. (Optional) Run the test script:
   ```bash
   ./test_run.sh
   ```

For a detailed getting started guide, see [QUICKSTART.md](QUICKSTART.md).

## Usage

LangDB provides an interactive SQL prompt where you can enter SQL commands:

```
=================================================
LangDB - A Simple SQL Database
=================================================
Type SQL commands to execute them.
Commands end with semicolon (;)
Special commands:
  .help - Display this help message
  .exit, .quit - Exit the program
  .tables - Show all tables
Examples:
  CREATE TABLE users (id INTEGER, name TEXT);
  INSERT INTO users VALUES (1, 'Alice');
  SELECT * FROM users;
=================================================
langdb> 
```

### Special Commands

- `.help` - Display help information
- `.exit` or `.quit` - Exit the program
- `.tables` - List all tables in the database

### Basic SQL Commands

#### Create a Table

```sql
CREATE TABLE users (id INTEGER, name TEXT, age INTEGER);
```

#### Insert Data

```sql
INSERT INTO users VALUES (1, 'Alice', 30);
INSERT INTO users VALUES (2, 'Bob', 25), (3, 'Charlie', 35);
```

#### Insert with Specific Columns

```sql
INSERT INTO users (id, name) VALUES (4, 'Dave');
```

#### Query Data

```sql
SELECT * FROM users;
SELECT name, age FROM users WHERE age > 25;
```

## SQL Support

### Supported Features

- **CREATE TABLE** with column definitions
  - Column types: INTEGER, TEXT
  - NULL/NOT NULL constraints
- **INSERT** statements
  - Full row inserts
  - Column-specific inserts
  - Multiple row inserts
- **SELECT** statements
  - Column projection (specific columns or *)
  - Basic WHERE clause with comparisons (=, <>, >, <, >=, <=)
  - Table scans

### Limitations

- No support for JOIN operations
- No support for aggregate functions (SUM, COUNT, etc.)
- No support for ORDER BY or GROUP BY
- No persistent storage (in-memory only)
- Limited data types (INTEGER and TEXT only)
- Basic WHERE clause (no AND/OR support)

## System Design

LangDB follows a layered architecture pattern, separating concerns into distinct modules that work together to provide a complete SQL database system.

> For a detailed architecture overview, see [ARCHITECTURE.md](ARCHITECTURE.md)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    REPL Interface                        │
│                     (main.rs)                            │
│  - User input handling                                   │
│  - Command routing                                       │
│  - Result formatting                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   SQL Parser                             │
│                  (parser/mod.rs)                         │
│  - Lexical analysis                                      │
│  - Syntax parsing (nom combinators)                     │
│  - AST generation                                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                 Query Executor                           │
│                (executor/mod.rs)                         │
│  - Statement routing                                     │
│  - Query planning                                        │
│  - WHERE clause evaluation                               │
│  - Column projection                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                 Storage Engine                           │
│                (storage/mod.rs)                          │
│  - Table management                                      │
│  - Row storage (in-memory)                              │
│  - Thread-safe operations (RwLock)                      │
│  - Schema validation                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Type System                            │
│                  (types/mod.rs)                          │
│  - Data type definitions                                 │
│  - Value representation                                  │
│  - Schema structures                                     │
│  - Type validation & conversion                          │
└─────────────────────────────────────────────────────────┘
```

### Module Breakdown

#### 1. REPL Interface (`main.rs`)
The entry point and user interaction layer:
- Implements a Read-Eval-Print Loop for interactive SQL execution
- Handles special commands (`.help`, `.exit`, `.tables`)
- Manages multi-line SQL input (statements ending with `;`)
- Formats and displays query results
- Provides error handling and user feedback

#### 2. SQL Parser (`parser/mod.rs`)
Transforms SQL text into executable statements:
- Built using the `nom` parser combinator library
- Supports three statement types:
  - `CREATE TABLE`: Table definition with columns and types
  - `INSERT`: Data insertion with optional column specification
  - `SELECT`: Data retrieval with column projection and filtering
- Generates Abstract Syntax Tree (AST) representations
- Provides detailed error messages for syntax errors

Key parsing components:
- Identifier parsing (table/column names)
- Literal parsing (strings, integers, NULL)
- Operator parsing (=, <>, >, <, >=, <=)
- Clause parsing (WHERE conditions)

#### 3. Query Executor (`executor/mod.rs`)
Executes parsed SQL statements:
- Routes statements to appropriate execution handlers
- Implements query logic:
  - **CREATE TABLE**: Validates schema and creates table structure
  - **INSERT**: Validates data types and inserts rows
  - **SELECT**: Performs table scans, applies filters, and projects columns
- Handles WHERE clause evaluation
- Manages column projection (selecting specific columns or `*`)
- Converts execution results into `ResultSet` objects

Execution flow:
1. Receive parsed statement from parser
2. Validate against storage schema
3. Execute operation on storage layer
4. Format results for display

#### 4. Storage Engine (`storage/mod.rs`)
Manages data persistence and retrieval:
- In-memory storage using `HashMap<String, Table>`
- Thread-safe operations via `Arc<RwLock<>>`
- Table structure:
  - Metadata (name, schema)
  - Row collection (Vec<Row>)
- Operations:
  - Table creation/deletion
  - Row insertion (single and batch)
  - Table scanning
  - Row filtering
- Schema validation on all write operations

Concurrency model:
- Read-write locks allow multiple concurrent readers
- Exclusive write access for modifications
- Prevents data races and ensures consistency

#### 5. Type System (`types/mod.rs`)
Defines core data structures:
- **DataType**: Supported SQL types (INTEGER, TEXT)
- **Value**: Runtime value representation (Integer, Text, Null)
- **Column**: Column definition with name, type, and nullability
- **Schema**: Collection of columns defining table structure
- **Row**: Collection of values representing a table row
- **ResultSet**: Query results with schema and rows

Type operations:
- Type validation and conversion
- Value comparison (for WHERE clauses)
- Schema validation
- Result formatting

### Data Flow

#### Query Execution Flow
```
User Input → Parser → Executor → Storage → Result
     ↓          ↓         ↓          ↓         ↓
  "SELECT"   Statement  Execute   Scan     Format
   SQL text    AST      Query     Table    Display
```

#### Example: SELECT Query
1. User enters: `SELECT name FROM users WHERE id = 1;`
2. Parser creates `SelectStatement` with:
   - columns: `["name"]`
   - table_name: `"users"`
   - where_clause: `Condition { column: "id", op: Equals, value: Integer(1) }`
3. Executor:
   - Retrieves table schema from storage
   - Scans all rows from `users` table
   - Filters rows where `id = 1`
   - Projects only `name` column
4. Storage returns matching rows
5. Executor creates `ResultSet` with filtered/projected data
6. REPL formats and displays results

### Design Patterns

1. **Layered Architecture**: Clear separation of concerns across modules
2. **Parser Combinator**: Composable parsing functions using `nom`
3. **Repository Pattern**: Storage layer abstracts data access
4. **Visitor Pattern**: Executor visits different statement types
5. **Builder Pattern**: Schema and row construction
6. **Thread-Safe Singleton**: Database instance with Arc<RwLock<>>

### Concurrency & Thread Safety

LangDB uses Rust's ownership system and synchronization primitives:
- `Arc<RwLock<HashMap>>` for shared database access
- Multiple readers can access data simultaneously
- Writers get exclusive access
- Prevents data races at compile time

### Error Handling

Comprehensive error types for each layer:
- `ParseError`: Syntax and parsing errors
- `ExecutionError`: Query execution failures
- `StorageError`: Data access and validation errors
- `TypeError`: Type conversion and validation errors

All errors implement `thiserror::Error` for consistent error handling.

## Code Structure

LangDB is organized into several modules, each handling a specific database component:

- **parser**: SQL parsing using the nom library
  - Parses SQL strings into structured Statement objects
  - Handles lexical analysis and syntactic validation

- **types**: Core data types and structures
  - Defines Value, DataType, and other foundational types
  - Implements schema validation and type conversion

- **storage**: Data storage and retrieval
  - Manages tables and their data
  - Provides thread-safe access to the database

- **executor**: Query execution
  - Executes parsed SQL statements
  - Routes operations to the storage engine
  - Handles result formatting

- **main**: REPL interface
  - Provides the interactive command-line interface
  - Processes user input and displays results

## Example Queries and Results

### Create a Table and Insert Data

```
langdb> CREATE TABLE products (id INTEGER, name TEXT, price INTEGER);
langdb> INSERT INTO products VALUES (1, 'Laptop', 1200), (2, 'Phone', 800);
Inserted rows. Total rows: 2
```

### Query Data

```
langdb> SELECT * FROM products;
| id | name   | price |
+----+--------+-------+
| 1  | Laptop | 1200  |
| 2  | Phone  | 800   |

2 row(s) returned
```

### Query with WHERE Clause

```
langdb> SELECT name, price FROM products WHERE price > 1000;
| name   | price |
+--------+-------+
| Laptop | 1200  |

1 row(s) returned
```

## Development and Testing

### Running Tests

LangDB includes comprehensive tests to verify functionality:

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test --test integration_tests
```

### Test Coverage

- **Unit tests**: Individual module testing
- **Integration tests**: End-to-end testing of the database functionality
- **Edge case testing**: Validation of error handling and corner cases

### Project Structure

```
langdb/
├── Cargo.toml          # Project configuration
├── src/
│   ├── main.rs         # REPL implementation
│   ├── parser/         # SQL parsing
│   ├── types/          # Core data types
│   ├── storage/        # Data storage
│   └── executor/       # Query execution
└── tests/
    └── integration_tests.rs  # Integration tests
```

## Future Enhancements

- Persistent storage (file-based)
- Support for more SQL features (JOIN, GROUP BY, etc.)
- Additional data types (FLOAT, BOOLEAN, DATE, etc.)
- Indexing for improved query performance
- Transaction support
- More complex WHERE clause expressions

## Contributing

Contributions are welcome! Here's how you can contribute to LangDB:

1. Fork the repository
2. Create a feature branch: `git checkout -b my-new-feature`
3. Make your changes and commit them: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin my-new-feature`
5. Submit a pull request

Please make sure your code follows the existing style and includes appropriate tests.

## Project Journey

Want to understand the thought process, challenges, and learnings behind this project? Read the detailed journey:
- [PROJECT_JOURNEY.md](PROJECT_JOURNEY.md) - A deep dive into why and how this database was built

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed history of changes and version information.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

LangDB was created as an educational project to learn about database internals, SQL parsing, and Rust programming. It is not intended for production use.

