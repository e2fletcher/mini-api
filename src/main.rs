//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use mini_api::repository::{
    memory::MemoryTodoRepository, postgres::PostgresTodoRepository, TodoRepository,
    TodoRepositoryFactory,
};
use serde::Deserialize;
use std::{env::args, sync::Arc};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let use_memory = args()
        .collect::<Vec<String>>()
        .iter()
        .find(|&e| e == "--memory" || e == "-m")
        .map_or_else(|| false, |_| true);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mini_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db: Db = Arc::new(RwLock::new(if use_memory {
        tracing::info!("Use memory");
        MemoryTodoRepository::create_repository().await
    } else {
        PostgresTodoRepository::create_repository().await
    }));

    let app = Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route(
            "/todos/:id",
            delete(todos_delete).put(todos_update).get(todos_get),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    tracing::debug!("listening on");
}

#[derive(Debug, Default, Deserialize)]
struct Pagination {
    offset: Option<usize>,
    limit: Option<usize>,
}

async fn todos_index(
    State(db): State<Db>,
    pagination: Option<Query<Pagination>>,
) -> Result<impl IntoResponse, StatusCode> {
    let Query(pagination) = pagination.unwrap_or_default();

    match db
        .read()
        .await
        .list(pagination.limit, pagination.offset)
        .await
    {
        Ok(todos) => Ok(Json(todos)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn todos_create(
    State(db): State<Db>,
    Json(input): Json<CreateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Ok(todo) = db.write().await.create(input.text).await {
        Ok((StatusCode::CREATED, Json(todo)))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Deserialize, Debug)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    match db
        .write()
        .await
        .update(id, input.text, input.completed)
        .await
    {
        Ok(todo) => Ok(Json(todo)),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn todos_get(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
) -> Result<impl IntoResponse, StatusCode> {
    match db.read().await.get(id).await {
        Ok(todo) => Ok(Json(todo)),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn todos_delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    match db.write().await.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        _ => StatusCode::NOT_FOUND,
    }
}

type Db = Arc<RwLock<Box<dyn TodoRepository + Sync + Send>>>;
