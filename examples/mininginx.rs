use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing::log::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

struct Config {
    listen_addr: String,
    upstream_addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8001".to_string(),
            upstream_addr: "0.0.0.0:8080".to_string(),
        }
    }
}
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     let config = Config::default();
//     let config = Arc::new(config);
//     let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
//     tracing_subscriber::registry().with(layer).init();
//     info!("listening on {}", config.listen_addr);
//     info!("upstream is {}", config.upstream_addr);
//
//     // 创建tcp
//     let listener = TcpListener::bind(&config.listen_addr).await?;
//     loop {
//         let (client, addr) = listener.accept().await?;
//         info!("accepted connection from {}", addr);
//         let cloned_config = config.clone();
//         tokio::spawn(async move {
//             let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
//             proxy(client, upstream).await?;
//             Ok::<(), anyhow::Error>(())
//         });
//     }
//     #[allow(unreachable_code)]
//     Ok(())
// }
//
// async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
//     let (mut client_reader, mut client_writer) = client.split();
//     let (mut upstream_reader, mut upstream_writer) = upstream.split();
//
//     let client_to_upstream = tokio::io::copy(&mut client_reader, &mut upstream_writer);
//     let upstream_to_client = tokio::io::copy(&mut upstream_reader, &mut client_writer);
//     match tokio::try_join!(client_to_upstream, upstream_to_client) {
//         Ok((n, m)) => info!(
//             "proxied {} bytes from client to upstream, {} bytes from upstream to client",
//             n, m
//         ),
//         Err(e) => warn!("error proxying: {:?}", e),
//     }
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = Config::default();
    let config = Arc::new(config);
    info!("Upstream is {}", config.upstream_addr);
    info!("Listening on {}", config.listen_addr);
    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        let cloned_config = config.clone();
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    let client_to_upstream = tokio::io::copy(&mut client_read, &mut upstream_write);
    let upstream_to_client = tokio::io::copy(&mut upstream_read, &mut client_write);
    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((n, m)) => info!(
            "proxied {} bytes from client to upstream, {} bytes from upstream to client",
            n, m
        ),
        Err(e) => warn!("error proxying: {:?}", e),
    }
    Ok(())
}
