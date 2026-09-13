use dotenv::dotenv;
use qdrant_client::{
    // prelude::*,
    qdrant::SearchPoints,
};
use std::env;
use std::fs::metadata;
use actix_web::http::StatusCode;
use anyhow::{Context, Result};
use log::{debug, info};
use qdrant_client::qdrant::VectorParams;
use serde_json::json;
use reqwest::Client as HttpClient;
use uuid::Uuid;
use qdrant_client::qdrant::SearchPointsBuilder;
use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder, SearchParamsBuilder};
use qdrant_client::Qdrant;
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    fmt_message, fmt_placeholder, fmt_template,
    language_models::llm::LLM as langchainLLM,
    // llm::ollama::{openai},// add llm local
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::messages::Message,
    template_fstring,
};
use warp::Reply;
use crate::domain::models::llm::{CmdBody, RequestBody};
use crate::rag_chaining::rag_pipeline::rag_pipeline;
use crate::shared::helperUtils::{current_time, srv_response};
use crate::WebResult;
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM};

//text2img, text2text, text2voice
pub async fn queryVec(uid : String, body : CmdBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    info!("QUERY VEC request - start");

    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    // let config = QdrantClient::from(&qdrant_url);
    let qdrant_client = Qdrant::from_url(&*qdrant_url).build().unwrap();

    let response = rag_pipeline(&qdrant_client, body.clone()).await;
    let mut versionModel = "version";

    info!("{} {}", format!("QUER VEC: Quick Think using flex embedding : {} {} Version {} At {} ", uid, body.clone().model, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    info!("QUERY VEC Success Result Quick Think");
    let result = &*response.unwrap();
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}
