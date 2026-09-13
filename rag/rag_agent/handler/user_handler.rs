use std::collections::HashMap;
use std::sync::Arc;
use log::debug;
use warp::Reply;
use crate::domain::models::user::{User, UserDB};
use crate::infrastructure::data::db_context::db_context::DB;
use crate::shared::helperUtils::current_time;
use crate::shared::secureUtils_LoginUsingQdrant::load_all_users_qdrant;
use crate::WebResult;
use std::env;
use anyhow::{Context, Result};

pub async fn init_users_db() -> HashMap<String, User> {
    let credential_source = env::var("CREDENTIAL_SOURCE").unwrap_or_else(|_| "surreal".to_string());

    match credential_source.as_str() {
        "qdrant" => init_users_db_qdrant().await,
        _ => init_users_db_surreal().await,
    }
}

pub async fn init_users_db_surreal() -> HashMap<String, User> {
    let records:Vec<UserDB> = DB.select(&String::from("user_credentials")).await.unwrap();
    let mut map = HashMap::new();
    for userDb in records {
        map.insert(userDb.email.to_string(), User { uid : userDb.email.to_string(), email : userDb.email.to_string(), password : userDb.password.to_string(), role : userDb.role.to_string()});
    }

    map
}

pub async fn init_users_db_qdrant() -> HashMap<String, User> {
    load_all_users_qdrant().await
}

pub fn init_users(usersdb: Arc<HashMap<String, User>>) -> HashMap<String, User> {
    debug!("Initial User - Start {}",  current_time());
    let mut map = HashMap::new();
    if !usersdb.as_ref().is_empty() {
        debug!("Check User - Found {}", current_time());
        for (key, value) in usersdb.as_ref().iter() {
            let user = User {
                uid: key.clone(),
                email: format!("{}", value.email.clone()),
                password: format!("{}", value.password.clone()),
                role: format!("{}", value.role.clone()),
            };
            map.insert(key.clone(), user);
        }
    }
    debug!("Initial User - Succed {}", current_time());
    map
}

pub async fn user_handler(uid : String) -> WebResult<impl Reply> {
    Ok(format!("User with id : {}", uid))
}

pub async fn admin_handler(uid : String) -> WebResult<impl Reply> {
    Ok(format!("Admin with id : {}", uid))
}
