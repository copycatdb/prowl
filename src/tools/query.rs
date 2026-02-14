use crate::connection::Connection;
use crate::tools::schema::query_to_markdown;

const BLOCKED_KEYWORDS: &[&str] = &[
    "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "TRUNCATE", "EXEC", "EXECUTE",
];

fn is_read_only(sql: &str) -> bool {
    let upper = sql.to_uppercase();
    for keyword in BLOCKED_KEYWORDS {
        // Check for keyword as a whole word (preceded by start/whitespace/semicolon)
        for part in upper.split_whitespace() {
            if part.trim_matches(|c: char| !c.is_alphanumeric()) == *keyword {
                return false;
            }
        }
    }
    true
}

pub async fn execute_query(
    conn: &mut Connection,
    sql: &str,
    max_rows: usize,
) -> Result<String, String> {
    if !is_read_only(sql) {
        return Err(
            "Write operations are not allowed. Only SELECT and read-only queries are permitted."
                .to_string(),
        );
    }

    let wrapped = format!(
        "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED; SET NOCOUNT ON;\n{}",
        sql
    );

    query_to_markdown(conn, &wrapped, Some(max_rows)).await
}

pub async fn query_plan(conn: &mut Connection, sql: &str) -> Result<String, String> {
    let wrapped = format!("SET SHOWPLAN_TEXT ON;\n{}\nSET SHOWPLAN_TEXT OFF;", sql);

    query_to_markdown(conn, &wrapped, None).await
}
