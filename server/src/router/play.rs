use crate::router::save;
use crate::util::AppState;
use crate::util::extract::{extract_game, extract_game_instance, extract_index};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::fs;
use std::sync::Arc;
use tracing::{error, warn};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{game_id}/{instance_id}/index", get(handle_play_index))
        .route(
            "/{game_id}/{instance_id}/modList.json",
            get(handle_mod_list),
        )
        .route(
            "/{game_id}/{instance_id}/save-sync/list",
            get(save::handle_save_list),
        )
        .route(
            "/{game_id}/{instance_id}/save-sync/access",
            post(save::handle_save_upload),
        )
        .route(
            "/{game_id}/{instance_id}/save-sync/access/{save_id}",
            get(save::handle_save_get).delete(save::handle_save_del),
        )
        .route("/{game_id}/{instance_id}/{*path}", get(handle_other_file))
}

async fn handle_play_index(
    Path((game_id, instance_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let (game, instance) = match extract_game_instance(&state, &game_id, &instance_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    let index_id = &instance.index;
    let index_info = match extract_index(game, index_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    match fs::read_to_string(&index_info.path) {
        Ok(content) => Html(content).into_response(),
        Err(err) => {
            error!(
                "Unable to read index file of {}: {}, {}",
                game_id, instance_id, err
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to read index file",
            )
                .into_response()
        }
    }
}

async fn handle_mod_list(
    Path((game_id, instance_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let (game, instance) = match extract_game_instance(&state, &game_id, &instance_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    let mut mods: Vec<String> = instance
        .mods
        .iter()
        .filter(|mod_id| game.mods.get(*mod_id).is_some())
        .map(|mod_id| format!("/repo/mod/{game_id}/{mod_id}"))
        .collect();
    mods.push(format!("/repo/mod/{game_id}/save-sync-integration"));

    Json(mods).into_response()
}

async fn handle_other_file(
    Path((game_id, instance_id, path)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let game = match extract_game(&state, &game_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    let instance_fs = match game.instance_fs.get(&instance_id) {
        Some(fs) => fs,
        None => {
            error!(
                "Unable to find instance fs for game {}, instance id {}",
                game_id, instance_id
            );
            return (
                StatusCode::NOT_FOUND,
                format!("No instance file system found for {}", instance_id),
            )
                .into_response();
        }
    };

    match instance_fs.resolve_path(&path) {
        Some(path) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            match fs::read(path) {
                Ok(content) => (
                    StatusCode::OK,
                    [
                        (CONTENT_TYPE, mime.as_ref()),
                        (CACHE_CONTROL, "public, max-age=31536000"),
                        (
                            ETAG,
                            format!("\"{}\"", xxhash_rust::xxh3::xxh3_64(&content)).as_str(),
                        ),
                    ],
                    content,
                )
                    .into_response(),
                Err(err) => {
                    error!("Failed to read file: {}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read file: {}", err),
                    )
                        .into_response()
                }
            }
        }
        None => {
            warn!("Unable to resolve path: {}", path);
            (
                StatusCode::NOT_FOUND,
                format!("Unable to resolve path: {}", path),
            )
                .into_response()
        }
    }
}
