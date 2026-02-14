use crate::connection::Connection;

/// Execute a SQL query and return results as a markdown table string.
pub async fn query_to_markdown(
    conn: &mut Connection,
    sql: &str,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let client = match conn.get_client().await {
        Ok(c) => c,
        Err(_) => conn.reconnect().await?,
    };

    let params: Vec<&dyn tabby::IntoSql> = vec![];
    let stream = client
        .execute(sql, &params)
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Result error: {}", e))?;

    if rows.is_empty() {
        return Ok("(no results)".to_string());
    }

    // Get column names from first row
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    if columns.is_empty() {
        return Ok("(no columns returned)".to_string());
    }

    let max = max_rows.unwrap_or(rows.len());
    let display_rows = &rows[..rows.len().min(max)];

    let mut md = String::new();

    // Header
    md.push('|');
    for col in &columns {
        md.push_str(&format!(" {} |", col));
    }
    md.push('\n');

    // Separator
    md.push('|');
    for _ in &columns {
        md.push_str(" --- |");
    }
    md.push('\n');

    // Rows
    for row in display_rows {
        md.push('|');
        for (i, _col) in columns.iter().enumerate() {
            let val: Option<&str> = row.try_get::<&str, _>(i).ok().flatten();
            md.push_str(&format!(" {} |", val.unwrap_or("NULL")));
        }
        md.push('\n');
    }

    if rows.len() > max {
        md.push_str(&format!("\n_Showing {} of {} rows_\n", max, rows.len()));
    }

    Ok(md)
}

pub async fn list_databases(conn: &mut Connection) -> Result<String, String> {
    query_to_markdown(conn, "SELECT name FROM sys.databases ORDER BY name", None).await
}

pub async fn list_tables(conn: &mut Connection, database: &str) -> Result<String, String> {
    let db = database.replace('\'', "''").replace(']', "]]");
    let sql = format!(
        "USE [{}]; SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE \
         FROM INFORMATION_SCHEMA.TABLES ORDER BY TABLE_SCHEMA, TABLE_NAME",
        db
    );
    query_to_markdown(conn, &sql, None).await
}

pub async fn describe_table(
    conn: &mut Connection,
    database: &str,
    schema: &str,
    table: &str,
) -> Result<String, String> {
    let db = database.replace('\'', "''").replace(']', "]]");
    let sch = schema.replace('\'', "''");
    let tbl = table.replace('\'', "''");

    let sql = format!(
        r#"USE [{db}];

SELECT
    c.COLUMN_NAME,
    c.DATA_TYPE,
    c.CHARACTER_MAXIMUM_LENGTH,
    c.NUMERIC_PRECISION,
    c.NUMERIC_SCALE,
    c.IS_NULLABLE,
    CASE WHEN pk.COLUMN_NAME IS NOT NULL THEN 'YES' ELSE 'NO' END AS IS_PRIMARY_KEY
FROM INFORMATION_SCHEMA.COLUMNS c
LEFT JOIN (
    SELECT ku.TABLE_SCHEMA, ku.TABLE_NAME, ku.COLUMN_NAME
    FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc
    JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE ku
        ON tc.CONSTRAINT_NAME = ku.CONSTRAINT_NAME
        AND tc.TABLE_SCHEMA = ku.TABLE_SCHEMA
    WHERE tc.CONSTRAINT_TYPE = 'PRIMARY KEY'
) pk ON c.TABLE_SCHEMA = pk.TABLE_SCHEMA
    AND c.TABLE_NAME = pk.TABLE_NAME
    AND c.COLUMN_NAME = pk.COLUMN_NAME
WHERE c.TABLE_SCHEMA = '{sch}' AND c.TABLE_NAME = '{tbl}'
ORDER BY c.ORDINAL_POSITION"#,
        db = db,
        sch = sch,
        tbl = tbl
    );

    let columns_md = query_to_markdown(conn, &sql, None).await?;

    let fk_sql = format!(
        r#"USE [{db}];

SELECT
    fk.name AS FK_NAME,
    COL_NAME(fkc.parent_object_id, fkc.parent_column_id) AS COLUMN_NAME,
    OBJECT_SCHEMA_NAME(fkc.referenced_object_id) AS REF_SCHEMA,
    OBJECT_NAME(fkc.referenced_object_id) AS REF_TABLE,
    COL_NAME(fkc.referenced_object_id, fkc.referenced_column_id) AS REF_COLUMN
FROM sys.foreign_keys fk
JOIN sys.foreign_key_columns fkc ON fk.object_id = fkc.constraint_object_id
WHERE fk.parent_object_id = OBJECT_ID('{sch}.{tbl}')
ORDER BY fk.name"#,
        db = db,
        sch = sch,
        tbl = tbl
    );

    let fk_md = query_to_markdown(conn, &fk_sql, None).await?;

    Ok(format!(
        "## Columns\n\n{}\n\n## Foreign Keys\n\n{}",
        columns_md, fk_md
    ))
}
