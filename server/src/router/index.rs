use crate::util::AppState;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use std::sync::Arc;
use askama::Template;
use axum::http::StatusCode;
use tracing::error;
use crate::foundation::registry::Registry;
use crate::foundation::structure::InstanceInfo;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    instance_map: Vec<(String, Vec<&'a InstanceInfo>)>
}

pub async fn index_page(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let instance_map: Vec<(String, Vec<&InstanceInfo>)> = state.registry
        .all()
        .iter()
        .map(|(id, game_info)| 
            (id.clone(), game_info.instances.iter().map(|(_, instance_info)| instance_info).collect())
        ).collect();
    let template = IndexTemplate { instance_map };
    
    match template.render() { 
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Failed to render index page: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
