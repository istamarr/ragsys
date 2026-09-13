use std::error::Error;
use crate::domain::models::llm::RequestBody;

//lsg ke bchain/bchain.rs
///add this deep size for 1024 dim and make it answer for blockchain (resize feature)
///now condition: answer -> llm -> qdrant
///resulting quick data with text answer analisys

fn quickEmbeddingBlockChain (body: RequestBody) -> anyhow::Result<String, Box<dyn Error>> {

    Ok(("".to_string()))
}