-- LangDB Demo Script
-- This file demonstrates the basic functionality of LangDB

-- Create a users table
CREATE TABLE users (id INTEGER, name TEXT, age INTEGER);

-- Insert some users
INSERT INTO users VALUES (1, 'Alice', 30);
INSERT INTO users VALUES (2, 'Bob', 25), (3, 'Charlie', 35);

-- Query all users
SELECT * FROM users;

-- Query with WHERE clause
SELECT name, age FROM users WHERE age > 25;

-- Create a products table
CREATE TABLE products (id INTEGER, name TEXT, price INTEGER);

-- Insert products
INSERT INTO products VALUES (1, 'Laptop', 1200), (2, 'Phone', 800), (3, 'Mouse', 25);

-- Query products
SELECT * FROM products;

-- Query expensive products
SELECT name, price FROM products WHERE price > 500;

-- Show all tables
.tables
