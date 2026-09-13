use anyhow::{Context, Result};
use dotenv::dotenv;
use md5;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use ethereum_abi::Abi;
use tokio;
use web3::contract::Contract;
use web3::ethabi::Address;
use web3::transports::Http;
// use ethereum_abi::Abi;
// use web3::types::Address;
//
// ---------------------
// Section 1: NFT Data Retrieval via Ethereum & ethers-rs
// ---------------------
//

/// Structure to hold NFT data
#[derive(Debug, Clone)]
struct NFTData {
    token_id: u64,
    token_uri: String,
    metadata: String,
    // Optionally add event data here in the future.
}

/// Fetch NFT metadata from a token URI (assumes an HTTP URL)
async fn fetch_nft_metadata(token_uri: &str) -> Result<String> {
    if token_uri.starts_with("http") {
        let resp = Client::new()
            .get(token_uri)
            .send()
            .await
            .context("Failed to send HTTP request for NFT metadata.")?;
        let text = resp
            .text()
            .await
            .context("Failed to read NFT metadata text.")?;
        Ok(text)
    } else {
        Ok("Non-HTTP token URI; cannot fetch metadata.".into())
    }
}

/// Retrieve NFT data from a deployed contract using ethers.
/// Expects these environment variables:
/// - INFURA_URL
/// - NFT_CONTRACT_ADDRESS
/// - NFT_ABI_PATH
async fn get_nft_data(token_id: u64) -> Result<NFTData> {
    // Read configuration from the environment
    let infura_url = env::var("INFURA_URL").context("INFURA_URL not set")?;
    let nft_address: Address = env::var("NFT_CONTRACT_ADDRESS")
        .context("NFT_CONTRACT_ADDRESS not set")?
        .parse()
        .context("Invalid NFT_CONTRACT_ADDRESS")?;
    let abi_path = env::var("NFT_ABI_PATH").context("NFT_ABI_PATH not set")?;
    let abi_json = fs::read_to_string(&abi_path)
        .with_context(|| format!("Failed to read ABI file from {}", abi_path))?;
    let abi: Abi = serde_json::from_str(&abi_json)
                    .context("Failed to parse NFT ABI JSON")?;

    // Connect to the Ethereum node.
    // let provider = Provider::<Http>::try_from(infura_url)
    //     .context("Failed to connect to Ethereum node")?;
    // let client = Arc::new(provider);

    // Create a contract instance.
    // let contract = Contract::new(nft_address, abi, client.clone());

    // Call the tokenURI function on the NFT contract.
    let token_uri: String = "".to_string();
        // contract
        // .method::<_, String>("tokenURI", token_id)
        // .context("Failed to create tokenURI method call")?
        // .call()
        // .await
        // .context("Failed to call tokenURI method")?;

    // Fetch metadata using the token URI.
    let metadata = fetch_nft_metadata(&token_uri).await?;

    Ok(NFTData {
        token_id,
        token_uri,
        metadata,
    })
}

//
// ---------------------
// Section 2: Local Embeddings & LLM Integration (Using a Free LLM)
// ---------------------
//

/// simple sample
fn embed_text_local(text: &str) -> Vec<f32> {
    // This dummy function creates an 8-dimensional vector
    let mut embedding = vec![0.0; 8];
    for (i, byte) in text.bytes().enumerate() {
        embedding[i % 8] += byte as f32 / 255.0;
    }
    embedding
}

///simple sample local query
fn query_llm_local(context: &str, question: &str) -> Result<String> {
    // Build the prompt by combining context and the user query.
    let prompt = format!(
        "You are a free LLM (Meta Llama/DeepSeek style model).\n\
         Context:\n{}\n\nQuery:\n{}\n\nAnswer:",
        context, question
    );

    // Note: Adjust the command and its arguments to match your local LLM inference binary.
    let output = Command::new("llama")
        .arg("--prompt")
        .arg(&prompt)
        .arg("--n_predict")
        .arg("200")
        .output()
        .context("Failed to execute local LLM inference binary")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Local LLM inference failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let answer = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(answer)
}

//
// ---------------------
// Section 3: Qdrant Integration for Document Retrieval
// ---------------------
//

#[derive(Debug, Serialize)]
struct QdrantSearchRequest {
    vector: Vec<f32>,
    limit: u32,
    with_payload: bool,
}

#[derive(Debug, Deserialize)]
struct QdrantPoint {
    id: String,
    // This example assumes each payload has a "text" field.
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantPoint>,
}

