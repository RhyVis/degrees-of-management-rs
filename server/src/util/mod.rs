use crate::foundation::registry::GameRegistry;

pub(crate) mod extract;
pub(crate) mod file;
pub(crate) mod resolve;

pub struct AppState {
    pub registry: GameRegistry,
}
