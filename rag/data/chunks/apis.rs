use reqwest::{Body, Client};
use scraper::{Html, Selector};
use std::{error::Error, io, time::Duration, env};
use qdrant_client::{Payload};
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::shared::sharedUtils::LLM;
// use crate::sharedshared::sharedUtils::LLM;

#[derive(Debug)]
struct Document {
    id: String,
    text: String,
    embedding: Vec<f32>,
}

fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in text.lines() {
        if current_chunk.len() + line.len() > max_chars {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(line);
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    chunks
}

#[derive(Debug, Deserialize)]
struct ExternalResource {
    data: String,
}

async fn fetch_external_resource(client: &Client, url: &str) -> Result<ExternalResource, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let resource = response.json::<ExternalResource>().await?;
    Ok(resource)
}

async fn process_api_data(api_url: &str, client: &Client) -> Result<Vec<Document>, Box<dyn Error>> {
    // Fetch data from the API
    let resource = fetch_external_resource(client, api_url).await?;

    let chunks = chunk_text(&resource.data, 500);
    let mut docs = Vec::new();

    for (idx, chunk) in chunks.into_iter().enumerate() {
        let emb = generate_embedding(&chunk).await?;
        docs.push(Document {
            id: format!("doc_{}", idx),
            text: chunk,
            embedding: emb,
        });
    }
    Ok(docs)
}


#[derive(Serialize)]
struct ModelEmbeddingRequest {
    prompt: String,
    model: String,
}

#[derive(Deserialize)]
struct ModelEmbeddingResponse {
    embedding: Vec<f32>,
    // Depending Ollama endpoint (mgkin ditambahkan field)
}

async fn generate_embedding(text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let post_qdrant_url = format!("{}/embed",qdrant_url);
    let client = Client::new();
    let model_name = format!("{}:{}", crate::shared::sharedUtils::LLM::PEDIA_ASIST_LM.code, crate::shared::sharedUtils::LLM::PEDIA_ASIST_LM.version);

    let request_body = ModelEmbeddingRequest {
        prompt: text.to_owned(),
        model: model_name,
    };

    let response = client
        .post(post_qdrant_url)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let err_text = response.text().await?;
        return Err(format!("Error from Model server: {}", err_text).into());
    }

    let response_data: ModelEmbeddingResponse = response.json().await?;
    Ok(response_data.embedding)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    ///direct url
    // let api_url = "https://localhost:8090/data";
    //
    // match process_api_data(api_url, &client).await {
    //     Ok(docs) => {
    //         println!("Fetched and processed {} document chunks from API.", docs.len());
    //
    //         // Initialize Qdrant client
    //         let client_qdrant = Qdrant::from_url("http://192.168.227.193:6334").build()?;
    //
    //         // Save documents into Qdrant
    //         index_documents(&client_qdrant, docs).await?;
    //         println!("Documents successfully indexed into Qdrant.\n");
    //
    //         println!("Enter your search query:");
    //         let mut query_input = String::new();
    //         io::stdin().read_line(&mut query_input)?;
    //         let query_input = query_input.trim().to_string();
    //
    //         // Specify the embedding model.
    //         let model_embed = LLM::DEEP_SEEK_R1.code();
    //         let embedding_response = get_embedding(&client, query_input.clone(), model_embed.to_string()).await?;
    //         let reduced_embedding = reduce_embedding(embedding_response.embedding);
    //         if reduced_embedding.is_empty() {
    //             return Err("Check Embedding Size: got an empty embedding".into());
    //         }
    //         println!("Query Embedding: {:?}", reduced_embedding);
    //
    //         let retrieved_docs = retrieve_documents(&client_qdrant, reduced_embedding.clone()).await?;
    //         println!("Retrieved Documents from Qdrant:");
    //         for doc in &retrieved_docs {
    //             println!("Document ID: {}, Content: {}", doc.id, doc.text);
    //         }
    //
    //         let final_prompt = create_prompt(&query_input, &retrieved_docs);
    //         println!("Final Prompt for Generation:\n{}", final_prompt);
    //     },
    //     Err(e) => {
    //         println!("Error processing API data: {:?}", e);
    //     }
    // }


    let agent_protocol_url_env = env::var("AGENT_PROTOCOL_URL")
        .unwrap_or_else(|_| "http://0.0.0.0:9191".to_string());
    let agent_protocol_url = &*format!("{}/retrieve", agent_protocol_url_env);
    let query = "sample query";
    let client_id = "unique-client-id";

    match fetch_mcp_data(&client, agent_protocol_url, query, client_id).await {
        Ok(response) => {
            println!("Fetched MCProtocol Client Response: {:?}", response);
            let chunks = chunk_text(&response.data.join(" "), 500);
            let mut docs = Vec::new();

            for (idx, chunk) in chunks.into_iter().enumerate() {
                let emb = generate_embedding(&chunk).await?;
                docs.push(Document {
                    id: format!("doc_{}", idx),
                    text: chunk,
                    embedding: emb,
                });
            }

            let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
                .unwrap_or_else(|_| "http://localhost:6334".to_string());
            let client_qdrant = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;

            index_documents(&client_qdrant, docs).await?;
            println!("Documents successfully indexed into Qdrant.");


            println!("Enter your search query:");
            let mut query_input = String::new();
            io::stdin().read_line(&mut query_input)?;
            let query_input = query_input.trim().to_string();

            // Specify the embedding model.
            let model_embed = LLM::PEDIA_ASIST_LM.code;
            let embedding_response = get_embedding(&client, query_input.clone(), model_embed.to_string()).await?;
            let reduced_embedding = reduce_embedding(embedding_response.embedding);
            if reduced_embedding.is_empty() {
                return Err("Check Embedding Size: got an empty embedding".into());
            }
            println!("Query Embedding: {:?}", reduced_embedding);

            let db_dataset = env::var("ASIST_DATASET")
                .unwrap_or_else(|_| "asist_collection_dataset".to_string());
            let retrieved_docs = retrieve_documents(&client_qdrant, reduced_embedding.clone(), Body::from(db_dataset.to_string())).await?;
            println!("Retrieved Documents from Qdrant:");
            for doc in &retrieved_docs {
                println!("Document ID: {}, Content: {}", doc.id, doc.text);
            }

            let final_prompt = create_prompt(&query_input, &retrieved_docs);
            println!("Final Prompt for Generation:\n{}", final_prompt);

        },
        Err(e) => {
            println!("Error fetching Agent Protocol data: {:?}", e);
        }
    }

    Ok(())
}