/// Query your Qdrant instance for similar documents using an embedding vector.
/// Expects these environment variables:
/// - QDRANT_URL
/// - QDRANT_COLLECTION
async fn search_qdrant_real(query_embedding: &[f32]) -> Result<Vec<String>> {
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL not set")?;
    let qdrant_collection =
        env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL not set")?;
    let url = format!(
        "{}/collections/{}/points/search",
        qdrant_url, qdrant_collection
    );

    let client = Client::new();
    let req_body = QdrantSearchRequest {
        vector: query_embedding.to_vec(),
        limit: 3,
        with_payload: true,
    };

    let resp = client
        .post(&url)
        .json(&req_body)
        .send()
        .await
        .context("Failed to call Qdrant search API")?;
    let resp_json: QdrantSearchResponse = resp
        .json()
        .await
        .context("Failed to parse Qdrant response")?;

    // Extract text from payload if present.
    let mut docs = Vec::new();
    for point in resp_json.result {
        if let Some(text) = point.payload.get("text").and_then(|v| v.as_str()) {
            docs.push(text.to_string());
        } else {
            docs.push(format!("Document {} (payload unstructured)", point.id));
        }
    }
    Ok(docs)
}

//
// ---------------------
// Section 4: Blockchain Ledger for Query–Answer Logging
// ---------------------
//

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u128,
    query: String,
    answer: String,
    previous_hash: String,
    hash: String,
}

impl Block {
    /// Create a new block; each block’s hash is an MD5 of its data.
    pub fn new(index: u64, query: &str, answer: &str, previous_hash: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let hash_input = format!("{}{}{}{}", index, timestamp, query, answer);
        let hash = format!("{:x}", md5::compute(hash_input));
        Self {
            index,
            timestamp,
            query: query.to_string(),
            answer: answer.to_string(),
            previous_hash: previous_hash.to_string(),
            hash,
        }
    }
}

/// Save the blockchain ledger to a file for persistence.
fn save_blockchain(ledger: &Vec<Block>) -> Result<()> {
    let json = serde_json::to_string_pretty(ledger)
        .context("Failed to serialize blockchain ledger to JSON")?;
    fs::write("blockchain_ledger.json", json)
        .context("Failed to write blockchain ledger to file")?;
    Ok(())
}

//
// ---------------------
// Section 5: Main Processing Pipeline
// ---------------------
//

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from .env (if available)
    dotenv().ok();

    // ========= Part 1: Retrieve NFT Data =========
    // For devnstration, we retrieve NFT data for token ID 1.
    let token_id = 1;
    let nft_data = get_nft_data(token_id)
        .await
        .context("Failed to retrieve NFT data")?;
    println!("Retrieved NFT Data:\n{:#?}\n", nft_data);

    // ========= Part 2: Build a RAG Pipeline Using Local LLM =========
    // Build a context string from NFT details.
    let context_data = format!(
        "NFT Data:\nToken ID: {}\nToken URI: {}\nMetadata: {}",
        nft_data.token_id, nft_data.token_uri, nft_data.metadata
    );

    // Define the user’s query.
    let user_query = "Analyze the NFT transaction data and verify its provenance using blockchain and RAG."
        .to_string();

    // 1. Generate an embedding for the user query using our local (dummy) function.
    let query_embedding = embed_text_local(&user_query);

    // 2. Retrieve related documents from Qdrant.
    let retrieved_docs = search_qdrant_real(&query_embedding)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Warning: Qdrant search failed, using fallback. Error: {}", err);
            vec!["Fallback document: No Qdrant data retrieved.".to_string()]
        });

    // 3. Combine context: NFT data + retrieved documents.
    let combined_context = format!(
        "{}\n\nRetrieved Documents:\n{}",
        context_data,
        retrieved_docs.join("\n")
    );

    // 4. Query the local free LLM (Meta Llama/DeepSeek) with the combined context.
    let rag_answer = query_llm_local(&combined_context, &user_query)
        .context("Local LLM query failed")?;

    // ========= Part 3: Log the Query–Answer Pair on the Blockchain Ledger =========
    let mut blockchain: Vec<Block> = Vec::new();
    // Create the genesis block.
    if blockchain.is_empty() {
        blockchain.push(Block::new(0, "genesis", "genesis", "0"));
    }
    // Append the new query–answer transaction.
    let previous_hash = blockchain.last().unwrap().hash.clone();
    let new_block = Block::new(
        blockchain.len() as u64,
        &user_query,
        &rag_answer,
        &previous_hash,
    );
    blockchain.push(new_block);

    // Save the ledger to a file.
    save_blockchain(&blockchain)?;

    // ========= Output the Results =========
    println!("User Query:\n{}\n", user_query);
    println!("Combined Context:\n{}\n", combined_context);
    println!("LLM Answer:\n{}\n", rag_answer);
    println!("Blockchain Ledger:");
    for block in &blockchain {
        println!("{:#?}", block);
    }

    Ok(())
}
