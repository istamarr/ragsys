/**
* Quick Think Embedding
* Flexible Dimensional Embedding LLMs
*/
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::error::Error;
use qdrant_client::{Payload, Qdrant};
use qdrant_client::qdrant::{
    PointStruct, SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, Value,
    Filter, QueryPointsBuilder, VectorParams
};
use qdrant_client::qdrant::value;
use serde_json::json;
use tokio::time::{timeout, Duration};
use ndarray::{Array1, Array2, Axis};
use std::collections::HashMap;
use std::env;
use qdrant_client;
use qdrant_client::Qdrant;
// use qdrant_client::client::QdrantClient;
use crate::domain::models::llm::{CmdBody, GGUFFile, RequestBody};
use crate::shared::sharedUtils::LLM;
use qdrant_client::{
    // prelude::*,
};
use anyhow::{Context, Result};
use log::info;
use reqwest::Client as HttpClient;
use crate::shared::helperUtils::{utils_embedding_vec_dim, utils_lm_attribute, validate_command_helper};
use crate::srv::srv_client::agent_protocol_client::agent_protocol_master_flow_cmd;
use crate::domain::dbs_rag_LM::Payload as dbs_rag_LM_payload;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
    message: Message,
    done: bool,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResponse {
    result: Vec<ScoredPoint>,
    time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScoredPoint {
    id: Option<PointId>,
    payload: HashMap<String, Value>,
    score: f32,
    version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PointId {
    point_id_options: Option<Num>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Num {
    num: u64,
}

fn extract_balanced_json(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let mut count = 0;
    let mut start_index = None;
    for (i, c) in trimmed.char_indices() {
        if c == '{' {
            if start_index.is_none() {
                start_index = Some(i);
            }
            count += 1;
        } else if c == '}' {
            count -= 1;
            if count == 0 {
                if let Some(start) = start_index {
                    return Some(&trimmed[start..=i]);
                }
            }
        }
    }
    None
}

async fn retrieve_documents(
    qdrant_client: &Qdrant,
    query_embedding: Vec<f32>,
    name_collection: String,
) -> Result<Vec<String>, Box<dyn Error>> {
    let search_result = timeout(
        Duration::from_secs(120),
        qdrant_client.search_points(
            SearchPointsBuilder::new(name_collection, query_embedding, 3)
                .params(SearchParamsBuilder::default().hnsw_ef(32).exact(false)),
        ),).await;
    match search_result {
        Ok(Ok(search_response)) => {
            println!("Qdrant search response: {:?}", search_response);
            let extracted_documents: Vec<String> = search_response
                .result
                .iter()
                .map(|point| {
                    let payload = &point.payload;
                    if let Some(val) = payload.get("description") {//--> description
                        if let Some(value::Kind::StringValue(text)) = &val.kind {
                            return text.clone();
                        }
                    }"Default document".to_string()
                }).collect();
            Ok(extracted_documents)
        }
        Ok(Err(qdrant_error)) => {
            println!("Qdrant search error: {:?}", qdrant_error);
            Err(Box::new(qdrant_error))
        }
        Err(e) => {
            println!("Qdrant search timed out: {:?}", e);
            Err(Box::new(e))
        }
    }
}

/**
* Get Embedding Ollama (default) - Di If Aja dengan Client ID, Mapping Client Id ke Server Mana, Buat Di Table
**/
async fn get_embedding(
    client: &Client,
    prompt: String,
    model_embed: String,
) -> Result<EmbeddingResponse, Box<dyn Error>> {
    let request_payload = EmbeddingRequest {
        model: model_embed,
        prompt,
    };

    let response = client
        .post("http://192.168.227.193:11434/api/embeddings")
        .json(&request_payload)
        .send()
        .await?;
    let raw_response = response.text().await?;
    println!("Raw embedding response: {:?}", raw_response);

    let json_str = extract_balanced_json(&raw_response)
        .ok_or_else(|| "No JSON found in embedding response".to_string())?;
    let embedding_response: EmbeddingResponse = serde_json::from_str(json_str)?;
    Ok(embedding_response)
}

/**
* Get Embedding Burn-LM - Di If Aja dengan Client ID, Mapping Client Id ke Server Mana, Buat Di Table
**/
async fn get_embedding_burnlm(
    client: &Client,
    prompt: String,
    model_embed: String,
) -> Result<EmbeddingResponse, Box<dyn Error>> {
    let request_payload = EmbeddingRequest {
        model: model_embed,
        prompt,
    };

    let response = client
        .post("http://192.168.227.193:11434/api/embeddings")
        .json(&request_payload)
        .send()
        .await?;
    let raw_response = response.text().await?;
    println!("Raw embedding response: {:?}", raw_response);

    let json_str = extract_balanced_json(&raw_response)
        .ok_or_else(|| "No JSON found in embedding response".to_string())?;
    let embedding_response: EmbeddingResponse = serde_json::from_str(json_str)?;
    Ok(embedding_response)
}


/**
* Get Embedding Local-VM - Di If Aja dengan Client ID, Mapping Client Id ke Server Mana, Buat Di Table
**/
// async fn get_embedding_server(
//     client: &Client,
//     prompt: String,
//     model_embed: String,
// ) -> Result<EmbeddingResponse, Box<dyn Error>> {
//     // let request_payload = EmbeddingRequest {
//     //     model: model_embed,
//     //     prompt.clone(),
//     // };
//     let mut model_pgd = model_embed.clone();
//     if model_pgd=="" {
//         model_pgd = "../asist.safetensors".parse().unwrap();
//     }
//     let model_options = ModelOptions::default();
//     let llama = LLama::new(
//         model_pgd.into(),
//         &model_options,
//     ).unwrap();
//
//     let predict_options = PredictOptions {
//         token_callback: Some(Box::new(|token| {
//             println!("token1: {}", token);
//             true
//         })),
//         ..Default::default()
//     };
//
//     let validate = validate_command_helper(prompt.clone());
//     if validate == "" {
//         let json_str = "Format Prompt Request Not Valid "
//             .ok_or_else(|| "No JSON found in embedding response".to_string())?;
//         let validate_response: EmbeddingResponse = serde_json::from_str(json_str)?;
//         Ok(validate_response)
//     }.expect("Format Prompt Request Not Valid");
//
//     let response= llama
//         .predict(
//             model_embed.clone().into(),
//             predict_options,
//         ).unwrap();
//
//     let raw_response = response.await?;
//     println!("Raw embedding response: {:?}", raw_response);
//
//     let json_str = extract_balanced_json(&raw_response)
//         .ok_or_else(|| "No JSON found in embedding response".to_string())?;
//     let embedding_response: EmbeddingResponse = serde_json::from_str(json_str)?;
//     Ok(embedding_response)
// }


async fn get_chat_response(client: &Client, payload: serde_json::Value) -> Result<String, Box<dyn Error>> {
    let response = client
        .post("http://172.24.176.1:11434/api/chat")
        .json(&payload)
        .send()
        .await?;
    let raw_response = response.text().await?;
    Ok(raw_response)
}

fn reduce_embedding(embedding: Vec<f32>) -> Vec<f32> {
    if embedding.len() == 512 {
        return embedding;
    }

    let mut reduced_embedding = embedding.clone();
    if reduced_embedding.len() > 512 {
        reduced_embedding.truncate(512);
    } else if reduced_embedding.len() < 512 {
        println!("Warning: Embedding dimension is less than 512, cannot truncate.");
        return vec![];
    }
    reduced_embedding
}


pub fn increase_embedding(embedding: Vec<f32>, target_size: usize) -> Vec<f32> {
    if embedding.len() == target_size {
        return embedding;
    }

    let mut increased_embedding = embedding.clone();
    if increased_embedding.len() < target_size {
        let additional_elements = target_size - increased_embedding.len();
        increased_embedding.extend(vec![0.0; additional_elements]);
    } else {
        println!("Warning: Embedding dimension is greater than target size, cannot increase.");
        return vec![];
    }
    increased_embedding
}


pub async fn retrieve_context_rag(
    qdrant_client: &Qdrant,
    query_embedding: Vec<f32>,
) -> anyhow::Result<String> {
    let log = " Rag Pipeline ~ Retrieve Context ".to_string();

    info!("{} # start", log.clone());
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
    info!("{} # url {}", log.clone(),qdrant_url);
    let mut COLLECTION_NAME = env::var("ASIST_DETAIL").context("ASIST_DETAIL belum diatur")?;
    info!("{} # collection {}", log.clone(),COLLECTION_NAME);

    // let query_embedding = generate_embedding_local(query).await?;
    // let query_embedding = utils_embedding_vec_dim(query, 3);
    info!("{} # query embedding {:?}", log.clone(), query_embedding);


    info!("{} # Search qdrant for relevant chunks - start", log.clone());
    // Search Qdrant for relevant chunks
    // http://localhost:6333/collections/:collection_name/points/:id
    // let search_result = qdrant_client
    //     .search_points(&SearchPoints {
    //         collection_name: COLLECTION_NAME.to_string(),
    //         vector: query_embedding,
    //         limit: 3, // Get top 3 relevant chunks
    //         with_payload: Some(true.into()),
    //         ..Default::default()
    //     }).await?;
    //
    let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
    // let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;

    // let search_request = self
    //     .client
    //     .search_points(
    //         SearchPointsBuilder::new("embeddings".to_string(), query_vector, 5)
    //             .with_payload(true)
    //             .params(SearchParamsBuilder::default().exact(true)),
    //     );

    // let mut search_request = SearchPointsBuilder::new(
    //     COLLECTION_NAME.to_string(),
    //     query_embedding,
    //     3,     // Search limit, number of results to return
    // ).with_payload(true);
    // let search_result = client.search_points(search_request).await?;

    let mut search_result = client
        .query(
            QueryPointsBuilder::new(COLLECTION_NAME.to_string())
                .query(query_embedding)
                .limit(3)
        ).await?;

    info!("{} # Search qdrant for relevant chunks - done", log.clone());
    info!("{} # query embedding ~ Search Result: {:?}", log.clone(), search_result);

    info!("{} # Extract text from results - start", log.clone());
    let context: Vec<String> = search_result
        .result
        .into_iter()
        .filter_map(|point| {
            point
                .payload
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    info!("{} # Extract text from results - done", log.clone());

    //sample payload request get master flow to agent protocol
     let agent_protocol_payload = dbs_rag_LM_payload {
        title: "String".to_string(),
        tags: vec!["".to_string(),"".to_string()],
        data: "String".to_string(),
        source: "String".to_string(),
        date: "String".to_string(),//created_date
         similiarity_score: "".to_string(),
         precision_score: "".to_string(),
         troubleshot: "".to_string(),
         instruction_hint: "".to_string(),
         response_struct_result: "".to_string(),
     };
    agent_protocol_master_flow_cmd(agent_protocol_payload).await.expect("Cmd to Agent Protocol ~ Flow Master ~ Requesting Master Flow");
    Ok(context.join("\n\n"))
}


/// Build Response Model RAG with burn-lm ASIST check and Ollama fallback
///
/// Flow:
/// 1. Check burn-lm ASIST availability (HTTP at BURN_LM_URL)
/// 2. If burn-lm available -> use ASIST model
/// 3. If burn-lm unavailable -> fallback to Ollama ollamar/tamar:1b
pub async fn build_response_model_rag(query: &str, context: &str, cmd_body: CmdBody) -> Result<String> {
    let log = " RAG ~ Build Response Model ".to_string();
    let llm_prompt_cmd = cmd_body.to_owned();
    let client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Get URLs from environment
    let burn_lm_url = env::var("BURN_LM_URL")
        .unwrap_or_else(|_| "http://localhost:9393".to_string());
    let ollama_url = env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    // Ollama fallback model
    let ollama_fallback_model = env::var("OLLAMA_FALLBACK_MODEL")
        .unwrap_or_else(|_| "ollamar/tamar:1b".to_string());

    info!("{} # Checking burn-lm ASIST at {}", log, burn_lm_url);

    // STEP 1: Try burn-lm ASIST first
    let burn_lm_result = try_burn_lm_asist(&client, &burn_lm_url, query, context).await;

    match burn_lm_result {
        Ok(response) => {
            info!("{} # burn-lm ASIST response received", log);
            Ok(response)
        }
        Err(burn_lm_error) => {
            info!("{} # burn-lm unavailable: {}, falling back to Ollama", log, burn_lm_error);

            // STEP 2: Fallback to Ollama ollamar/tamar:1b
            let ollama_result = try_ollama_fallback(
                &client,
                &ollama_url,
                &ollama_fallback_model,
                query,
                context,
                &llm_prompt_cmd
            ).await;

            match ollama_result {
                Ok(response) => {
                    info!("{} # Ollama fallback response received", log);
                    Ok(response)
                }
                Err(ollama_error) => {
                    // Both failed - return combined error
                    Err(anyhow::anyhow!(
                        "All model sources failed. burn-lm: {}, Ollama: {}",
                        burn_lm_error, ollama_error
                    ))
                }
            }
        }
    }
}

/// Try burn-lm ASIST model via HTTP API
async fn try_burn_lm_asist(
    client: &HttpClient,
    burn_lm_url: &str,
    query: &str,
    context: &str,
) -> Result<String> {
    let log = " RAG ~ burn-lm ASIST ".to_string();

    // First check health
    let health_check = client
        .get(&format!("{}/health", burn_lm_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match health_check {
        Ok(resp) if resp.status().is_success() => {
            info!("{} # Health check passed", log);
        }
        Ok(resp) => {
            return Err(anyhow::anyhow!("burn-lm health check failed: status {}", resp.status()));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("burn-lm not reachable: {}", e));
        }
    }

    // Combine query with context for better response
    let combined_query = if context.is_empty() {
        query.to_string()
    } else {
        format!("Context: {}\n\nQuestion: {}", context, query)
    };

    // Call burn-lm /ask endpoint
    let ask_payload = json!({
        "question": combined_query,
        "format": "detailed",
        "include_metadata": true
    });

    info!("{} # Sending request to {}/ask", log, burn_lm_url);

    let response = client
        .post(&format!("{}/ask", burn_lm_url))
        .json(&ask_payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("burn-lm API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;

    // Extract answer from burn-lm response format
    response_json["answer"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from burn-lm ASIST"))
}

/// Fallback to Ollama with ollamar/tamar:1b model
async fn try_ollama_fallback(
    client: &HttpClient,
    ollama_url: &str,
    model_name: &str,
    query: &str,
    context: &str,
    cmd_body: &CmdBody,
) -> Result<String> {
    let log = " RAG ~ Ollama Fallback ".to_string();

    info!("{} # Using model: {}", log, model_name);

    // Prepare the prompt with context
    let system_prompt = if cmd_body.prompt.is_empty() {
        "Kamu adalah asisten AI yang membantu menjawab pertanyaan dengan akurat berdasarkan konteks yang diberikan. Jawab dalam Bahasa Indonesia.".to_string()
    } else {
        cmd_body.prompt.clone()
    };

    let messages = json!([
        {
            "role": "system",
            "content": system_prompt
        },
        {
            "role": "user",
            "content": format!(
                "Konteks:\n{}\n\nPertanyaan: {}",
                context, query
            )
        }
    ]);

    let response = client
        .post(&format!("{}/api/chat", ollama_url))
        .json(&json!({
            "model": model_name,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "num_ctx": 4096
            }
        }))
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Ollama chat API error ({}): {}",
            status,
            error_text
        ));
    }

    let response_json: serde_json::Value = response.json().await?;

    response_json["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Ollama"))
}

//local embed
pub async fn build_response_model_embed(query: &str, context: &str, cmd_body: CmdBody) -> Result<String> {
    // let mut owned_string: String = nameModel.to_owned();
    // let borrowed_string: &str = &*versionModel;
    let mut llm_prompt_cmd = cmd_body.to_owned();
    let mut LOCAL_LLM_MODEL: String = utils_lm_attribute(llm_prompt_cmd.clone());
    let client = HttpClient::new();
    let ollama_url = env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string()); //todo: gunakan redis

    let model_name = env::var("LLM_MODEL")
        .unwrap_or_else(|_| LOCAL_LLM_MODEL.clone().to_string());

    // Prepare the prompt with context
    let messages = json!([
        {
            "role": "system",
            "content": llm_prompt_cmd.clone().prompt
        },
        {
            "role": "user",
            "content": format!(
                "Context {} {}",
                context, query
            )
        }
    ]);

    let response = client
        .post(&format!("{}/api/chat", ollama_url))
        .json(&json!({
            "model": model_name,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "num_ctx": 4096  // Increase context window size
            }
        }))
        .send()
        .await?;

    let errorstatus = response.status();
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Ollama chat API error ({}): {}",
            errorstatus,
            error_text
        ));
    }

    let response_json: serde_json::Value = response.json().await?;

    response_json["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Ollama"))
}


