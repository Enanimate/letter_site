use std::env;

use axum::{
    extract::Path, routing::get, Router
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest_service("/pkg", ServeDir::new("../render_app/lib-web/pkg"))
        .nest_service("/index.js", ServeFile::new("test/index.js"))
        .nest_service("/test", ServeFile::new("test/index.html"))
        .nest_service("/resources", ServeDir::new("../render_app/resources"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}