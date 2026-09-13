use std::sync::{Mutex, MutexGuard};
use lazy_static::lazy_static;
use log::{debug, info};
use once_cell::sync::Lazy;
use redis::Connection;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Result, Surreal,
};
use crate::shared;
use crate::shared::sharedUtils::Databases;
use redis::Commands;
use redis::Client as redisClient;

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

pub async fn connect_db() -> Result<()> {
    let _ = DB.connect::<Ws>(shared::sharedUtils::Databases::SURREAL_DB.url.to_owned() +":"+shared::sharedUtils::Databases::SURREAL_DB.port).await?;
    let _ = DB
        .signin(Root {
            username: shared::sharedUtils::Databases::SURREAL_DB.username,
            password: shared::sharedUtils::Databases::SURREAL_DB.password,
        })
        .await;
    let _ = DB.use_ns(shared::sharedUtils::Databases::SURREAL_DB.ns).use_db(shared::sharedUtils::Databases::SURREAL_DB.db).await?;
    debug!("surreal connected");
    Ok(())
}

// pub static RDB: Lazy<Client> = Lazy::new(|| {
//     Client::open(Databases::REDIS_DB.url.to_string() + ":" + &Databases::REDIS_DB.port.to_string()).expect("Failed to create Redis client");
//     debug!("redis server connected");
// });

// pub async fn connect_db_redis() -> Connection {
//     let redisClient = redis::Client::open(Databases::REDIS_DB.url.to_string() + ":" + &Databases::REDIS_DB.port.to_string());
//     let mut redisconnect = redisClient.get_connection()?;
//     debug!("redis server connected");
//     return redisconnect;
// }

// let mut redisconnect = redis::Connection();
// pub fn connect_db_redis() -> Connection {
    // let nodes = vec![Databases::REDIS_DB.url.to_string() + ":" + &Databases::REDIS_DB.port.to_string()];
    // let client = redisClient::new(nodes).unwrap();
    // let mut connection = client.get_connection().unwrap();
    // return connection

    // let _: () = connection.set("test", "test_data").unwrap();
    // let rv: String = connection.get("test").unwrap();
    // return rv;

    // info!("connect redis server");
    // let client = redis::Client::open( Databases::REDIS_DB.url.to_string() + ":" + &Databases::REDIS_DB.port.to_string());
    // let mut connectRedisOxide = client.get_connection()?;
    // connectRedisOxide
// }

lazy_static! {
    pub static ref REDIS_CLIENT: Mutex<redisClient> = Mutex::new(redis::Client::open(Databases::REDIS_DB.url.to_string() + ":" + &Databases::REDIS_DB.port.to_string()).unwrap());
    pub static ref CONNECT_REDIS_OXIDE: Mutex<redis::Connection> = {
        let client: MutexGuard<redisClient> = REDIS_CLIENT.lock().unwrap();
       Mutex::new((*client).get_connection().unwrap())
    };
}