pub async fn run_quick_embedding(body: RequestBody) -> Result<String, Box<dyn Error>> { /// Result<(), Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(300)).build()?;
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
    let client_qdrant = Qdrant::from_url(&*qdrant_url).build()?;
    // let client_qdrant = Qdrant::from_url("http://192.168.227.193:6334").build()?;
    let mut messages: Vec<Message> = vec![];
    let model_embed = LLM::PEDIA_ASIST_LM.code;//move from deepseekR1 or LLama

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let user_message = Message {
        role: "user".to_string(),
        content: input.trim().to_string(),
    };
    messages.push(user_message.clone());
    println!("User input: {}", input.trim());
    let embedding_response =
        get_embedding(&client, input.trim().to_string(), model_embed.to_string()).await?;
    let reduced_embedding = reduce_embedding(embedding_response.embedding);
    if reduced_embedding.is_empty() {
        return Ok(("Check Embedding Size in Empty").parse().unwrap());
    }
    println!("Reduced Embedding: {:?}", reduced_embedding);

    let documents = retrieve_documents(&client_qdrant, reduced_embedding.clone(), "name_collection".to_string()).await?;
    let context = documents.join("\n");
    let chat_payload = json!({
        "model": model_embed,
        "stream": false,
        "messages": [
            { "role": "system", "content": context },
            { "role": "user", "content": input.trim().to_string() }
        ]
    });

    let raw_chat_response = get_chat_response(&client, chat_payload).await?;
    println!("Raw chat response: {:?}", raw_chat_response);

    let json_str = extract_balanced_json(&raw_chat_response)
        .ok_or_else(|| "No JSON found in chat response".to_string())?;
    println!("Extracted JSON for chat response: {:?}", json_str);
    let decoded_response: Response = serde_json::from_str(json_str)?;

    let response_embedding_response = get_embedding(
        &client,
        decoded_response.message.content.clone(),
        model_embed.to_string(),
    ).await?;
    let reduced_response_embedding = reduce_embedding(response_embedding_response.embedding);

    // Increase Embedding:
    // let original_embedding = vec![0.1, 0.2, 0.3];
    // let target_sizes = vec![512, 1024, 2048, 3072, 4096, 5120];
    //
    // for target_size in target_sizes {
    //     let increased_embedding = increase_embedding(original_embedding.clone(), target_size);
    //     println!("Increased embedding to size {}: {:?}", target_size, increased_embedding.len());
    // }

    client_qdrant
        .upsert_points(
            UpsertPointsBuilder::new(
                "wolt",
                vec![PointStruct::new(
                    messages.len() as u64,
                    reduced_response_embedding,
                    Payload::try_from(json!({ "description": decoded_response.message.content }))?,
                )],//tambahkan name dan date dan email / nohp untuk hist data
            )
                .wait(true),
        )
        .await?;

    println!("AI Response: {}", decoded_response.message.content);
    Ok(decoded_response.message.content.clone())
}

pub async fn generate_embedding_local(text: &str) -> Result<Vec<f32>> {
    let local_embedding_model: String = format!("{}:{}",
                                                LLM::PEDIA_ASIST_LM.code.to_string(),
                                                LLM::PEDIA_ASIST_LM.version.to_string());

    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
    let mut COLLECTION_NAME = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL belum diatur")?;

    let client = HttpClient::new();
    let ollama_url = env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let model = env::var("EMBEDDING_MODEL")
        .unwrap_or_else(|_| local_embedding_model.to_string());

    let response = client
        .post(&format!("{}/api/embeddings", ollama_url))
        .json(&json!({
            "model": model,
            "prompt": text
        }))
        .send()
        .await?;

    let errorstatus = response.status();
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Ollama embedding API error ({}): {}",
            errorstatus,
            error_text
        ));
    }
    let response_json: serde_json::Value = response.json().await?;
    response_json["embedding"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid embedding format"))?
        .iter()
        .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(|| anyhow::anyhow!("Invalid float value")))
        .collect()
}
