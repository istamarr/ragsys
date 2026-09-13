use reqwest::Client;
use scraper::{Html, Selector};
use std::{error::Error, io, time::Duration, env};
use std::ptr::null;
use qdrant_client::Payload;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    prompt: String,
    model: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

async fn generate_embedding(text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let client = Client::new();
    let model_name = "ollama-embedding-model".to_string();

    let request_body = OllamaEmbeddingRequest {
        prompt: text.to_owned(),
        model: model_name,
    };

    let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_endpoint = format!("{}/embed",QDRANT_URL_PORT_6334);
    let response = client
        .post(qdrant_endpoint)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let err_text = response.text().await?;
        return Err(format!("Error from Ollama server: {}", err_text).into());
    }

    let response_data: OllamaEmbeddingResponse = response.json().await?;
    Ok(response_data.embedding)
}

async fn process_page(url: &str) -> Result<Vec<Document>, Box<dyn Error>> {
    let html_content = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html_content);
    let paragraph_selector = Selector::parse("p").unwrap();

    let mut extracted_text = String::new();
    for element in document.select(&paragraph_selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        extracted_text.push_str(&text);
        extracted_text.push('\n');
    }

    let chunks = chunk_text(&extracted_text, 500);
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
        // In actual use, connect and authenticate as needed.
        Ok(Qdrant)
    }
}

async fn index_documents(client_qdrant: &Qdrant, docs: Vec<Document>) -> Result<(), Box<dyn Error>> {
    for doc in docs.iter() {
        let id: u64 = doc.id[4..].parse()?;

        // add request body dan mapping
        let payload = Payload::try_from(json!({
            "description": doc.text,
            "name": "Istamar Rozid",
            "date": "2025-04-23",
            "email": "istamar.rozid@gmail.com",
            "nohp": "08123456789"
        }))?;

        // client_qdrant
        //     .upsert_points(
        //         UpsertPointsBuilder::new(
        //             "pgd_rag_collection",
        //             vec![PointStruct::new(
        //                 id,
        //                 doc.embedding.clone(),
        //                 payload,
        //             )],
        //         )
        //             .wait(true),
        //     )
        //     .await?;
    }
    Ok(())
}


async fn retrieve_documents(client_qdrant: &Qdrant, embedding: Vec<f32>) -> Result<Vec<Document>, Box<dyn Error>> {
    Ok(vec![
        Document {
            id: "doc".to_string(),
            text: "retrieved.".to_string(),
            embedding,
        },
    ])
}

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

enum LLM {
    PEDIA_ASIST_LM,
}

impl LLM {
    fn code(&self) -> &'static str {
        match self {
            LLM::PEDIA_ASIST_LM => "ASIST",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_qdrant = Qdrant::from_url("http://192.168.227.193:6334").build()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    //println!("Enter the URL to scrape and vectorize:");
    //let mut url_input = String::new();
    //io::stdin().read_line(&mut url_input)?;
    //let url_input = url_input.trim();


    let agent_url = "https://localhost:9292/agent-endpoint";
    let query = "hallo";
    let client_id = "UniqueClientID";

    let agent_response = fetch_agent_data(&client, agent_url, query, client_id).await?;
    println!("AGENT Response: {:?}", agent_response);

    let docs = process_agent_data(agent_response).await?;
    println!("Processed {} document chunks from AGENT.", docs.len());

    // let client_qdrant = Qdrant::from_url("http://192.168.227.193:6334").build()?;

    index_documents(&client_qdrant, docs).await?;
    println!("Documents successfully indexed into Qdrant.");


    let url_input = "...";
    let mut query_input = String::new();
    io::stdin().read_line(&mut query_input)?;
    let query_input = query_input.trim().to_string();

    let model_embed = LLM::PEDIA_ASIST_LM.code();
    let embedding_response = get_embedding(&client, query_input.clone(), model_embed.to_string()).await?;
    let reduced_embedding = reduce_embedding(embedding_response.embedding);
    if reduced_embedding.is_empty() {
        return Err("Check Embedding Size: got an empty embedding".into());
    }
    println!("Query Embedding: {:?}", reduced_embedding);
    let retrieved_docs = retrieve_documents(&client_qdrant, reduced_embedding.clone()).await?;
    println!("Retrieved Documents from Qdrant:");
    for doc in &retrieved_docs {
        println!("Document ID: {}, Content: {}", doc.id, doc.text);
    }
    let final_prompt = create_prompt(&query_input, &retrieved_docs);
    println!("{}", final_prompt);
    Ok(())
}

#[derive(Debug, Serialize)]
struct MCPRequest {
    query: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
struct MCPResponse {
    status: String,
    data: Vec<String>, //pakai untuk agent response
}

struct AgentRequest {
    query: String,
    client_id: String,
}

#[derive(Debug)]
struct AgentResponse{
    response: String,
    data: ()
}
async fn fetch_agent_data(client: &Client, url: &str, query: &str, client_id: &str) -> Result<AgentResponse, Box<dyn Error>> {
    let request_body = AgentRequest {
        query: query.to_string(),
        client_id: client_id.to_string(),
    };

    // let agent_url = "https://localhost:9292/agent-endpoint";
    // let query = "hallo";
    // let client_id = "UniqueClientID";

    // let response = client.post(url).json(&request_body).send().await?;
    // if !response.status().is_success() {
    //     let err_text = response.text().await?;
    //     return Err(format!("Error from AGENT server: {}", err_text).into());
    // }
    //
    // let response_data = response.json::<AgentResponse>().await?;
    // Ok(response_data)
    let result = AgentResponse {
        response: "".to_string(),
        data: ()
    };

    Ok(result)
}

async fn process_agent_data(agent_response: AgentResponse) -> Result<Vec<Document>, Box<dyn Error>> {
    // let chunks = chunk_text(&agent_response.data.join("\n"), 500);
    let mut docs = Vec::new();

    // for (idx, chunk) in chunks.into_iter().enumerate() {
    //     let embedding = generate_embedding(&chunk).await?;
    //     docs.push(Document {
    //         id: format!("doc_{}", idx),
    //         text: chunk,
    //         embedding,
    //     });
    // }
    Ok(docs)
}