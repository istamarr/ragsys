use actix_web::http::StatusCode;
use log::{debug, info, warn};
use warp::Reply;
use crate::domain::models::llm::{Detection, GGUFFile, LLMCreateBuildRequest, LLMRequest};
use crate::domain::models::login::LoginRequest;
use crate::shared::helperUtils::{current_time, srv_response};
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM};
use crate::WebResult;
use std::fs;
use std::error::Error;
use crate::rag_agent::handler::r#impl::load_training_data_impl::load_n_training_data;

pub async fn llm_image_handler(uid : String, body : LLMRequest) -> WebResult<impl Reply> {
    Ok(format!("Image Scan : {}", uid))
}

pub async fn generate_completion_handler(uid : String, body : LLMRequest) -> WebResult<impl Reply> {
    Ok(format!("Generate Completion : {}", uid))
}

pub async fn generate_analysis_tag_handler(uid : String, body : LLMRequest) -> WebResult<impl Reply> {
    Ok(format!("Generate Ans Completion : {} {}", uid, LLM::PEDIA_ASIST_LM.code))
}


/**
* Naming Convention:
* Llama-3.2-1B-Instruct-Q8.fileformat
* format naming:
* [name]-[version]-[size]-[specific tunning or indicator]
* -[quantization level, e.g Q8 highest precision, Q4_0 for 4bit, _K sprcial kernel, _F faster ]
* .[fileformat:  .bin, .pt, .onnx, .gguf, .tflite]
*
* terbaik(analitycs), definition, create (infer, dll) , ada yg idle buuat pgd ada yg active buat project terkait
*/
pub async fn generate_llm(uid : String, body : LLMCreateBuildRequest) -> WebResult<impl Reply> {
    // gather data --> build llm --> deploy MLFLow (test open web client ui ollama server) --> build client wasm --> deploy artifactory

    if body.message!="create"{
        warn!("Please using system message ");
        return srv_response(format!("Error: {}", "Please using system message "),StatusCode::from_u16(StatusCode::INTERNAL_SERVER_ERROR.as_u16()).unwrap());
    }

    /**
    * Define Naming
    */
    info!("Phase Define Naming...");
    let version = "1";//nanti disimpan di database dan load sequence version, buat sequence di table dan function to generate
    let fullname = LLM::PEDIA_ASIST_LM.code.to_owned() +"-"+version;
    println!("Fullname of LLM {} ", fullname);

    /**
    * Gather & Prepare Datasets
    */
    info!("Phase Gather & Prepare Datasets...");
    load_n_training_data().await;


    /**
    * Create And Build .bin
    */
    info!("Phase Create and Build BIN...");


    /**
    * Create And Build tokenizer
    */
    info!("Phase Create and Build tokenizer...");


    /**
    * Create And Build .gguf
    */
    info!("Phase Create and Build GGUF...");

    /**
    * Simple Dev & Testing
    */
    info!("Phase Internal Dev And Testing...");

    /**
    * Deploy into MLFLow
    */
    info!("Phase Deploying into MLFlow...");

    /**
    * Next Task (coding wasm client lib & deploy artifactory)
    */
    warn!("Next Task Create Code For WASM Client and Deploy into Artifactory...");

    // let model = Array2::random((2, 2), rand::distributions::Uniform::new(-1., 1.));
    // save_model(&model, "modules/ai_model/bin/model_wights.bin").expect("Failed to save model");
    // println!("Model saved to modules/ai_model/bin/model_weights.bin");

    // Ok(format!("Generate LLM PGD Completion : {} {} Version {} At {} ", uid, LLM::MODEL_PGD.code, version, current_time()))
    srv_response(format!("Generate LLM PGD Completion : {} {} Version {} At {} ", uid, LLM::PEDIA_ASIST_LM.code, version, current_time()),
                         StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}
