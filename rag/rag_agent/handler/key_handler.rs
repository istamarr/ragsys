use log::{debug, info};
use redis::{Commands, RedisError};
use warp::http::StatusCode;
use warp::Reply;
use crate::domain::models::llm::LLMRequest;
use crate::domain::models::login::{LoginRequest, ReqAiKeyRequest, ReqKeyRequest};
use crate::shared::helperUtils::{current_time, srv_response};
use crate::shared::secureUtils::{generate_api_key, hash_api_key, validate_api_key};
use crate::WebResult;
use crate::infrastructure::data::db_context::db_context::CONNECT_REDIS_OXIDE;
use warp::reject::{self, Reject};

pub async fn reqKey(uid : String, body : ReqKeyRequest) -> WebResult<impl Reply> {
    // Ok(format!("Image Scan : {}", uid)); // tambahkan untuk prefix (get from request app id ReqKeyRequest)
    let api_key = generate_api_key();
    debug!("Generated API Key: {} {}", api_key, current_time());
    let hashed_key = hash_api_key(&api_key);
    debug!("Hashed API Key: {} {}", hashed_key, current_time());

    let mut connectRedis = CONNECT_REDIS_OXIDE.lock().unwrap();
    debug!("success connectRedis");
    let _: () = connectRedis.set(api_key.clone(), &hashed_key).unwrap();
    srv_response(format!("{}", api_key),StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

#[derive(Debug)]
struct RedisRejection(RedisError);
impl Reject for RedisRejection {}

pub async fn validateKey(uid: String, body : ReqAiKeyRequest) -> WebResult<impl Reply>{
    info!("Start To Validate App Ai Key {}", body.reqkey.clone());
    let mut connectRedis = CONNECT_REDIS_OXIDE.lock().unwrap();
    info!("Success Connect Redis");
    let valueHashKey: redis::RedisResult<String> = connectRedis.get(body.reqkey.clone().to_string());
    let redisResult: String = match valueHashKey {
        Ok(value) => value,
        Err(err) => {eprintln!("Error: {}", err);
            return srv_response(format!("Error: {}", err),StatusCode::from_u16(StatusCode::INTERNAL_SERVER_ERROR.as_u16()).unwrap())
        }
    };
    debug!("Found Req Ai Key {}", redisResult);
    let is_valid = validate_api_key(&body.reqkey, redisResult.as_str());
    debug!("Is the provided key valid? {}", is_valid);
    srv_response("valid".to_string(),StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}


