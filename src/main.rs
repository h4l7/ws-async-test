use anyhow::anyhow;
use clap::Parser;
use futures::stream::{StreamExt};
use futures::SinkExt;

use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
    time::Duration,
};
use tracing::{info, warn};

#[derive(clap::Parser, Debug)]
#[clap(author, version)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,
    #[clap(value_enum)]
    verbosity: Verbosity,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum Verbosity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Verbosity {
    fn into_level(self) -> tracing::Level {
        match self {
            Verbosity::Trace => tracing::Level::TRACE,
            Verbosity::Debug => tracing::Level::DEBUG,
            Verbosity::Info => tracing::Level::INFO,
            Verbosity::Warn => tracing::Level::WARN,
            Verbosity::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum Mode {
    Client,
    Server,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let level: tracing::Level = args.verbosity.into_level();
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .with_max_level(level)
        .pretty()
        .init();

    match args.mode {
        Mode::Client => {
            main_client().await?;
        }
        Mode::Server => {
            main_server().await?;
        }
    }

    Ok(())
}

async fn main_client() -> anyhow::Result<()> {
    let uri: http::Uri = "ws://127.0.0.1:1337/".parse()?;
    let authority = uri
        .authority()
        .ok_or(anyhow!("NoHostName"))?
        .as_str();
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);

    if host.is_empty() {
        return Err(anyhow!("EmptyHostName"));
    }

    let req = http::request::Builder::new()
        .method("GET")
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tungstenite::handshake::client::generate_key(),
        )
        .header("Sec-WebSocket-Protocol", "test")
        .uri(uri)
        .body(())?;

    let (mut stream, response) = tokio_tungstenite::connect_async(req).await?;

    info!(?response, "connected");

    let mut count: u64 = 0;

    loop {
        stream.send(tungstenite::protocol::Message::Text(count.to_string())).await?;
        count += 1;
        info!(?count);
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

async fn main_server() -> anyhow::Result<()> {
    let addr = SocketAddr::V4("127.0.0.1:1337".parse()?);
    let listener = TcpListener::bind(&addr).await?;
    let (inner_stream, bound_addr) = listener.accept().await?;
    let mut stream = tokio_tungstenite::accept_async(inner_stream).await?;

    info!(?bound_addr, "connection established");

    while let Some(frame) = stream.next().await {
        info!(?frame);
    }

    warn!("stream closed");

    Ok(())
}
