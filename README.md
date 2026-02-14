# prowl 🐱

MCP server for SQL Server — let AI agents prowl your database.

Built on [tabby](https://github.com/copycatdb/tabby) (pure Rust TDS 7.4+), prowl gives AI agents like Claude and Copilot direct access to your SQL Server through the [Model Context Protocol](https://modelcontextprotocol.io/).

Single binary. Zero external dependencies. Just point it at your SQL Server and go.

## Installation

### Pre-built binaries (recommended)

Download the latest release for your platform:
- [Linux x64](https://github.com/copycatdb/prowl/releases/latest/download/prowl-x86_64-unknown-linux-gnu) | [Linux ARM64](https://github.com/copycatdb/prowl/releases/latest/download/prowl-aarch64-unknown-linux-gnu)
- [macOS x64](https://github.com/copycatdb/prowl/releases/latest/download/prowl-x86_64-apple-darwin) | [macOS Apple Silicon](https://github.com/copycatdb/prowl/releases/latest/download/prowl-aarch64-apple-darwin)
- [Windows x64](https://github.com/copycatdb/prowl/releases/latest/download/prowl-x86_64-pc-windows-msvc.exe) | [Windows ARM64](https://github.com/copycatdb/prowl/releases/latest/download/prowl-aarch64-pc-windows-msvc.exe)

Or use curl:
```bash
# Linux/macOS
curl -fsSL https://github.com/copycatdb/prowl/releases/latest/download/prowl-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m) -o prowl
chmod +x prowl
```

### From source
```bash
cargo install --git https://github.com/copycatdb/prowl.git
```

## Quick Start

```bash
# Configure
export TDSSERVER=localhost
export TDSPORT=1433
export TDSUSER=sa
export TDSPASSWORD=YourPassword123!

# Run (stdin/stdout JSON-RPC — connect via MCP client)
prowl
```

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "prowl": {
      "command": "/path/to/prowl",
      "args": ["--host", "localhost", "--user", "sa", "--password", "YourPassword123!"]
    }
  }
}
```

### VS Code / GitHub Copilot

Add to `.vscode/settings.json`:

```json
{
  "github.copilot.chat.mcpServers": {
    "prowl": {
      "command": "/path/to/prowl",
      "args": ["--host", "localhost", "--user", "sa", "--password", "yourpass"]
    }
  }
}
```

## Tools

prowl exposes 10 MCP tools for database exploration and monitoring:

### Schema Discovery

| Tool | Description |
|------|-------------|
| `list_databases` | List all databases on the instance |
| `list_tables` | List all tables in a database |
| `describe_table` | Columns, types, nullability, PKs, and FKs |

### Query Execution

| Tool | Description |
|------|-------------|
| `query` | Execute read-only SQL, returns markdown table (write ops blocked) |
| `query_plan` | Show execution plan for a query |

### Monitoring & Diagnostics

| Tool | Description |
|------|-------------|
| `active_sessions` | Active user sessions with CPU, reads, writes |
| `blocking_chains` | Sessions blocking other sessions |
| `index_usage` | Top missing indexes by improvement measure |
| `table_sizes` | Space used per table in a database |
| `server_info` | Version, edition, compatibility level |

### Example: `list_databases`

```
| name |
| --- |
| master |
| tempdb |
| model |
| msdb |
| MyApp |
```

### Example: `describe_table`

```
## Columns

| COLUMN_NAME | DATA_TYPE | CHARACTER_MAXIMUM_LENGTH | NUMERIC_PRECISION | NUMERIC_SCALE | IS_NULLABLE | IS_PRIMARY_KEY |
| --- | --- | --- | --- | --- | --- | --- |
| id | int | NULL | 10 | 0 | NO | YES |
| name | nvarchar | 255 | NULL | NULL | NO | NO |
| email | nvarchar | 255 | NULL | NULL | YES | NO |
| created_at | datetime2 | NULL | NULL | NULL | NO | NO |

## Foreign Keys

(no results)
```

### Safety

The `query` tool rejects any SQL containing write keywords (`INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER`, `CREATE`, `TRUNCATE`, `EXEC`, `EXECUTE`). All queries run at `READ UNCOMMITTED` isolation level with `NOCOUNT ON`.

## Configuration

prowl reads connection settings from environment variables or command-line arguments (CLI args take precedence):

| Environment Variable | CLI Argument | Default | Description |
|---------------------|-------------|---------|-------------|
| `TDSSERVER` | `--host` | `localhost` | SQL Server hostname |
| `TDSPORT` | `--port` | `1433` | SQL Server port |
| `TDSUSER` | `--user` | `sa` | Username |
| `TDSPASSWORD` | `--password` | (empty) | Password |
| `TDSDATABASE` | `--database` | (none) | Default database |
| — | `--no-trust-cert` | `false` | Disable trusting server certificate |

By default, prowl trusts the server certificate (dev-friendly). Use `--no-trust-cert` in production environments with proper certificates.

## Architecture

```
src/
  main.rs         — stdin/stdout JSON-RPC loop, arg parsing
  server.rs       — MCP protocol handler (initialize, tools/list, tools/call)
  connection.rs   — tabby connection management (connect, reconnect)
  tools/
    mod.rs        — Tool registry and dispatch
    schema.rs     — list_databases, list_tables, describe_table
    query.rs      — query, query_plan
    monitor.rs    — active_sessions, blocking_chains, index_usage, table_sizes, server_info
```

prowl implements the MCP protocol directly using JSON-RPC over stdin/stdout — no heavy SDK needed. It maintains a single persistent TDS connection and reconnects on error.

## CopyCat Ecosystem

prowl is part of the [CopyCat](https://github.com/copycatdb) family — all built on tabby:

| Project | What it does |
|---------|-------------|
| [**tabby**](https://github.com/copycatdb/tabby) | Pure Rust TDS 7.4+ protocol library |
| [**pounce**](https://github.com/copycatdb/pounce) | Arrow-native ADBC driver (Python) |
| [**prowl**](https://github.com/copycatdb/prowl) | MCP server for AI agents 👈 you are here |
| [**hiss**](https://github.com/copycatdb/hiss) | Async Python driver |
| [**whiskers**](https://github.com/copycatdb/whiskers) | DB-API 2.0 Python driver |
| [**claw**](https://github.com/copycatdb/claw) | Idiomatic Rust API |
| [**furball**](https://github.com/copycatdb/furball) | ODBC driver |
| [**kibble**](https://github.com/copycatdb/kibble) | Node.js driver |
| [**catnip**](https://github.com/copycatdb/catnip) | Go driver |
| [**hairball**](https://github.com/copycatdb/hairball) | JDBC driver |
| [**nuzzle**](https://github.com/copycatdb/nuzzle) | .NET ADO.NET driver |

## License

MIT
