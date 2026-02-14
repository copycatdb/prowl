use tabby::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::Args;

pub type TdsClient = Client<tokio_util::compat::Compat<TcpStream>>;

pub struct Connection {
    args: Args,
    client: Option<TdsClient>,
}

impl Connection {
    pub fn new(args: Args) -> Self {
        Self { args, client: None }
    }

    pub async fn get_client(&mut self) -> Result<&mut TdsClient, String> {
        if self.client.is_none() {
            self.connect().await?;
        }
        Ok(self.client.as_mut().unwrap())
    }

    async fn connect(&mut self) -> Result<(), String> {
        let mut config = Config::new();
        config.host(&self.args.host);
        config.port(self.args.port);
        config.authentication(AuthMethod::sql_server(&self.args.user, &self.args.password));

        if !self.args.no_trust_cert {
            config.trust_cert();
        }

        if let Some(ref db) = self.args.database {
            config.database(db);
        }

        let addr = config.get_addr().to_string();
        eprintln!("Connecting to {}", addr);

        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(|e| format!("TCP connection failed: {}", e))?;
        tcp.set_nodelay(true)
            .map_err(|e| format!("set_nodelay failed: {}", e))?;

        let client = Client::connect(config, tcp.compat_write())
            .await
            .map_err(|e| format!("TDS connection failed: {}", e))?;

        eprintln!("Connected successfully");
        self.client = Some(client);
        Ok(())
    }

    pub async fn reconnect(&mut self) -> Result<&mut TdsClient, String> {
        self.client = None;
        self.connect().await?;
        Ok(self.client.as_mut().unwrap())
    }
}
