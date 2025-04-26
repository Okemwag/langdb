// // Integration tests for LangDB

// use langdb::{
//     executor::QueryExecutor,
//     parser::parse_sql,
//     storage::Database,
//     types::{ResultSet, Value},
// };

// // Helper function to execute SQL and return results
// fn execute_sql(executor: &QueryExecutor, sql: &str) -> Result<ResultSet, String> {
//     match parse_sql(sql) {
//         Ok(stmt) => match executor.execute(stmt) {
//             Ok(result) => Ok(result),
//             Err(e) => Err(format!("Execution error: {}", e)),
//         },
//         Err(e) => Err(format!("Parse error: {}", e)),
//     }
// }

// // Helper function to execute SQL and ignore results (for CREATE TABLE, INSERT, etc.)
// fn execute_sql_no_result(executor: &QueryExecutor, sql: &str) -> Result<(), String> {
//     match execute_sql(executor, sql) {
//         Ok(_) => Ok(()),
//         Err(e) => Err(e),
//     }
// }

// #[test]
// fn test_basic_table_operations() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // Create a table
//     let create_sql = "CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)";
//     assert!(execute_sql_no_result(&executor, create_sql).is_ok());

//     // Insert data
//     let insert_sql = "INSERT INTO users VALUES (1, 'Alice', 30)";
//     assert!(execute_sql_no_result(&executor, insert_sql).is_ok());

//     // Insert more data
//     let insert_sql2 = "INSERT INTO users VALUES (2, 'Bob', 25), (3, 'Charlie', 35)";
//     assert!(execute_sql_no_result(&executor, insert_sql2).is_ok());

//     // Select all data
//     let select_sql = "SELECT * FROM users";
//     let result = execute_sql(&executor, select_sql).unwrap();
//     assert_eq!(result.rows.len(), 3);

//     // Verify data in the first row
//     assert_eq!(result.rows[0].values[0], Value::Integer(1));
//     assert_eq!(result.rows[0].values[1], Value::Text("Alice".to_string()));
//     assert_eq!(result.rows[0].values[2], Value::Integer(30));

//     // Select with WHERE clause
//     let select_where_sql = "SELECT id, name FROM users WHERE age > 25";
//     let result = execute_sql(&executor, select_where_sql).unwrap();
//     assert_eq!(result.rows.len(), 2);

//     // Verify schema projection (only id and name should be in results)
//     assert_eq!(result.schema.columns.len(), 2);
//     assert_eq!(result.schema.columns[0].name, "id");
//     assert_eq!(result.schema.columns[1].name, "name");
// }

// #[test]
// fn test_error_handling() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // Invalid SQL syntax
//     let invalid_sql = "SELEKT * FROM table";
//     let result = execute_sql_no_result(&executor, invalid_sql);
//     assert!(result.is_err());
//     assert!(result.unwrap_err().contains("Parse error"));

//     // Table not found
//     let not_found_sql = "SELECT * FROM nonexistent_table";
//     let result = execute_sql_no_result(&executor, not_found_sql);
//     assert!(result.is_err());
//     assert!(result.unwrap_err().contains("Table not found"));

//     // Create a table
//     let create_sql = "CREATE TABLE items (id INTEGER, name TEXT, price INTEGER)";
//     assert!(execute_sql_no_result(&executor, create_sql).is_ok());

//     // Schema validation error (wrong number of columns)
//     let invalid_insert = "INSERT INTO items VALUES (1, 'Item')";
//     let result = execute_sql_no_result(&executor, invalid_insert);
//     assert!(result.is_err());

//     // Type mismatch (would fail when parser implemented strictly)
//     // Currently our parser handles this by converting types
//     let insert_with_type_mismatch = "INSERT INTO items VALUES (1, 'Laptop', 'expensive')";
//     let result = execute_sql_no_result(&executor, insert_with_type_mismatch);
//     assert!(result.is_err(), "Type mismatch should be rejected");

//     // Duplicate table creation
//     let duplicate_create = "CREATE TABLE items (id INTEGER, name TEXT)";
//     let result = execute_sql_no_result(&executor, duplicate_create);
//     assert!(result.is_err());
//     assert!(result.unwrap_err().contains("already exists"));
// }

// #[test]
// fn test_null_handling() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // Create a table with nullable column
//     let create_sql = "CREATE TABLE products (id INTEGER, name TEXT, description TEXT NULL)";
//     assert!(execute_sql_no_result(&executor, create_sql).is_ok());

//     // Insert with NULL
//     let insert_sql = "INSERT INTO products VALUES (1, 'Laptop', NULL)";
//     assert!(execute_sql_no_result(&executor, insert_sql).is_ok());

//     // Insert without NULL
//     let insert_sql2 = "INSERT INTO products VALUES (2, 'Phone', 'Smart Phone')";
//     assert!(execute_sql_no_result(&executor, insert_sql2).is_ok());

//     // Select all
//     let select_sql = "SELECT * FROM products";
//     let result = execute_sql(&executor, select_sql).unwrap();
//     assert_eq!(result.rows.len(), 2);

//     // Verify NULL value
//     assert_eq!(result.rows[0].values[2], Value::Null);

//     // Verify non-NULL value
//     assert_eq!(
//         result.rows[1].values[2],
//         Value::Text("Smart Phone".to_string())
//     );
// }

