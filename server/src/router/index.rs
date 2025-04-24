use crate::foundation::registry::Registry;
use crate::foundation::structure::InstanceInfo;
use crate::util::AppState;
use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use std::sync::Arc;
use tracing::error;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    instance_map: Vec<((String, String), Vec<&'a InstanceInfo>)>,
}

pub async fn index_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut instance_map: Vec<((String, String), Vec<&InstanceInfo>)> = state
        .registry
        .all()
        .iter()
        .map(|(id, game_info)| {
            let mut instances: Vec<&InstanceInfo> = game_info
                .instances
                .iter()
                .map(|(_, instance_info)| instance_info)
                .collect();
            instances.sort_by(|a, b| a.id.cmp(&b.id));

            if let Some(game_name) = &game_info.game_def.name {
                ((id.clone(), game_name.clone()), instances)
            } else {
                ((id.clone(), id.clone()), instances)
            }
        })
        .collect();
    instance_map.sort_by(|a, b| a.0.0.cmp(&b.0.0));
    let template = IndexTemplate { instance_map };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Failed to render index page: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
