use crate::util::AppState;
use crate::util::extract::extract_game;
use anyhow::Result;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Local;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

const SAVE_FILE_EXTENSION: &str = "save";

pub async fn handle_save_list(
    Path((game_id, instance_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let game = match extract_game(&state, &game_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };
    Json(iter_save_list(&game.get_save_path_append(&instance_id))).into_response()
}

pub async fn handle_save_get(
    Path((game_id, instance_id, save_id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let game = match extract_game(&state, &game_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    let content = match get_save_content(&game.get_save_path_append(&instance_id), &save_id) {
        Some(content) => content,
        None => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    info!("Request save file: {}-{}-{}", game_id, instance_id, save_id);

    content.into_response()
}

pub async fn handle_save_del(
    Path((game_id, instance_id, save_id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let game = match extract_game(&state, &game_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    del_save_content(&game.get_save_path_append(&instance_id), &save_id);

    info!("Delete save file: {}-{}", game_id, instance_id);
    format!("Successfully deleted {save_id}").into_response()
}

pub async fn handle_save_upload(
    Path((game_id, instance_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    Json(save_code): Json<SaveCode>,
) -> impl IntoResponse {
    let game = match extract_game(&state, &game_id) {
        Ok(result) => result,
        Err(response) => return response.into_response(),
    };

    match write_save_content(
        &game.get_save_path_append(&instance_id),
        &game_id,
        &instance_id,
        save_code,
    ) {
        Ok(_) => {
            info!("Save file successfully: {}-{}", game_id, instance_id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => {
            error!(
                "Failed to write save file ({game_id}-{instance_id}): {}",
                err
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SaveCode {
    pub code: String,
    alias: String,
}

impl SaveCode {
    fn get_alias_no_empty(&self) -> String {
        if self.alias.is_empty() {
            String::from("anonymous")
        } else {
            self.alias.clone()
        }
    }
}

fn iter_save_list(save_dir: &PathBuf) -> Vec<String> {
    if !save_dir.exists() {
        fs::create_dir(save_dir).unwrap_or_else(|err| {
            error!("Failed to create save directory: {}", err);
        });
    }
    fs::read_dir(save_dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map_or(false, |ext_str| ext_str == SAVE_FILE_EXTENSION)
                })
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_else(|err| {
            error!("Failed to read save directory: {}", err);
            vec![]
        })
}

fn get_save_content(save_dir: &PathBuf, save_id: &str) -> Option<String> {
    if !save_dir.exists() {
        fs::create_dir(save_dir).unwrap_or_else(|err| {
            error!("Failed to create save directory: {}", err);
        });
    }
    match fs::read_to_string(save_dir.join(assemble_save_name(&save_id))) {
        Ok(file) => Some(file),
        Err(err) => {
            error!("Failed to read save file ({}) : {}", save_id, err);
            None
        }
    }
}

fn del_save_content(save_dir: &PathBuf, save_id: &str) {
    if !save_dir.exists() {
        fs::create_dir(save_dir).unwrap_or_else(|err| {
            error!("Failed to create save directory: {}", err);
        });
    }
    let file_path = save_dir.join(assemble_save_name(&save_id));
    if file_path.exists() {
        if let Err(err) = fs::remove_file(&file_path) {
            error!("Failed to delete save file ({}) : {}", save_id, err);
        }
    } else {
        warn!("Save file not found for deletion: {}", file_path.display());
    }
}

fn write_save_content(
    save_dir: &PathBuf,
    game_id: &str,
    instance_id: &str,
    code: SaveCode,
) -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d+%H-%M-%S").to_string();
    let file_name = format!(
        "{}@{}.{}",
        code.get_alias_no_empty(),
        timestamp,
        SAVE_FILE_EXTENSION
    );

    if !save_dir.exists() {
        fs::create_dir(save_dir).unwrap_or_else(|err| {
            error!("Failed to create save directory: {}", err);
        });
    }
    let file_path = save_dir.join(file_name);
    if file_path.exists() {
        warn!(
            "Save file already exists ({game_id}-{instance_id}): {}",
            file_path.display()
        );
    }
    let mut file = fs::File::create(&file_path)?;
    file.write_all(code.code.as_bytes())?;

    Ok(())
}

fn assemble_save_name(save_id: &str) -> String {
    format!("{}.{}", save_id, SAVE_FILE_EXTENSION)
}
