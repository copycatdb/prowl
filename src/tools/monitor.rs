use crate::connection::Connection;
use crate::tools::schema::query_to_markdown;

pub async fn active_sessions(conn: &mut Connection) -> Result<String, String> {
    let sql = "SELECT s.session_id, s.login_name, s.status, \
               r.command, r.wait_type, r.blocking_session_id, \
               s.cpu_time, s.reads, s.writes \
               FROM sys.dm_exec_sessions s \
               LEFT JOIN sys.dm_exec_requests r ON s.session_id = r.session_id \
               WHERE s.is_user_process = 1 \
               ORDER BY s.cpu_time DESC";
    query_to_markdown(conn, sql, None).await
}

pub async fn blocking_chains(conn: &mut Connection) -> Result<String, String> {
    let sql = r#"SELECT
    r.session_id AS blocked_session,
    r.blocking_session_id AS blocking_session,
    s.login_name AS blocked_login,
    bs.login_name AS blocking_login,
    r.wait_type,
    r.wait_time,
    r.status AS blocked_status,
    r.command AS blocked_command
FROM sys.dm_exec_requests r
JOIN sys.dm_exec_sessions s ON r.session_id = s.session_id
LEFT JOIN sys.dm_exec_sessions bs ON r.blocking_session_id = bs.session_id
WHERE r.blocking_session_id <> 0
ORDER BY r.blocking_session_id, r.session_id"#;
    query_to_markdown(conn, sql, None).await
}

pub async fn index_usage(conn: &mut Connection, database: Option<&str>) -> Result<String, String> {
    let db_filter = match database {
        Some(db) => {
            let db = db.replace('\'', "''");
            format!("AND DB_NAME(d.database_id) = '{}'", db)
        }
        None => String::new(),
    };

    let sql = format!(
        r#"-- Missing indexes
SELECT TOP 20
    DB_NAME(d.database_id) AS [database],
    d.equality_columns,
    d.inequality_columns,
    d.included_columns,
    gs.unique_compiles,
    gs.user_seeks,
    gs.avg_total_user_cost * gs.avg_user_impact * (gs.user_seeks + gs.user_scans) AS improvement_measure
FROM sys.dm_db_missing_index_details d
JOIN sys.dm_db_missing_index_groups g ON d.index_handle = g.index_handle
JOIN sys.dm_db_missing_index_group_stats gs ON g.index_group_handle = gs.group_handle
WHERE 1=1 {}
ORDER BY improvement_measure DESC"#,
        db_filter
    );

    query_to_markdown(conn, &sql, Some(20)).await
}

pub async fn table_sizes(conn: &mut Connection, database: &str) -> Result<String, String> {
    let db = database.replace('\'', "''").replace(']', "]]");
    let sql = format!(
        r#"USE [{db}];

SELECT
    s.name AS [schema],
    t.name AS [table],
    SUM(p.rows) AS row_count,
    SUM(a.total_pages) * 8 AS total_space_kb,
    SUM(a.used_pages) * 8 AS used_space_kb,
    (SUM(a.total_pages) - SUM(a.used_pages)) * 8 AS unused_space_kb
FROM sys.tables t
JOIN sys.schemas s ON t.schema_id = s.schema_id
JOIN sys.indexes i ON t.object_id = i.object_id
JOIN sys.partitions p ON i.object_id = p.object_id AND i.index_id = p.index_id
JOIN sys.allocation_units a ON p.partition_id = a.container_id
WHERE i.index_id <= 1
GROUP BY s.name, t.name
ORDER BY SUM(a.total_pages) DESC"#,
        db = db
    );

    query_to_markdown(conn, &sql, None).await
}

pub async fn server_info(conn: &mut Connection) -> Result<String, String> {
    let sql = r#"SELECT
    @@VERSION AS [version],
    @@SERVERNAME AS [server_name],
    CAST(SERVERPROPERTY('Edition') AS nvarchar(256)) AS [edition],
    CAST(SERVERPROPERTY('ProductVersion') AS nvarchar(256)) AS [product_version],
    CAST(SERVERPROPERTY('ProductLevel') AS nvarchar(256)) AS [product_level],
    (SELECT compatibility_level FROM sys.databases WHERE name = DB_NAME()) AS [compatibility_level]"#;

    query_to_markdown(conn, sql, None).await
}