// #[test]
// fn test_complex_scenario() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // Create multiple tables
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "CREATE TABLE customers (id INTEGER, name TEXT, email TEXT)"
//         )
//         .is_ok()
//     );

//     assert!(execute_sql_no_result(&executor,
//         "CREATE TABLE orders (id INTEGER, customer_id INTEGER, product TEXT, quantity INTEGER, total INTEGER)"
//     ).is_ok());

//     // Insert data into customers
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "INSERT INTO customers VALUES
//         (1, 'Alice', 'alice@example.com'),
//         (2, 'Bob', 'bob@example.com'),
//         (3, 'Charlie', 'charlie@example.com')"
//         )
//         .is_ok()
//     );

//     // Insert data into orders
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "INSERT INTO orders VALUES
//         (101, 1, 'Laptop', 1, 1200),
//         (102, 2, 'Phone', 1, 800),
//         (103, 1, 'Mouse', 2, 50),
//         (104, 3, 'Monitor', 1, 300)"
//         )
//         .is_ok()
//     );

//     // Verify customers
//     let result = execute_sql(&executor, "SELECT * FROM customers").unwrap();
//     assert_eq!(result.rows.len(), 3);

//     // Verify orders
//     let result = execute_sql(&executor, "SELECT * FROM orders").unwrap();
//     assert_eq!(result.rows.len(), 4);

//     // Query with WHERE clause
//     let result = execute_sql(&executor, "SELECT * FROM orders WHERE total > 500").unwrap();
//     assert_eq!(result.rows.len(), 2); // Laptop and Phone

//     // Query with projection
//     let result = execute_sql(
//         &executor,
//         "SELECT product, quantity FROM orders WHERE customer_id = 1",
//     )
//     .unwrap();
//     assert_eq!(result.rows.len(), 2); // Alice's orders
//     assert_eq!(result.schema.columns.len(), 2); // Only product and quantity

//     // Verify projection content
//     let product1 = &result.rows[0].values[0];
//     let quantity1 = &result.rows[0].values[1];

//     assert!(matches!(product1, Value::Text(p) if p == "Laptop" || p == "Mouse"));
//     assert!(matches!(quantity1, Value::Integer(q) if *q == 1 || *q == 2));
// }

// #[test]
// fn test_sequential_operations() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // 1. Create a table
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "CREATE TABLE inventory (
//             item_id INTEGER,
//             name TEXT,
//             quantity INTEGER,
//             price INTEGER,
//             category TEXT NULL
//         )"
//         )
//         .is_ok()
//     );

//     // 2. Insert initial data
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "INSERT INTO inventory VALUES
//         (1, 'Widget A', 100, 10, 'Widgets'),
//         (2, 'Gadget B', 50, 25, 'Gadgets'),
//         (3, 'Tool C', 30, 15, NULL)"
//         )
//         .is_ok()
//     );

//     // 3. Query the data
//     let result = execute_sql(&executor, "SELECT * FROM inventory").unwrap();
//     assert_eq!(result.rows.len(), 3);

//     // 4. Insert more data with specific columns
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "INSERT INTO inventory (item_id, name, quantity, price) VALUES
//         (4, 'Device D', 20, 50)"
//         )
//         .is_ok()
//     );

//     // 5. Query again to verify the new data
//     let result = execute_sql(&executor, "SELECT * FROM inventory").unwrap();
//     assert_eq!(result.rows.len(), 4);

//     // 6. Query with WHERE clause
//     let result = execute_sql(
//         &executor,
//         "SELECT name, price FROM inventory WHERE price > 20",
//     )
//     .unwrap();
//     assert_eq!(result.rows.len(), 2); // Gadget B and Device D

//     // 7. Query with different WHERE clause
//     let result = execute_sql(
//         &executor,
//         "SELECT name FROM inventory WHERE category = 'Widgets'",
//     )
//     .unwrap();
//     assert_eq!(result.rows.len(), 1);
//     assert_eq!(
//         result.rows[0].values[0],
//         Value::Text("Widget A".to_string())
//     );
// }

// #[test]
// fn test_different_data_types() {
//     // Set up database
//     let db = Database::new();
//     let executor = QueryExecutor::new(db);

//     // Create a table with different types
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "CREATE TABLE measurements (
//             id INTEGER,
//             name TEXT,
//             value INTEGER,
//             notes TEXT NULL
//         )"
//         )
//         .is_ok()
//     );

//     // Insert mixed data
//     assert!(
//         execute_sql_no_result(
//             &executor,
//             "INSERT INTO measurements VALUES
//         (1, 'Temperature', 72, 'Fahrenheit'),
//         (2, 'Pressure', 1013, 'Millibars'),
//         (3, 'Humidity', 45, NULL)"
//         )
//         .is_ok()
//     );

//     // Query the data
//     let result = execute_sql(&executor, "SELECT * FROM measurements").unwrap();
//     assert_eq!(result.rows.len(), 3);

//     // Verify integer values
//     for row in &result.rows {
//         assert!(matches!(row.values[0], Value::Integer(_)));
//         assert!(matches!(row.values[1], Value::Text(_)));
//         assert!(matches!(row.values[2], Value::Integer(_)));
//         // notes column can be either Text or Null
//     }

//     // Verify third row has NULL in the last column
//     assert_eq!(result.rows[2].values[3], Value::Null);
// }
