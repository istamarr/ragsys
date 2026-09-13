use warp::Rejection;
use wasm_bindgen::prelude::*;
use rag::secure::error;

// pub mod domain;
// pub mod shared;
// pub mod secure;
// pub mod utils;
// pub mod rag_agent;
// pub mod srv;

// use agent_difusser::shared::secureUtils::ErrHandler;
// use execute::shared::secureUtils::ErrHandler;
// use orchestrate_workflow::shared::secureUtils::ErrHandler;
use rag::shared::secureUtils::ErrHandler;

type Result<T> = std::result::Result<T, error::Error>;//error::Error
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

use dotenv::dotenv;
use std::path::PathBuf;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;
use warp::{Filter, Reply};
use futures_util::StreamExt;
use std::env;
use futures_util::{
    future::{ready, BoxFuture, FutureExt},
    TryFutureExt};
use log::{error, info, log_enabled, warn, Level, LevelFilter};
use tokio_postgres::Error;
// type ResultErrFramework<T> = std::result::Result<T, Error>;
// use crate::srv::api::router::create_router;
// use crate::domain::models::user::User;
// use chrono::{DateTime, Utc};
// use serde_derive::Deserialize;
use rag::srv::api::router::create_router;
use surrealdb::sql::Thing;
use rag::domain::models::user::User;

#[tokio::main]
async fn main() -> ResultErrFramework<()> {
    info!("Start Call Module");
    dotenv().ok();

    env_logger::init();
    log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("[RAG] pgd, Rag System Starting!");
    }
    info!("[RAG] pgd, Rag System running");

    if log_enabled!(Level::Debug) {
        info!("[RAG] welcome to Rag System");
        info!("[CMD] command ");
        info!("[CMD] cargo run --bin (show all agent option)");
        info!("[CMD] cargo run --bin rag -- args1");
        info!("[CMD] cargo run --bin rag -- rag");
        info!("[CMD] cargo run --bin rag -- rag_socket");
        info!("[CMD] cargo run --bin rag_socket (standalone)");
    }

    let mut users_map = HashMap::new();
    users_map.insert("user1".to_string(), User {
        uid: String::from("1"),
        password: String::from("somepwd123"),
        email: String::from("someone@example.com"),
        role: String::from("dev"),
    });

    let users = Arc::new(users_map);

    let app = create_router(users.clone());
    info!("Rag Server started successfully ✨");
    warp::serve(app).run(([0, 0, 0, 0], 9090)).await;

    Ok(())
}
