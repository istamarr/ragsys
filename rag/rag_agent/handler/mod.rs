pub mod native_handler;
pub mod user_handler;
pub mod llm_handler;
pub mod key_handler;
pub mod rfid_handler;
mod r#impl;
pub mod query_handler;
pub mod yolov8;
pub mod qrcode;
pub mod command_helper;
pub mod t2v;
pub mod v2t;
pub mod flow_code;
mod documents;
pub mod embedded;

// only use for upload
// command in postman/simple http for
// fixco, flowcode
pub mod embedd_helper_handler;
