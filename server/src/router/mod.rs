use crate::constants::CACHE_HEADER;
use crate::util::AppState;
use crate::util::file::etag_check;
use axum::Router;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG};
use axum::http::{HeaderMap, StatusCode};
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

async fn get_icon(headers: HeaderMap) -> impl IntoResponse {
    if let Some(response) = etag_check(&ICON.to_vec(), &headers) {
        return response;
    }

    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "image/x-icon"),
            (CACHE_CONTROL, CACHE_HEADER),
            (ETAG, &ICON_ETAG),
        ],
        ICON,
    )
        .into_response()
}
