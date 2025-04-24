use crate::foundation::registry::Registry;
use crate::foundation::structure::{GameInfo, IndexInfo, InstanceInfo, ModInfo};
use crate::util::AppState;
use axum::http::StatusCode;
use std::sync::Arc;

pub fn extract_game<'a>(
    state: &'a Arc<AppState>,
    game_id: &'a str,
) -> Result<&'a GameInfo, (StatusCode, String)> {
    state.registry.get(game_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no game found with id {}", game_id),
        )
    })
}

pub fn extract_game_instance<'a>(
    state: &'a Arc<AppState>,
    game_id: &'a str,
    instance_id: &'a str,
) -> Result<(&'a GameInfo, &'a InstanceInfo), (StatusCode, String)> {
    let game = state.registry.get(game_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no game found with id {}", game_id),
        )
    })?;

    let instance = game.instances.get(instance_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no instance found with id {}", instance_id),
        )
    })?;

    Ok((game, instance))
}

pub fn extract_game_mod<'a>(
    state: &'a Arc<AppState>,
    game_id: &'a str,
    mod_id: &str,
) -> Result<&'a ModInfo, (StatusCode, String)> {
    let game = state.registry.get(game_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no game found with id {}", game_id),
        )
    })?;

    let mod_info = game.mods.get(mod_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no mod found with id {}", mod_id),
        )
    })?;

    Ok(mod_info)
}

pub fn extract_index<'a>(
    game: &'a GameInfo,
    index_id: &str,
) -> Result<&'a IndexInfo, (StatusCode, String)> {
    game.indexes
        .get(index_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("索引 '{}' 不存在", index_id)))
}
