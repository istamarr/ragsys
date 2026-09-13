use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email : String,
    pub password : String
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token : String,
}

#[derive(Deserialize)]
pub struct ReqKeyRequest {
    pub app: String
}

#[derive(Serialize)]
pub struct ReqKeyResponse {
    pub reqkey: String
}

#[derive(Deserialize)]
pub struct ReqAiKeyRequest {
    pub reqkey: String
}

