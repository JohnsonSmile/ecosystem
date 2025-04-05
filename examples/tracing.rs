use anyhow::Result;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::{BatchConfigBuilder, RandomIdGenerator, Sampler, SdkTracer};
use opentelemetry_sdk::{runtime, trace, Resource};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;
use tracing::{debug, info, instrument};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter::LevelFilter, fmt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();

    // 终端
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        // .pretty()
        .with_filter(LevelFilter::INFO);

    // 文件rotate
    let file_appender = tracing_appender::rolling::daily("./tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        // .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    // tracer
    let tracer = init_tracer()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // 注册
    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(telemetry)
        .init();

    let addr = "127.0.0.1:8080";
    let app = Router::new().route("/", get(index_handler));

    info!("Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> impl IntoResponse {
    debug!("index handler started");
    sleep(Duration::from_millis(100)).await;
    let ret = long_task().await;
    info!("index handler returned: {}", ret);
    "Hello, World!"
}

#[instrument]
async fn long_task() -> i64 {
    sleep(Duration::from_millis(500)).await;
    123
}

fn init_tracer() -> Result<SdkTracer> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://124.71.216.50:4317")
        .with_timeout(Duration::from_secs(3))
        .build()?;
    let batch = BatchSpanProcessor::builder(exporter, runtime::Tokio)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(4096)
                .build(),
        )
        .build();
    let tracer_provider = trace::SdkTracerProvider::builder()
        .with_span_processor(batch)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(
            Resource::builder_empty()
                .with_attributes([KeyValue::new("service.name", "my-service")])
                .build(),
        )
        .build();
    let tracer = tracer_provider.tracer("sdk-trace");
    // global::set_tracer_provider(tracer_provider.clone());
    // let tracer = global::tracer("tracer-name");
    Ok(tracer)
}
