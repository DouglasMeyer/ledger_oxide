use std::net::SocketAddr;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

mod db;
mod graphql;

use graphql::schema::{MutationRoot, QueryRoot};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(pool.clone())
        .finish();

    let app = Router::new()
        .route("/graphql", axum::routing::any_service(GraphQL::new(schema)))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
