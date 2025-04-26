use crate::types::{DataType, Value};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, tuple},
};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("SQL syntax error: {0}")]
    SyntaxError(String),
    #[error("Unsupported SQL feature: {0}")]
    #[allow(dead_code)]
    UnsupportedFeature(String),
    #[error("Invalid token: {0}")]
    #[allow(dead_code)]
    InvalidToken(String),
}

/// Different types of SQL statements
#[derive(Debug, Clone)]
pub enum Statement {
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
    Select(SelectStatement),
    // Can be extended with more statement types
}

/// CREATE TABLE statement
#[derive(Debug, Clone)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
}

/// Column definition for CREATE TABLE
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

/// INSERT statement
#[derive(Debug, Clone)]
pub struct InsertStatement {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Value>>,
}

/// SELECT statement
#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub columns: Vec<String>,
    pub table_name: String,
    pub where_clause: Option<WhereClause>,
}

/// WHERE clause condition
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub conditions: Vec<Condition>,
}

/// Condition in WHERE clause
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub operator: Operator,
    pub value: Value,
}

/// Comparison operators
#[derive(Debug, Clone)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

// Basic parser functions

/// Parse SQL identifier (table name, column name, etc.)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse whitespace
fn parse_whitespace(input: &str) -> IResult<&str, ()> {
    map(multispace0, |_| ())(input)
}

/// Case-insensitive keyword parser
fn keyword<'a>(word: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (input, _) = parse_whitespace(input)?;
        tag_no_case(word)(input)
    }
}

/// Parse a string literal (enclosed in single quotes)
fn parse_string_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = parse_whitespace(input)?;
    delimited(
        char('\''),
        map(take_while(|c| c != '\''), |s: &str| s.to_string()),
        char('\''),
    )(input)
}

/// Parse an integer literal
fn parse_integer_literal(input: &str) -> IResult<&str, i64> {
    let (input, _) = parse_whitespace(input)?;
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

/// Parse a SQL value (string, integer, or NULL)
fn parse_value(input: &str) -> IResult<&str, Value> {
    let (input, _) = parse_whitespace(input)?;
    alt((
        map(parse_string_literal, Value::Text),
        map(parse_integer_literal, Value::Integer),
        map(tag_no_case("NULL"), |_| Value::Null),
    ))(input)
}

/// Parse a data type (INTEGER, TEXT, etc.)
fn parse_data_type(input: &str) -> IResult<&str, DataType> {
    let (input, _) = parse_whitespace(input)?;
    map_res(
        alt((
            tag_no_case("INTEGER"),
            tag_no_case("INT"),
            tag_no_case("TEXT"),
            tag_no_case("VARCHAR"),
            tag_no_case("STRING"),
        )),
        |s: &str| DataType::from_str(s),
    )(input)
}

/// Parse a column definition for CREATE TABLE
fn parse_column_def(input: &str) -> IResult<&str, ColumnDef> {
    let (input, _) = parse_whitespace(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, data_type) = parse_data_type(input)?;
    let (input, nullable) = opt(preceded(multispace1, tag_no_case("NULL")))(input)?;

    let nullable = nullable.is_some();
    Ok((
        input,
        ColumnDef {
            name,
            data_type,
            nullable,
        },
    ))
}

/// Parse a CREATE TABLE statement
fn parse_create_table(input: &str) -> IResult<&str, CreateTableStatement> {
    let (input, _) = tuple((
        keyword("CREATE"),
        multispace1,
        keyword("TABLE"),
        multispace1,
    ))(input)?;

    let (input, table_name) = parse_identifier(input)?;
    let (input, columns) = delimited(
        tuple((parse_whitespace, char('('), parse_whitespace)),
        separated_list1(
            tuple((parse_whitespace, char(','), parse_whitespace)),
            parse_column_def,
        ),
        tuple((parse_whitespace, char(')'))),
    )(input)?;

    Ok((
        input,
        CreateTableStatement {
            table_name,
            columns,
        },
    ))
}

/// Parse a list of column names
fn parse_column_list(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        tuple((parse_whitespace, char('('), parse_whitespace)),
        separated_list1(
            tuple((parse_whitespace, char(','), parse_whitespace)),
            parse_identifier,
        ),
        tuple((parse_whitespace, char(')'))),
    )(input)
}

