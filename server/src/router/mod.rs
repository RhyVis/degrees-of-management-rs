use crate::util::AppState;
use axum::Router;
use std::sync::Arc;

mod play;
mod repo;
mod save;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/play", play::routes())
        .nest("/repo", repo::routes())
}
