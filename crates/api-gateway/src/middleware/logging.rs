use axum::extract::Request;
use axum::middleware::Next;
use std::time::Instant;

pub async fn log_requests(req: Request, next: Next) -> axum::response::Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    tracing::info!("--> {} {}", method, path);

    let response = next.run(req).await;

    let elapsed = start.elapsed();
    tracing::info!("<-- {} {} {}ms", method, path, elapsed.as_millis());

    response
}
