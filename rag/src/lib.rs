use warp::Rejection;
use wasm_bindgen::prelude::*;
use crate::secure::error;
pub mod domain;
pub mod shared;
pub mod secure;
pub mod utils;
pub mod srv;
pub mod infrastructure;
pub mod libmodules;
pub mod rag_agent;

pub mod rag_chaining;
pub mod scripts_modules;

pub mod data;
pub mod rag_feedback;

type Result<T> = std::result::Result<T, error::Error>;//error::Error
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

///---------- WASM module -------///
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, lib presets! Devel Test Wasmer!".to_string()
}

//wasm-pack build --target web
//cargo build --lib --release --target wasm32-unknown-unknown

///----------- MODULE RAG RUN ------------- /////
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
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
use crate::srv::api::router::create_router;
use crate::domain::models::user::User;
use crate::infrastructure::data::db_context::db_context::{connect_db, CONNECT_REDIS_OXIDE};
use crate::rag_agent::handler;
use crate::rag_agent::handler::user_handler::{init_users, init_users_db};
use crate::secure::error::Error as RagError;
use crate::shared::secureUtils::{ErrHandler, ErrMsg};

// #[tokio::main]
pub async fn run() -> ResultErrFramework<()>  {
    println!("Running RAG pipeline...");
    /**
    * note:
    * info: log system
    * debug: log process
    * warn: need to warn or prevent
    * error: error log
    */

    //edit # Add this line at the end of the file ~/.bashrc
    // export RUST_LOG=debug
    // source ~/.bashrc
    // env_logger::init();
    // log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("[RAG] pgd, lib! starting up!");
    }
    // handler::queue::main();
    // handler::jobcron::main();

    info!("[RAG] pgd, lib running");

    if log_enabled!(Level::Debug) {
        info!("[RAG] welcome D.Rag Server");
        info!("[RAG] this server is running to Resources ");
        info!("[RAG] pgd, lib! starting up!");
    }

    //connect to db redis
    // let mut connectRedis = RDB.clone();
    // let mut connectRedis = connect_db_redis();
    // let _: () = redis::cmd("PING").query(&mut connectRedis).expect("Failed to execute PING command");
    info!("Try to connect redis oxide database");
    let mut con = CONNECT_REDIS_OXIDE.lock().unwrap();
    // info!("Set First Ping");
    // let _: () = con.set("ping", "pong").unwrap(); //only for first time
    let val: String = con.get("ping").unwrap();
    debug!("Value redis test : {}", val);
    //drop connection
    drop(con);
    debug!("Redis connection closed");

    // Connect to Qdrant server
    info!("Try to connect Qdrant database");
    // let mut QDRANT_URL_PORT_6334=  env::var("QDRANT_URL_PORT_6334")
    //     .context("QDRANT_URL_PORT_6334 belum diatur");
    // let client = Qdrant::from_url(&QDRANT_URL_PORT_6334.unwrap()).build();
    // let client = Qdrant::from_url("http://10.253.247.105:6334").build();
    let client = Qdrant::from_url(&*"http://localhost:6334").build().unwrap();
    //clone:
    // let client_clone = client.clone().unwrap();
    // Health check
    let health_check_qdrant = client.health_check().await;
    info!("Health check Qdrant: {:?}", health_check_qdrant);
    // List collections
    let collections_list = client.list_collections().await;
    info!("Collections list Qdrant: {:?}", collections_list);


    //connect to postgre database server
    // Define connection string
    info!("Try To Connect PostgreDB ");
    // let config = "host=10.253.247.105 port=5432 user=postgres password=admin dbname=rag_db";
    let config = "host=localhost port=2345 user=postgres password=admin dbname=d_rag_lm";
    let Ok((clientPgDB, connectionPgDB)) =
        tokio_postgres::connect(config, NoTls).await
    else { todo!("check connection") };
    tokio::spawn(async move {
        if let Err(e) = connectionPgDB.await {
            error!("Connection error: {}", e);
        }
    });
    let rows = clientPgDB.query("SELECT version()", &[]).await;
    for row in rows {
        let version: Option<&Row> = row.get(0);
        info!("PostgreSQL version: {:?}", version);
    }
    info!("Connection to PostgreDB Done ");


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

    // let mut users_map = HashMap::new();
    // users_map.insert("user1".to_string(), User {
    //     uid: String::from("1"),
    //     password: String::from("somepwd123"),
    //     email: String::from("someone@example.com"),
    //     role: String::from("dev"),
    // });
    //
    // let users = Arc::new(users_map);

    let app = create_router(users.clone());
    info!("Rag Server started successfully ✨");
    warp::serve(app).run(([0, 0, 0, 0], 9090)).await;

    Ok(())
}


#[tokio::main]
pub async fn debug_err() -> ResultInclude<()>  {
    info!(" Debug Unknown Error ");
    Ok(())
}
