# LangDB Quick Start Guide

This guide will help you get started with LangDB in just a few minutes.

## Prerequisites

- Rust and Cargo installed (1.70+ recommended)
- Basic knowledge of SQL

## Installation

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

## Your First Database

Once LangDB starts, you'll see the interactive prompt:

```
langdb>
```

### Step 1: Create a Table

```sql
CREATE TABLE users (id INTEGER, name TEXT, email TEXT);
```

### Step 2: Insert Data

```sql
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com'), (3, 'Charlie', 'charlie@example.com');
```

### Step 3: Query Data

```sql
SELECT * FROM users;
```

Output:
```
| id | name      | email               |
+----+-----------+---------------------+
| 1  | 'Alice'   | 'alice@example.com' |
| 2  | 'Bob'     | 'bob@example.com'   |
| 3  | 'Charlie' | 'charlie@example.com' |

3 row(s) returned
```

### Step 4: Filter Data

```sql
SELECT name, email FROM users WHERE id > 1;
```

Output:
```
| name      | email               |
+-----------+---------------------+
| 'Bob'     | 'bob@example.com'   |
| 'Charlie' | 'charlie@example.com' |

2 row(s) returned
```

## Special Commands

LangDB supports several special commands:

- `.help` - Display help information
- `.tables` - List all tables in the database
- `.exit` or `.quit` - Exit LangDB

Example:
```
langdb> .tables
Tables:
  users
  products
  orders
```

## Common Operations

### Creating Tables with Different Types

```sql
CREATE TABLE products (
    id INTEGER,
    name TEXT,
    price INTEGER,
    description TEXT NULL
);
```

### Inserting with Specific Columns

```sql
INSERT INTO products (id, name, price) VALUES (1, 'Laptop', 1200);
```

### Using WHERE Clauses

Supported operators: `=`, `<>`, `!=`, `>`, `<`, `>=`, `<=`

```sql
SELECT * FROM products WHERE price >= 1000;
SELECT name FROM products WHERE id = 1;
SELECT * FROM products WHERE name <> 'Laptop';
```

### Working with NULL Values

```sql
INSERT INTO products VALUES (2, 'Phone', 800, NULL);
SELECT * FROM products WHERE description = NULL;
```

## Multi-line SQL

You can write SQL statements across multiple lines. LangDB will wait for the semicolon:

```sql
langdb> SELECT id, name, price
....... FROM products
....... WHERE price > 500;
```

## Example Session

Here's a complete example session:

```sql
-- Create a table
CREATE TABLE books (id INTEGER, title TEXT, author TEXT, year INTEGER);

-- Insert some books
INSERT INTO books VALUES 
    (1, 'The Rust Programming Language', 'Steve Klabnik', 2018),
    (2, 'Programming Rust', 'Jim Blandy', 2017),
    (3, 'Rust in Action', 'Tim McNamara', 2021);

-- Query all books
SELECT * FROM books;

-- Find recent books
SELECT title, year FROM books WHERE year >= 2020;

-- List all tables
.tables

-- Exit
.exit
```

## Running SQL from a File

You can pipe SQL commands from a file:

```bash
./target/release/langdb < demo.sql
```

Or using cargo:

```bash
cargo run --release < demo.sql
```

## Testing the Installation

Run the provided test script:

```bash
./test_run.sh
```

This will build the project and run a quick test to verify everything is working.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out [demo.sql](demo.sql) for more examples
- Review the [CHANGELOG.md](CHANGELOG.md) for version history
- Explore the source code to understand the implementation

## Troubleshooting

### Build Errors

If you encounter build errors, make sure you have:
- Rust 1.70 or later installed
- Updated cargo: `rustup update`

### Runtime Errors

- Make sure SQL statements end with a semicolon (`;`)
- Check that table names and column names are spelled correctly
- Verify that data types match the schema

## Getting Help

- Use `.help` command in the REPL
- Check the [README.md](README.md) for detailed documentation
- Review error messages carefully - they usually indicate what went wrong

## Have Fun!

LangDB is an educational project. Experiment with it, break it, and learn from it. Happy querying!
