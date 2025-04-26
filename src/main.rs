mod executor;
mod parser;
mod storage;
mod types;

use anyhow::{Context, Result};
use executor::QueryExecutor;
use parser::parse_sql;
use std::io::{self, BufRead, Write};
use storage::Database;
use types::{Column, DataType, Schema};

/// Print the welcome message and usage instructions
fn print_welcome() {
    println!("=================================================");
    println!("LangDB - A Simple SQL Database");
    println!("=================================================");
    println!("Type SQL commands to execute them.");
    println!("Commands end with semicolon (;)");
    println!("Special commands:");
    println!("  .help - Display this help message");
    println!("  .exit, .quit - Exit the program");
    println!("  .tables - Show all tables");
    println!("Examples:");
    println!("  CREATE TABLE users (id INTEGER, name TEXT);");
    println!("  INSERT INTO users VALUES (1, 'Alice');");
    println!("  SELECT * FROM users;");
    println!("=================================================");
}

/// Process a special command (starting with .)
fn process_special_command(cmd: &str, executor: &QueryExecutor) -> Result<bool> {
    match cmd.trim().to_lowercase().as_str() {
        ".exit" | ".quit" => {
            println!("Exiting LangDB. Goodbye!");
            return Ok(true); // true signals exit
        }
        ".help" => {
            print_welcome();
        }
        ".tables" => {
            let table_names = executor
                .get_storage()
                .get_table_names()
                .context("Failed to get table list")?;

            if table_names.is_empty() {
                println!("No tables defined");
            } else {
                println!("Tables:");
                for name in table_names {
                    println!("  {}", name);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", cmd);
            println!("Type .help for usage information");
        }
    }
    Ok(false) // false means continue
}

/// Create a schema from string column definitions
fn create_schema_from_strs(column_defs: Vec<&str>) -> Result<Schema> {
    let mut columns = Vec::new();

    for def in column_defs {
        let parts: Vec<&str> = def.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid column definition: {}", def));
        }

        let name = parts[0].to_string();
        let data_type = match parts[1].to_uppercase().as_str() {
            "INTEGER" => DataType::Integer,
            "TEXT" => DataType::Text,
            _ => return Err(anyhow::anyhow!("Unsupported data type: {}", parts[1])),
        };

        let nullable = parts.len() > 2 && parts[2].to_uppercase() == "NULL";
        columns.push(Column::new(name, data_type, nullable));
    }

    Ok(Schema::new(columns))
}

/// Run the REPL (Read-Eval-Print Loop)
fn run_repl() -> Result<()> {
    let storage = Database::new();

    // Create initial tables if they don't exist
    if !storage.table_exists("users")? {
        let schema = create_schema_from_strs(vec!["id INTEGER", "name TEXT"])?;
        storage
            .create_table("users".to_string(), schema)
            .context("Failed to create users table")?;
    }

    if !storage.table_exists("products")? {
        let schema = create_schema_from_strs(vec!["id INTEGER", "name TEXT"])?;
        storage
            .create_table("products".to_string(), schema)
            .context("Failed to create products table")?;
    }

    if !storage.table_exists("orders")? {
        let schema =
            create_schema_from_strs(vec!["id INTEGER", "user_id INTEGER", "product_id INTEGER"])?;
        storage
            .create_table("orders".to_string(), schema)
            .context("Failed to create orders table")?;
    }

    let executor = QueryExecutor::new(storage);
    print_welcome();

    let mut input_buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        if input_buffer.is_empty() {
            print!("langdb> ");
            io::stdout().flush()?;
        } else {
            print!("....... ");
            io::stdout().flush()?;
        }

        let mut line = String::new();
        handle.read_line(&mut line)?;

        if line.is_empty() {
            println!("Exiting due to EOF. Goodbye!");
            break;
        }

        let line = line.trim();

        if line.starts_with(".") {
            if process_special_command(line, &executor)? {
                break;
            }
            continue;
        }

        input_buffer.push_str(line);
        input_buffer.push(' ');

        if !line.ends_with(';') {
            continue;
        }

        // Remove trailing semicolon
        input_buffer.pop(); // Remove the space
        input_buffer.pop(); // Remove the semicolon

        match process_sql_command(&input_buffer, &executor) {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }

        input_buffer.clear();
    }

    Ok(())
}

/// Process a SQL command
fn process_sql_command(sql: &str, executor: &QueryExecutor) -> Result<()> {
    let statement = match parse_sql(sql) {
        Ok(stmt) => stmt,
        Err(e) => {
            return Err(anyhow::anyhow!("Parse error: {}", e));
        }
    };

    match executor.execute(statement) {
        Ok(result) => {
            if !result.is_empty() {
                println!("{}", result.to_string());
            }
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Execution error: {}", e)),
    }
}

fn main() -> Result<()> {
    match run_repl() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Fatal error: {}", e);
            std::process::exit(1);
        }
    }
}
