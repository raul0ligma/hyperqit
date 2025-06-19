use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::services::{CurrentUserPositionResponse, OpenPositionRequest, StrategyManagerService};

#[derive(Deserialize, Serialize)]
pub struct ResponseJson<T> {
    data: T,
}

pub async fn create_new_position(
    State(manager_svc): State<Arc<StrategyManagerService>>,
    Json(req): Json<OpenPositionRequest>,
) -> Result<Json<ResponseJson<CurrentUserPositionResponse>>, String> {
    match manager_svc.open_position(req).await {
        Ok(response) => Ok(Json(ResponseJson { data: response })),
        Err(e) => Err(format!("error opening position: {:?}", e)),
    }
}

pub async fn get_current_position(
    State(manager_svc): State<Arc<StrategyManagerService>>,
) -> Result<Json<ResponseJson<CurrentUserPositionResponse>>, String> {
    match manager_svc.current_user_position().await {
        Ok(response) => Ok(Json(ResponseJson { data: response })),
        Err(e) => Err(format!("error getting current position: {:?}", e)),
    }
}

pub async fn close_position(
    State(manager_svc): State<Arc<StrategyManagerService>>,
) -> Result<Json<ResponseJson<String>>, String> {
    match manager_svc.close_position().await {
        Ok(response) => Ok(Json(ResponseJson { data: response })),
        Err(e) => Err(format!("error closing position: {:?}", e)),
    }
}
