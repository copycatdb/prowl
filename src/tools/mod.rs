pub mod monitor;
pub mod query;
pub mod schema;

use serde_json::{json, Value};

use crate::connection::Connection;

pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_databases",
            "description": "List all databases on the SQL Server instance",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "list_tables",
            "description": "List all tables in a database",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "database": {
                        "type": "string",
                        "description": "Database name"
                    }
                },
                "required": ["database"]
            }
        },
        {
            "name": "describe_table",
            "description": "Describe a table's columns, types, nullability, primary keys, and foreign keys",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "database": { "type": "string", "description": "Database name" },
                    "schema": { "type": "string", "description": "Schema name (default: dbo)" },
                    "table": { "type": "string", "description": "Table name" }
                },
                "required": ["database", "table"]
            }
        },
        {
            "name": "query",
            "description": "Execute a read-only SQL query and return results as a markdown table. Write operations are blocked.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sql": { "type": "string", "description": "SQL query to execute" },
                    "max_rows": { "type": "integer", "description": "Maximum rows to return (default: 100)" }
                },
                "required": ["sql"]
            }
        },
        {
            "name": "query_plan",
            "description": "Show the execution plan for a SQL query",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sql": { "type": "string", "description": "SQL query to get execution plan for" }
                },
                "required": ["sql"]
            }
        },
        {
            "name": "active_sessions",
            "description": "Show active user sessions on the SQL Server",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "blocking_chains",
            "description": "Show blocking chains — sessions blocking other sessions",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "index_usage",
            "description": "Show top unused and missing indexes",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "database": { "type": "string", "description": "Database name (optional)" }
                },
                "required": []
            }
        },
        {
            "name": "table_sizes",
            "description": "Show space used per table in a database",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "database": { "type": "string", "description": "Database name" }
                },
                "required": ["database"]
            }
        },
        {
            "name": "server_info",
            "description": "Show SQL Server version, edition, and configuration",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }
    ])
}

pub async fn dispatch(
    tool_name: &str,
    arguments: &Value,
    conn: &mut Connection,
) -> Result<String, String> {
    match tool_name {
        "list_databases" => schema::list_databases(conn).await,
        "list_tables" => {
            let db = get_str(arguments, "database")?;
            schema::list_tables(conn, &db).await
        }
        "describe_table" => {
            let db = get_str(arguments, "database")?;
            let schema = arguments
                .get("schema")
                .and_then(|v| v.as_str())
                .unwrap_or("dbo");
            let table = get_str(arguments, "table")?;
            schema::describe_table(conn, &db, schema, &table).await
        }
        "query" => {
            let sql = get_str(arguments, "sql")?;
            let max_rows = arguments
                .get("max_rows")
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize;
            query::execute_query(conn, &sql, max_rows).await
        }
        "query_plan" => {
            let sql = get_str(arguments, "sql")?;
            query::query_plan(conn, &sql).await
        }
        "active_sessions" => monitor::active_sessions(conn).await,
        "blocking_chains" => monitor::blocking_chains(conn).await,
        "index_usage" => {
            let db = arguments.get("database").and_then(|v| v.as_str());
            monitor::index_usage(conn, db).await
        }
        "table_sizes" => {
            let db = get_str(arguments, "database")?;
            monitor::table_sizes(conn, &db).await
        }
        "server_info" => monitor::server_info(conn).await,
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

fn get_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}
