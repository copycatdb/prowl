mod connection;
mod server;
mod tools;

use clap::Parser;
use std::io::BufRead;

/// 🐱 prowl — MCP server for SQL Server
#[derive(Parser, Debug, Clone)]
#[command(name = "prowl", version, about)]
pub struct Args {
    /// SQL Server hostname
    #[arg(long, env = "TDSSERVER", default_value = "localhost")]
    pub host: String,

    /// SQL Server port
    #[arg(long, env = "TDSPORT", default_value = "1433")]
    pub port: u16,

    /// SQL Server username
    #[arg(long, env = "TDSUSER", default_value = "sa")]
    pub user: String,

    /// SQL Server password
    #[arg(long, env = "TDSPASSWORD", default_value = "")]
    pub password: String,

    /// Default database
    #[arg(long, env = "TDSDATABASE")]
    pub database: Option<String>,

    /// Disable trusting the server certificate
    #[arg(long, default_value = "false")]
    pub no_trust_cert: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    eprintln!("prowl 🐱 MCP server starting...");
    eprintln!("Connecting to {}:{} as {}", args.host, args.port, args.user);

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut srv = server::Server::new(args);

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("stdin read error: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("JSON parse error: {}", e);
                let err_resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                let _ = write_response(&stdout, &err_resp);
                continue;
            }
        };

        // Notifications have no "id" — handle silently
        if request.get("id").is_none() {
            let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
            eprintln!("Notification: {}", method);
            continue;
        }

        let response = srv.handle_request(&request).await;
        if let Err(e) = write_response(&stdout, &response) {
            eprintln!("stdout write error: {}", e);
            break;
        }
    }

    eprintln!("prowl shutting down 🐱");
}

fn write_response(
    stdout: &std::io::Stdout,
    response: &serde_json::Value,
) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut out = stdout.lock();
    serde_json::to_writer(&mut out, response)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}
