use std::collections::HashMap;
use std::sync::Arc;
use warp::Rejection;
use crate::secure::error;
use wasm_bindgen::prelude::*;
pub mod api;
pub mod infrastructure;
pub mod secure;
pub mod shared;
pub mod domain;
pub mod task_module;

type Result<T> = std::result::Result<T, error::Error>;//error::Error
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;
// use std::io::Error;


///----------- MODULE TASK RUN ------------- /////
#[macro_use]
extern crate lazy_static;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;
use log::{debug, error, info, log, log_enabled, warn, Level, LevelFilter};
use redis::{Client, Commands, Connection, RedisResult};
use std::sync::{Mutex, MutexGuard};
use anyhow::Context;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use redis::Client as redisClient;
use qdrant_client::Qdrant;
use tokio_postgres::{NoTls, Error, Row};
use std::process::Command;
use dotenv::dotenv;
use crate::api::router::create_router;
use crate::domain::models::user::User;
use crate::infrastructure::data::db_context::db_context::{connect_db, CONNECT_REDIS_OXIDE};
use crate::task_module::handler;
use crate::task_module::handler::user_handler::{init_users, init_users_db};
use crate::secure::error::Error as RagError;
use crate::shared::secureUtils::{ErrHandler, ErrMsg};

// #[tokio::main]
pub async fn run() -> ResultErrFramework<()>   {
    dotenv().ok();
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("[TASK] Running Task...! starting up!");
    }

    info!("Start To Connect Surreal Database ");
    connect_db().await.unwrap();
    // load users credential (using jwt)
    let usersdb = Arc::new(init_users_db().await);
    if !usersdb.as_ref().is_empty() {
        debug!("found and load user credential");
    }else{
        warn!("no users credential found");
    }
    let users = Arc::new(init_users(usersdb));
    info!("Connection to Surreal Database Done ");

    // let app = create_router(users.clone());
    // info!("[OK] D.Rag Server started successfully");
    // warp::serve(app).run(([0, 0, 0, 0], 9191)).await;

    Ok(())
}
