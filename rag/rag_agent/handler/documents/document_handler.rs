use qdrant_client::{
     //prelude::*,
};
use std::env;
use anyhow::{Context, Result};
use log::{info, Level};
use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams};
use serde_json::json;
use qdrant_client::qdrant::{QueryPointsBuilder, SearchParamsBuilder};
use uuid::Uuid;
use crate::rag_agent::embedding::quickEmbedding::generate_embedding_local;
use crate::shared::helperUtils::{utils_embedding_vec_dim, TagValidator};
use crate::shared::secureUtils::generate_api_key;
use crate::shared::sharedUtils::VECTOR_SIZE;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::PointStruct;
use crate::domain::dbs_rag_LM::Payload;

async fn ingest_documents(
    qdrant_client: &Qdrant,
    documents: &[&str],
) -> Result<()> {
    let collection_name = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL belum diatur")?;
    let _ = qdrant_client.create_collection(CreateCollection {
        collection_name: collection_name.to_string(),
        vectors_config: Some(VectorParams {
            size: 768, // nomic-embed-text dimension
            distance: Distance::Cosine.into(),
            ..Default::default()
        }.into()),
        ..Default::default()
    }).await;

    let mut points = Vec::new();
    for (i, doc) in documents.iter().enumerate() {
        let embedding = generate_embedding_local(doc).await?;

        let payload: Payload = json!(
        {"text": doc, }).try_into().unwrap();
        let point = PointStruct::new(
            i as u64,
            embedding,
            payload,
        );
        points.push(point);
        info!("embedding {:?}", doc);
    }

    upsert_result("data_chaining".to_string(), "Model-Context-Protocol".to_string(), generate_api_key(), "documents")
        .await.expect("ingest_documents: upsert_result");

    info!("Ingested {} documents", documents.len());
    Ok(())
}

async fn upsert_result(title: String, model: String, uuid: String, resp_text:&str) -> Result<()> {
    info!("upsert result get data");
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
    let collection_name = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL belum diatur")?;
    let point_id = Uuid::new_v4().to_string();
    let embedding_vector = utils_embedding_vec_dim(resp_text, VECTOR_SIZE);
    let string_resp_text: String = resp_text.parse().unwrap();

    let qdrant_client = reqwest::Client::new();
    let point = json!({
        "source": model,
        "id": point_id,
        "vector": embedding_vector,
        "payload": {
           "id": uuid,
           "title": title,
           "content": string_resp_text,
        }
    });
    let upsert_url = format!("{}/collections/{}/points?wait=true", qdrant_url, collection_name);
    let req_body = json!({ "points": [point] });
    info!("req_body {:?}", req_body);
    let resp = qdrant_client
        .put(&upsert_url)
        .json(&req_body)
        .send()
        .await
        .context("Gagal mengirim upsert ke Qdrant")?;
    let resp_text = resp.text().await.context("Gagal membaca response Qdrant")?;
    info!("Response Qdrant: {}", resp_text);
    Ok(())
}
