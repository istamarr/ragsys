use lazy_static::lazy_static;
use log::{debug, info, log_enabled, Level, LevelFilter};
use warp::Rejection;
use wasm_bindgen::prelude::*;
use std::sync::{Mutex, MutexGuard};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use redis::Connection;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Result, Surreal,
};
use redis::Commands;
use redis::Client as redisClient;
use rag::shared::sharedUtils::Databases;

lazy_static! {
    pub static ref REDIS_CLIENT: Mutex<redisClient> = Mutex::new(redis::Client::open("redis://127.0.0.1".to_string() + ":" + &"6379".to_string()).unwrap());
    pub static ref CONNECT_REDIS_OXIDE: Mutex<redis::Connection> = {
        let client: MutexGuard<redisClient> = REDIS_CLIENT.lock().unwrap();
       Mutex::new((*client).get_connection().unwrap())
    };
}
fn main() {
    dotenv().ok();
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("[TEST][REDIS][105]");
    }

    println!("connect redis oxide database 105");

    //connect to db redis
    // let mut connectRedis = RDB.clone();
    // let mut connectRedis = connect_db_redis();
    // let _: () = redis::cmd("PING").query(&mut connectRedis).expect("Failed to execute PING command");
    info!("Try to connect redis oxide database");
    let mut con = CONNECT_REDIS_OXIDE.lock().unwrap();
    info!("Set First Ping");
    let _: () = con.set("ping", "pong").unwrap(); //only for first time
    let val: String = con.get("ping").unwrap();
    debug!("Value redis test : {}", val);
    //drop connection
    drop(con);
    debug!("Redis connection closed");

}
