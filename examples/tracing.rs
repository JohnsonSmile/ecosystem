use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tracing::{info, instrument};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter::LevelFilter, fmt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();

    // 终端
    let console_layer = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        // .pretty()
        .with_filter(LevelFilter::INFO);

    // 文件rotate
    let file_appender = tracing_appender::rolling::daily("./tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        // .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    // 注册
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    let addr = "127.0.0.1:8080";
    let app = Router::new().route("/", get(index_handler));

    info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}
