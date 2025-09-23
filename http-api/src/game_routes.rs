use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::gamefinder::GameFinderSettings;
use std::sync::{Arc};
use tokio::sync::{Mutex, RwLock};
use crate::state::AppState;

#[axum::debug_handler]
pub async fn update_settings(
    app_state: State<AppState>,
    request: Json<GameFinderSettings>,
) -> (StatusCode, Json<GameFinderSettings>) {
    let mut finder_settings = app_state.0.finder_settings.write().await;
    *finder_settings = request.clone().0;

    (StatusCode::OK, request)
}