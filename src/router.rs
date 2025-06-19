use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    handlers::{close_position, create_new_position, get_current_position},
    services::StrategyManagerService,
};

pub fn create_router(manager_svc: Arc<StrategyManagerService>) -> Router {
    Router::new()
        .route("/v1/strategy/open", post(create_new_position))
        .route("/v1/strategy/close", post(close_position))
        .route("/v1/strategy/position", get(get_current_position))
        .with_state(manager_svc)
}
