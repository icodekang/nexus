use axum::{routing::{post, get}, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn register() -> &'static str {
    "register"
}

async fn login() -> &'static str {
    "login"
}

async fn logout() -> &'static str {
    "logout"
}
