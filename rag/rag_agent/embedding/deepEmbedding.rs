/**
* Deep Think Embedding
* Precise Dimensional Embedding LLMs
*/


use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::error::Error;
use qdrant_client::{Payload, Qdrant};
use qdrant_client::qdrant::{PointStruct, QueryPointsBuilder, SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, Value};
use qdrant_client::qdrant::value; // For value::Kind
use serde_json::json;
use tokio::time::{timeout, Duration};
use ndarray::{Array1, Array2, Axis};
// use ndarray_linalg::SolveH;
use std::collections::HashMap;
// use diesel::query_dsl::InternalJoinDsl;  // Disabled - requires libpq.lib
use crate::domain::models::llm::{Detection, GGUFFile, RequestBody};
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM, SizeDim};

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
    collection: String
) -> Result<Vec<String>, Box<dyn Error>> {
    let search_result = timeout(
        Duration::from_secs(120),
        qdrant_client.search_points(
            SearchPointsBuilder::new(collection, query_embedding, 3)
                .params(SearchParamsBuilder::default().hnsw_ef(32).exact(false)),
        ),
        // qdrant_client
        //     .query(
        //         QueryPointsBuilder::new(collection)
        //             .query(query_embedding)
        //             .limit(3)
        //     ).await?,
    )
        .await;

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
                    }
                    "Default document".to_string()
                })
                .collect();
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

async fn get_chat_response(client: &Client, payload: serde_json::Value) -> Result<String, Box<dyn Error>> {
    let response = client
        .post("http://172.24.176.1:11434/api/chat")
        .json(&payload)
        .send()
        .await?;
    let raw_response = response.text().await?;
    Ok(raw_response)
}
fn check_embedding_size(embedding: Vec<f32>) -> Vec<f32> {
    let sizeDim: usize = SizeDim::SIZE_DIM_2048.size.parse().unwrap();
    if embedding.len() == sizeDim {
        return embedding;
    }else {
        println!("Warning: Embedding dimension is not exactly match to 2048, cannot processed.");
        return vec![];
    }

}

fn reduce_embedding(embedding: Vec<f32>) -> Vec<f32> {
    let sizeDim: usize = SizeDim::SIZE_DIM_2048.size.parse().unwrap();
    if embedding.len() == sizeDim.clone() {
        return embedding;
    }

    let mut reduced_embedding = embedding.clone();
    if reduced_embedding.len() > sizeDim.clone() {
        reduced_embedding.truncate(sizeDim.clone());
    } else if reduced_embedding.len() < sizeDim.clone() {
        println!("Warning: Embedding dimension is less than 2048, cannot truncate.");
        return vec![];
    }
    reduced_embedding
}

fn increase_embedding(embedding: Vec<f32>, target_size: usize) -> Vec<f32> {
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

pub async fn run_deep_embedding(body: RequestBody) -> Result<String, Box<dyn Error>> { /// Result<(), Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(300)).build()?;

    let client_qdrant = Qdrant::from_url(&*"http://192.168.227.193:6334").build()?;
    let mut messages: Vec<Message> = vec![];
    let model_embed = LLM::PEDIA_ASIST_LM.code;
        //LLM::DEEP_SEEK_R1.code;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let user_message = Message {
        role: "user".to_string(),
        content: input.trim().to_string(),
    };
    messages.push(user_message.clone());
    println!("User input: {}", input.trim());

    // 2. Create embedding for the user prompt.
    let embedding_response =
        get_embedding(&client, input.trim().to_string(), model_embed.to_string()).await?;
    let check_embedding_size = check_embedding_size(embedding_response.embedding);
    if check_embedding_size.is_empty() {
        return Ok(("Check Embedding Size in Empty").parse().unwrap());
    }
    let documents = "";
    let context = "";
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

    println!("AI: {}", decoded_response.message.content);
    Ok(decoded_response.message.content.clone())
}
