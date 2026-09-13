use axum::{Json};
use chrono::Local;
use log::{debug, info};
use warp::{reply, Reply};
use crate::shared::helperUtils::current_time;
use crate::WebResult;

pub mod router;

pub async fn health_checker_handler() -> WebResult<impl Reply> {
    debug!("Health Check Service - Start {}",  current_time());
    const MESSAGE: &str = "Working fine, thanks!";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    debug!("Health Check Service - Succeed {}", current_time());
    Ok(reply::json(&json_response))
}
