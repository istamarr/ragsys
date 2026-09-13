use md5;
use reqwest;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::domain::models::llm::RequestBody;
use std::{error::Error, io, time::Duration, env};
use std::fmt::format;
use nom::Parser;
use crate::shared::sharedUtils::MaxChar;

#[derive(Debug, Clone)]
struct Document {
    id: String,
    content: String,
}

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
    pub fn new(index: u64, query: &str, answer: &str, previous_hash: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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

fn embed_text(text: &str) -> Vec<f32> {
    let mut embedding = vec![0.0; 8];
    for (i, byte) in text.bytes().enumerate() {
        embedding[i % 8] += byte as f32 / 255.0;
    }
    embedding
}

async fn search_qdrant(_query_embedding: Vec<f32>, param: String) -> Vec<Document> {
    vec![
        Document {
            id: "1".into(),
            content: param.clone().into(),
        },
        Document {
            id: "2".into(),
            content: param.clone().into(),//pakai body ambil beda parameter
        },
    ]
}

async fn query_llm(context: String, question: String) -> String {
    format!("{} {}", context, question)
}

pub async fn bchain(body: RequestBody) -> Result<String, Box<dyn Error>> { /// Result<(), Box<dyn Error>> {
    let query= body.prompt.to_string();
    let query_embedding = embed_text(&query);
    let documents = search_qdrant(query_embedding, "parameter".to_string()).await;
    let context = documents
        .iter()
        .map(|doc| format!("{}: {}", doc.id, doc.content))
        .collect::<Vec<String>>()
        .join("\n");

    let answer = query_llm(context.clone(), query.clone()).await;
    let mut blockchain: Vec<Block> = Vec::new();
    if blockchain.is_empty() {
        let genesis = Block::new(0, "genesis", "genesis", "0");
        blockchain.push(genesis);
    }

    let previous_hash = blockchain.last().unwrap().hash.clone();
    let new_block = Block::new(blockchain.len() as u64, &query, &answer, &previous_hash);
    blockchain.push(new_block);

    let mut ledgerResult = "";
    for block in &blockchain {
        println!("{:#?}", block);

        let mut ledger_result_add = format!("{:?} - ", block).as_str();
        ledgerResult = "";
    }

    let mut checkMaxLength = format!("Retrieved Context:\n{}\n - LLM Result:\n{}\n - Blockchain Ledger:\n{}\n ", context.clone(), answer.clone(), ledgerResult.clone());
    let maxCharUtil: usize = MaxChar::MAX_CHAR_IO_TXT_10240.max.parse().unwrap();
    if(checkMaxLength.len()>maxCharUtil){
        return Ok(format!("Warning: Result Maximum Character to {:?}", maxCharUtil.clone()));
    }

    Ok(format!("Retrieved Context:\n{}\n - LLM Result:\n{}\n - Blockchain Ledger:\n{}\n ", context.clone(), answer.clone(), ledgerResult.clone()))
}
