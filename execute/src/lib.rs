use std::collections::HashMap;
use std::sync::Arc;
use dotenv::dotenv;
use warp::Rejection;
use crate::secure::error::Error;
use crate::secure::error;
use wasm_bindgen::prelude::*;
use crate::domain::models::user::User;
use crate::shared::secureUtils::ErrHandler;

pub mod secure;
pub mod shared;
pub mod domain;
// type Result<T> = std::result::Result<T, error::Error>;
// type WebResult<T> = std::result::Result<T, Rejection>;

type Result<T> = std::result::Result<T, error::Error>;//error::Error
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

// #[tokio::main]
pub async fn run() -> ResultErrFramework<()>   {
    dotenv().ok();
    println!("Running Execute...");

    Ok(())
}
