use crate::util::AppState;
use axum::Router;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG};
use axum::response::IntoResponse;
use axum::routing::get;
use lazy_static::lazy_static;
use std::sync::Arc;

mod index;
mod play;
mod repo;
mod save;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index::index_page))
        .route("/favicon.ico", get(get_icon))
        .nest("/play", play::routes())
        .nest("/repo", repo::routes())
}

const ICON: &[u8] = include_bytes!("../../res/favicon.ico");

lazy_static! {
    static ref ICON_ETAG: String = format!("\"{}\"", xxhash_rust::xxh3::xxh3_64(ICON));
}

async fn get_icon() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/zip"),
            (CACHE_CONTROL, "public, max-age=31536000"),
            (ETAG, &ICON_ETAG),
        ],
        ICON,
    )
        .into_response()
}
