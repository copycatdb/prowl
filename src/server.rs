use serde_json::{json, Value};

use crate::connection::Connection;
use crate::tools;
use crate::Args;

pub struct Server {
    connection: Connection,
}

impl Server {
    pub fn new(args: Args) -> Self {
        Self {
            connection: Connection::new(args),
        }
    }

    pub async fn handle_request(&mut self, request: &Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request).await,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }),
        }
    }

    fn handle_initialize(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "prowl",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })
    }

    fn handle_tools_list(&self, id: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools::tool_definitions()
            }
        })
    }

    async fn handle_tools_call(&mut self, id: Value, request: &Value) -> Value {
        let params = request.get("params").cloned().unwrap_or(Value::Null);
        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        eprintln!("Tool call: {} with {:?}", tool_name, arguments);

        let result = tools::dispatch(tool_name, &arguments, &mut self.connection).await;

        match result {
            Ok(text) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": format!("Error: {}", e)
                    }],
                    "isError": true
                }
            }),
        }
    }
}
