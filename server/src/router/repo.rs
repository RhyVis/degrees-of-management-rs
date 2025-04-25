use crate::foundation::structure::FileInfo;
use crate::util::AppState;
use crate::util::extract::{extract_game, extract_game_mod};
use crate::util::file::{etag_check, etag_hash};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use lazy_static::lazy_static;
use std::sync::Arc;
use tracing::error;

pub const SAVE_SYNC_INTEGRATION_MOD_ID: &str = "save-sync-integration";
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
    headers: HeaderMap,
) -> impl IntoResponse {
    let game_info = match extract_game(&state, &game_id) {
        Ok(game_info) => game_info,
        Err(response) => return response.into_response(),
    };

    if !game_info.game_def.use_mods {
        return (StatusCode::BAD_REQUEST, "Game does not support mods").into_response();
    }

    if mod_id == SAVE_SYNC_INTEGRATION_MOD_ID {
        if let Some(if_not_match) = headers.get(IF_NONE_MATCH) {
            if let Ok(cli_tag) = if_not_match.to_str() {
                if cli_tag == SAVE_SYNC_INTEGRATION_ETAG.as_str() {
                    return (
                        StatusCode::NOT_MODIFIED,
                        [
                            (CACHE_CONTROL, "public, max-age=31536000"),
                            (ETAG, SAVE_SYNC_INTEGRATION_ETAG.as_str()),
                        ],
                    )
                        .into_response();
                }
            }
        }

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

    if let Some(response) = etag_check(&file, &headers) {
        return response;
    }

    let etag_val = etag_hash(&file);
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/zip"),
            (CACHE_CONTROL, "public, max-age=31536000"),
            (ETAG, etag_val.as_str()),
        ],
        file,
    )
        .into_response()
}
