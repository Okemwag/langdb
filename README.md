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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

LangDB was created as an educational project to learn about database internals, SQL parsing, and Rust programming. It is not intended for production use.

