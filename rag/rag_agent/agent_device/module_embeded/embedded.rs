use log::{debug, info};
use reqwest::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::shared::helperUtils::current_time;
use wasm_bindgen::prelude::*;
use crate::domain::models::llm::{LLMRequest, RequestBody};
use thiserror::Error;
use crate::srv::stream::http::request::get_request;

// #[wasm_bindgen]
pub async fn embed_rfid_interface_raspi()-> Result<(), Error> {//mfrc
    info!("interface: rfid raspi");
    debug!("embedded: wasm rfid {:?}", current_time() );

    Ok(get_request().await?)
}

// #[wasm_bindgen]
pub async fn embed_rfid_interface()-> Result<(), Error> {//mfrc
    info!("interface: rfid stm32L4");
    debug!("embedded: wasm rfid {:?}", current_time() );

    Ok(get_request().await?)
}
