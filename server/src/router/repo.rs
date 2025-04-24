use crate::foundation::structure::FileInfo;
use crate::util::AppState;
use crate::util::extract::extract_game_mod;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG};
use axum::response::IntoResponse;
use axum::routing::get;
use lazy_static::lazy_static;
use std::sync::Arc;
use tracing::error;

const SAVE_SYNC_INTEGRATION_INTERNAL: &[u8] =
    include_bytes!("../../res/save-sync-integration.mod.zip");

lazy_static! {
    static ref SAVE_SYNC_INTEGRATION_ETAG: String = format!(
        "\"{}\"",
        xxhash_rust::xxh3::xxh3_64(SAVE_SYNC_INTEGRATION_INTERNAL)
    );
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/mod/{game_id}/{mod_id}", get(handle_mod_file))
}

async fn handle_mod_file(
    Path((game_id, mod_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if mod_id == "save-sync-integration" {
        return (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/zip"),
                (CACHE_CONTROL, "public, max-age=31536000"),
                (ETAG, &SAVE_SYNC_INTEGRATION_ETAG),
            ],
            SAVE_SYNC_INTEGRATION_INTERNAL,
        )
            .into_response();
    }

    let mod_info = match extract_game_mod(&state, &game_id, &mod_id) {
        Ok(x) => x,
        Err(y) => return y.into_response(),
    };

    let file = match mod_info.read_bytes() {
        Ok(x) => x,
        Err(y) => {
            error!(
                "Failed to read mod file for game {}: {}, error: {}",
                &game_id, &mod_id, y
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to access mod {}:{}", &game_id, &mod_id),
            )
                .into_response();
        }
    };

    (StatusCode::OK, [(CONTENT_TYPE, "application/zip")], file).into_response()
}
