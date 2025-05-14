mod constants;
mod foundation;
mod router;
mod util;

use crate::foundation::config::{CONFIG, init_config};
use crate::foundation::registry::init_registry;
use crate::router::get_router;
use crate::util::AppState;
use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    init_config()?;
    let registry = init_registry()?;

    let port = CONFIG.get().expect("Config not initialized.").port;
    let addr = format!("0.0.0.0:{port}");

    let app = Router::new()
        .merge(get_router())
        .with_state(Arc::new(AppState { registry }));
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
