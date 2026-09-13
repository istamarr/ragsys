use std::collections::HashMap;
use std::sync::Arc;
use dotenv::dotenv;
use log::info;
use warp::Rejection;
use crate::secure::error::Error;
use crate::secure::error;
use crate::domain::models::user::User;
use crate::handler::dfsr_img::main;
use crate::shared::secureUtils::{ErrHandler};

pub mod secure;
pub mod shared;
pub mod domain;
pub mod handler;
pub mod srv;
pub mod qrcode;
// type Result<T> = std::result::Result<T, error::Error>;
// type WebResult<T> = std::result::Result<T, Rejection>;

type Result<T> = std::result::Result<T, error::Error>;//error::Error
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

/// ---- function of test ---- ///
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}


///  ---- main of image  ---- ///
// #[tokio::main]
pub async fn run() -> ResultErrFramework<()>   {
    info!("Running Image...");
    main().expect("Use Command Rag Image: rustimg --help");
    dotenv().ok();
    info!("Diffuser Create Image - {:?}", shared::helperUtils::current_time());
    Ok(())
}

