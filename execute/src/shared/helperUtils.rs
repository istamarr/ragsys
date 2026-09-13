use std::io::empty;
use std::ptr::null;
//global or general function for all codes
use chrono::Local;
use dotenv::dotenv;
use log::{debug, error, info};
use tokio::io::split;
use tokio_postgres::NoTls;
use warp::{http::StatusCode, reply::json, Rejection, Reply};
use crate::domain::models::llm::{LLMRequest, RequestBody};
use crate::domain::models::srv::RespSrv;
use crate::shared::sharedUtils::LLM;
use crate::WebResult;
use anyhow::{Context, Result};
use std::process;
use std::process::ExitCode;

pub fn current_time() -> String {
    let now = Local::now();
    format!("{}", now.to_rfc3339())
}

pub fn srv_response(message: String, status: StatusCode) -> WebResult<impl Reply> {
    let response = RespSrv {
        message: message.to_string(),
        status: status.as_u16(),
    };

    Ok(json(&response))
}

pub fn utils_embedding_vec_dim(text: &str, dim_input: usize) -> Vec<f32> {
    let dim = dim_input;
    let mut embedding = vec![0.0; dim];
    for (i, byte) in text.bytes().enumerate() {
        embedding[i % dim] += byte as f32 / 255.0;
    }
    embedding
}

pub fn validate_empty_value(){
    //todo: add all validate empty and blank value
    return ;
}

pub fn utils_lm_attribute(llmPrompt: RequestBody) -> String {
    let log = " Util LM Attribute ".to_string();

    //V.1
    //START GET VERSION
    let modelrequest= llmPrompt.model;
    let mut nameModel = "".to_string();
    let mut versionModel = "".to_string();
    if (modelrequest.is_empty() || modelrequest == "") {
        //todo: make validate_empty_value for validate all req and return if empty or blank
        error!("{}","RAG EMBED: FILL MODEL PROMPT REQ ".to_string());
        process::exit(0x0100);
        // ExitCode::FAILURE
        // Err("".to_string())
    }

    if(!modelrequest.is_empty() && modelrequest!=""){
        nameModel = format!("{}", modelrequest.split(':').nth(0).unwrap());
        versionModel = format!("{}", modelrequest.split(':').nth(1).unwrap());
    }
    info!("embed LLM: nameModel {}",nameModel);
    info!("embed LLM: versionModel {}",versionModel);

    let mut owned_string: String = nameModel.to_owned();
    let borrowed_string: &str = &*versionModel;
    //todo: bedakan dengan category
    if (owned_string==LLM::PEDIA_AIS_LM.code && versionModel==LLM::PEDIA_AIS_LM.version) {
        if borrowed_string != "" {
            owned_string.push_str(":");
            owned_string.push_str(borrowed_string);
        }
    }else if (owned_string==LLM::PEDIA_ASIST_LM.code && versionModel==LLM::PEDIA_ASIST_LM.version) {
        if borrowed_string != "" {
            owned_string.push_str(":");
            owned_string.push_str(borrowed_string);
        }
    }else {
        if borrowed_string!="" {
            owned_string.push_str(":");
            owned_string.push_str(borrowed_string);
        }
        error!("{} # {}",log.clone(),"RAG EMBED: LLM MODEL PROMPT NOT AVAILABLE ".to_string());
        // debug!("{}","RAG EMBED: LLM MODEL PROMPT NOT AVAILABLE ".to_string());
        process::exit(0x0100);
        //todo: check database if it available, return if not found
    }
    info!("{} embed LLM: Found LLM Prompt Model {:?}", log.clone(),owned_string);

    //V.2
    //todo : ubah dengan persistance --> surreal --> code
    //temp: lsg ke postgres(persistance)
    // info!("Try To Connect PostgreDB ");
    // let Ok((clientPgDB, connectionPgDB)) =
    //     tokio_postgres::connect("host=localhost port=2345 user=postgres password=admin dbname=d_rag_lm", NoTls).await
    // else { todo!("not found connection") };
    // tokio::spawn(async move {
    //     if let Err(e) = connectionPgDB.await {
    //         error!("Connection error: {}", e);
    //     }
    // });
    // let rows = clientPgDB.query("SELECT version()", &[]).await?;
    // for row in rows {
    //     let version: &str = row.get(0);
    //     info!("PostgreSQL version: {}", version);
    // }
    // info!("Connection to PostgreDB Done ");
    //
    // let LOCAL_LLM_MODEL: &str = owned_string.as_str();
    // LOCAL_LLM_MODEL.to_string()

    owned_string.to_string()
}

pub fn validate_command_helper(promptCMD: String, typeCMD: String) -> String {
    if promptCMD=="" {
        error!("{}","RAG VALIDATE COMMAND: NOT VALID COMMAND - BLANK ".to_string());
        process::exit(0x0100);
    }

    // if  {  }

    println!("");
    println!("");
    println!("");
    println!("");
    println!("");
    println!("");

    "".to_string()
}

