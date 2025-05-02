use anyhow::Result;
use axum::extract::Path;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use log::info;
use nanoid::nanoid;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing::log::info;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

#[derive(Debug)]
struct AppState {
    db: PgPool,
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        // create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
            );
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self { db: pool })
    }

    async fn create(&self, url: &str) -> Result<String> {
        let id = nanoid!(6);
        sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2)")
            .bind(&id)
            .bind(url)
            .execute(&self.db)
            .await?;
        Ok(id)
    }

    async fn get(&self, id: &str) -> Result<String> {
        let record: (String,) = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(record.0)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // 初始化数据库
    let db_url = "postgres://postgres:postgres@localhost:5432/shortener";
    let state = Arc::new(AppState::try_new(db_url).await?);
    info!("postgres database url: {}", &db_url);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);
    info!("server starting at {}", addr);
    if let Err(e) = axum::serve::serve(listener, app.into_make_service()).await? {
        warn!("server exited with error: {}", e);
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

/// state 的 destruct 要放在最前面
async fn shorten(
    state: AppState,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .create(&data.url)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let body = Json(ShortenRes {
        url: format!("http://localhost:8080/{}", &id),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    state: AppState,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state.get(&id).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, url.parse().unwrap());
    Ok((StatusCode::FOUND, headers))
}