async fn index_documents(client_qdrant: &Qdrant, docs: Vec<Document>) -> Result<(), Box<dyn Error>> {
    for doc in docs.iter() {
        let id: u64 = doc.id[4..].parse()?;

        let payload = Payload::try_from(json!({
            "description": doc.text,
            "name": "",
            "date": "2025-04-23",
            "email": "istamar.rozid@gmail.com",
            "nohp": "08123456789"
        }))?;
    }
    Ok(())
}

async fn retrieve_documents(client_qdrant: &Qdrant, embedding: Vec<f32>, req:Body) -> Result<Vec<Document>, Box<dyn Error>> {
    Ok(vec![
        Document {
            id: "".to_string(),
            text: "".to_string(),
            embedding,
        },
    ])
}

async fn get_embedding(client: &Client, input: String, model: String) -> Result<EmbeddingResponse, Box<dyn Error>> {
    Ok(EmbeddingResponse { embedding: vec![0.0, 1.0, 2.0] })
}

struct EmbeddingResponse {
    embedding: Vec<f32>,
}

fn reduce_embedding(embedding: Vec<f32>) -> Vec<f32> {
    embedding
}

struct Qdrant;

impl Qdrant {
    fn from_url(url: &str) -> QdrantBuilder {
        QdrantBuilder { url: url.to_string() }
    }
}

struct QdrantBuilder {
    url: String,
}

impl QdrantBuilder {
    fn build(self) -> Result<Qdrant, Box<dyn Error>> {
        Ok(Qdrant)
    }
}

// ASIST, prompt buat dengan command format
fn create_prompt(query: &str, contexts: &[Document]) -> String {
    let mut prompt = String::new();
    prompt.push_str("Context:\n");
    for doc in contexts {
        prompt.push_str(&doc.text);
        prompt.push_str("\n---\n");
    }
    prompt.push_str("Question:\n");
    prompt.push_str(query);
    prompt
}


// add agent protocol:
#[derive(Debug, Deserialize)]
struct AgentProtocolResponse {
    status: String,
    data: Vec<String>, //reponse mcp
}

#[derive(Serialize)]
struct AgentProtocolRequest {
    query: String,
    client_id: String,
}

async fn fetch_mcp_data(client: &Client, url: &str, query: &str, client_id: &str) -> Result<AgentProtocolResponse, Box<dyn Error>> {
    let request_body = AgentProtocolRequest {
        query: query.to_string(),
        client_id: client_id.to_string(),
    };

    let response = client.post(url).json(&request_body).send().await?;
    if !response.status().is_success() {
        let error_message = response.text().await?;
        return Err(format!("Error from Agent Protocol Server: {}", error_message).into());
    }

    let agent_protocol_data = response.json::<AgentProtocolResponse>().await?;
    Ok(agent_protocol_data)
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let client = Client::builder()
//         .timeout(std::time::Duration::from_secs(60))
//         .build()?;
//
//     let agent_url = "https://server_agent_protocol/endpoint";
//     let query = "sample query";
//     let client_id = "unique-client-id";
//
//     match fetch_mcp_data(&client, agent_url, query, client_id).await {
//         Ok(response) => {
//             println!("Fetched Agent Protocol Response: {:?}", response);
//             let chunks = chunk_text(&response.data.join(" "), 500);
//             let mut docs = Vec::new();
//
//             for (idx, chunk) in chunks.into_iter().enumerate() {
//                 let emb = generate_embedding(&chunk).await?;
//                 docs.push(Document {
//                     id: format!("doc_{}", idx),
//                     text: chunk,
//                     embedding: emb,
//                 });
//             }
//
//             let client_qdrant = Qdrant::from_url("http://192.168.227.193:6334").build()?;
//
//             index_documents(&client_qdrant, docs).await?;
//             println!("Documents successfully indexed into Qdrant.");
//         },
//         Err(e) => {
//             println!("Error fetching Agent Protocol data: {:?}", e);
//         }
//     }
//
//     Ok(())
// }



// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let client = Client::builder()
//         .timeout(std::time::Duration::from_secs(60))
//         .build()?;
//
//     // Replace with your external resource endpoint.
//     let url = "https://api.example.com/data";
//     match fetch_external_resource(&client, url).await {
//         Ok(resource) => {
//             println!("Fetched external resource: {:?}", resource);
//             // Use resource.data as additional context/document retrieval result.
//         },
//         Err(e) => {
//             println!("Error fetching external resource: {:?}", e);
//         }
//     }
//
//     Ok(())
// }