/// Parse a list of values for INSERT
fn parse_value_list(input: &str) -> IResult<&str, Vec<Value>> {
    delimited(
        tuple((parse_whitespace, char('('), parse_whitespace)),
        separated_list1(
            tuple((parse_whitespace, char(','), parse_whitespace)),
            parse_value,
        ),
        tuple((parse_whitespace, char(')'))),
    )(input)
}

/// Parse multiple value lists for INSERT
fn parse_values_clause(input: &str) -> IResult<&str, Vec<Vec<Value>>> {
    let (input, _) = tuple((keyword("VALUES"), multispace1))(input)?;

    separated_list1(
        tuple((parse_whitespace, char(','), parse_whitespace)),
        parse_value_list,
    )(input)
}

/// Parse an INSERT statement
fn parse_insert(input: &str) -> IResult<&str, InsertStatement> {
    let (input, _) = tuple((keyword("INSERT"), multispace1, keyword("INTO"), multispace1))(input)?;

    let (input, table_name) = parse_identifier(input)?;
    let (input, columns) = opt(parse_column_list)(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, values) = parse_values_clause(input)?;

    Ok((
        input,
        InsertStatement {
            table_name,
            columns,
            values,
        },
    ))
}

/// Parse a comparison operator
fn parse_operator(input: &str) -> IResult<&str, Operator> {
    let (input, _) = parse_whitespace(input)?;
    alt((
        map(tag("="), |_| Operator::Equals),
        map(tag("<>"), |_| Operator::NotEquals),
        map(tag("!="), |_| Operator::NotEquals),
        map(tag(">="), |_| Operator::GreaterThanOrEqual),
        map(tag("<="), |_| Operator::LessThanOrEqual),
        map(tag(">"), |_| Operator::GreaterThan),
        map(tag("<"), |_| Operator::LessThan),
    ))(input)
}

/// Parse a single condition in a WHERE clause
fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (input, _) = parse_whitespace(input)?;
    let (input, column) = parse_identifier(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, operator) = parse_operator(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, value) = parse_value(input)?;

    Ok((
        input,
        Condition {
            column,
            operator,
            value,
        },
    ))
}

/// Parse a WHERE clause
fn parse_where_clause(input: &str) -> IResult<&str, WhereClause> {
    let (input, _) = tuple((keyword("WHERE"), multispace1))(input)?;

    // For simplicity, we'll only handle one condition for now
    // In a more complete implementation, we would parse multiple conditions with AND/OR
    let (input, condition) = parse_condition(input)?;
    Ok((
        input,
        WhereClause {
            conditions: vec![condition],
        },
    ))
}

/// Parse a SELECT statement
fn parse_select(input: &str) -> IResult<&str, SelectStatement> {
    let (input, _) = tuple((keyword("SELECT"), multispace1))(input)?;

    // Parse column list or * for all columns
    let (input, columns) = alt((
        map(tag("*"), |_| vec!["*".to_string()]),
        separated_list1(
            tuple((parse_whitespace, char(','), parse_whitespace)),
            parse_identifier,
        ),
    ))(input)?;

    let (input, _) = tuple((multispace1, keyword("FROM"), multispace1))(input)?;

    let (input, table_name) = parse_identifier(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, where_clause) = opt(parse_where_clause)(input)?;

    Ok((
        input,
        SelectStatement {
            columns,
            table_name,
            where_clause,
        },
    ))
}

/// Parse an SQL statement
fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = parse_whitespace(input)?;
    alt((
        map(parse_create_table, Statement::CreateTable),
        map(parse_insert, Statement::Insert),
        map(parse_select, Statement::Select),
    ))(input)
}

/// Parse an SQL statement and ensure the input is completely consumed
pub fn parse_sql(input: &str) -> Result<Statement, ParseError> {
    let (remainder, stmt) = parse_statement(input)
        .map_err(|e| ParseError::SyntaxError(format!("SQL parsing error: {:?}", e)))?;

    // Check if the entire input was consumed
    if remainder.trim().is_empty() {
        Ok(stmt)
    } else {
        Err(ParseError::SyntaxError(format!(
            "Unexpected trailing input: '{}'",
            remainder
        )))
    }
}
