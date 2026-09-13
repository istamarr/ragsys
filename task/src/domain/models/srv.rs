use serde_derive::Serialize;

#[derive(Serialize)]
pub struct RespSrv {
    pub message: String,
    pub status: u16,
}