use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json,
    response::Html,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use langdb::{
    storage::Database,
    executor::QueryExecutor,
    parser::parse_sql,
};

// Shared state
struct AppState {
    executor: Mutex<QueryExecutor>,
}

#[derive(Deserialize)]
struct QueryRequest {
    sql: String,
}

#[derive(Serialize)]
struct QueryResponse {
    status: String,
    result: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize database (load from disk or create new)
    let db = match Database::load_from_disk("../langdb.json") {
        Ok(db) => {
            println!("Loaded database from ../langdb.json");
            db
        },
        Err(_) => {
            println!("Starting with empty database");
            Database::new()
        }
    };

    let executor = QueryExecutor::new(db);
    let app_state = Arc::new(AppState {
        executor: Mutex::new(executor),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/query", post(query))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    let sql = payload.sql;
    println!("Executing Query: {}", sql);

    let executor = state.executor.lock().unwrap();

    // Parse
    let stmt = match parse_sql(&sql) {
        Ok(s) => s,
        Err(e) => return Json(QueryResponse {
            status: "error".to_string(),
            result: None,
            error: Some(format!("Parse error: {}", e)),
        }),
    };

    // Execute
    match executor.execute(stmt) {
        Ok(result) => {
            // Auto-save after modification (simple approach for now)
            // In a real app we'd have a background task or better persistence strategy
            let _ = executor.get_storage().save_to_disk("../langdb.json"); 
            
            Json(QueryResponse {
                status: "success".to_string(),
                result: Some(result.to_string()),
                error: None,
            })
        },
        Err(e) => Json(QueryResponse {
            status: "error".to_string(),
            result: None,
            error: Some(format!("Execution error: {}", e)),
        }),
    }
}
