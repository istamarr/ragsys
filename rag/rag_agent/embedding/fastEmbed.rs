/**
 * FastEmbed — pure-Rust local embedding (no HTTP, no Ollama required)
 * Model : AllMiniLML6V2  → 384-dim vectors (cosine)
 * Access: crate::rag_agent::embedding::fastEmbed
 */
use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::OnceCell;
use qdrant_client::qdrant::{PointStruct, QueryPointsBuilder, UpsertPointsBuilder};
use qdrant_client::{Payload, Qdrant};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

// ── singleton model (loaded once, reused across calls) ────────────────────────

static MODEL: OnceCell<Mutex<TextEmbedding>> = OnceCell::new();

fn get_model() -> Result<&'static Mutex<TextEmbedding>> {
    MODEL.get_or_try_init(|| {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(false),
        )
        .context("fastembed: failed to init AllMiniLML6V2")?;
        Ok(Mutex::new(model))
    })
}

/// Vector dimension produced by AllMiniLML6V2
pub const EMBED_DIM: usize = 384;

// ── sync core (safe to call inside spawn_blocking) ────────────────────────────

/// Embed a batch of texts synchronously.
/// Returns one `Vec<f32>` per input (length = `EMBED_DIM`).
pub fn embed_texts_sync(texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let mutex = get_model()?;
    let model = mutex
        .lock()
        .map_err(|e| anyhow::anyhow!("fastembed mutex poisoned: {e}"))?;
    let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    model.embed(refs, None).context("fastembed: embed() failed")
}

/// Embed a single text synchronously.
pub fn embed_text_sync(text: &str) -> Result<Vec<f32>> {
    embed_texts_sync(vec![text.to_string()])?
        .into_iter()
        .next()
        .context("fastembed: empty result for single text")
}

// ── async wrappers ────────────────────────────────────────────────────────────

/// Async single-text embedding.
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let owned = text.to_string();
    tokio::task::spawn_blocking(move || embed_text_sync(&owned))
        .await
        .context("fastembed spawn_blocking panicked")?
}

/// Async batch embedding.
pub async fn generate_embeddings_batch(texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    tokio::task::spawn_blocking(move || embed_texts_sync(texts))
        .await
        .context("fastembed batch spawn_blocking panicked")?
}

// ── Qdrant helpers ────────────────────────────────────────────────────────────

/// Embed `text` and upsert a point into Qdrant with the provided JSON payload.
pub async fn embed_and_upsert_qdrant(
    client: &Qdrant,
    collection: &str,
    id: u64,
    text: &str,
    payload: Value,
) -> Result<()> {
    let vector = generate_embedding(text).await?;
    let qdrant_payload = Payload::try_from(payload)
        .context("fastembed: invalid Qdrant payload")?;
    client
        .upsert_points(
            UpsertPointsBuilder::new(
                collection,
                vec![PointStruct::new(id, vector, qdrant_payload)],
            )
            .wait(true),
        )
        .await
        .context("fastembed: Qdrant upsert failed")?;
    Ok(())
}

/// Embed `query`, search Qdrant, return `text` field from each result.
pub async fn embed_and_search_qdrant(
    client: &Qdrant,
    collection: &str,
    query: &str,
    limit: u64,
) -> Result<Vec<String>> {
    let vector = generate_embedding(query).await?;
    let result = client
        .query(
            QueryPointsBuilder::new(collection)
                .query(vector)
                .limit(limit)
                .with_payload(true),
        )
        .await
        .context("fastembed: Qdrant search failed")?;
    Ok(result
        .result
        .into_iter()
        .filter_map(|pt| {
            pt.payload
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect())
}

/// Embed `query`, search Qdrant, return full payload maps for each result.
pub async fn search_qdrant_payloads(
    client: &Qdrant,
    collection: &str,
    query: &str,
    limit: u64,
) -> Result<Vec<HashMap<String, Value>>> {
    let vector = generate_embedding(query).await?;
    let result = client
        .query(
            QueryPointsBuilder::new(collection)
                .query(vector)
                .limit(limit)
                .with_payload(true),
        )
        .await
        .context("fastembed: Qdrant payload search failed")?;
    Ok(result
        .result
        .into_iter()
        .map(|pt| {
            pt.payload
                .into_iter()
                .filter_map(|(k, v)| serde_json::to_value(&v).ok().map(|jv| (k, jv)))
                .collect()
        })
        .collect())
}

// ── candle_eng pipe ───────────────────────────────────────────────────────────
// serve_causal (candle_eng) runs on port 8082
// POST /api/v1/query  { query, context?, max_tokens, temperature }
// Response            { answer, response, tokens_generated, processing_time_ms }

/// Configuration for the candle_eng causal inference server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleEngConfig {
    /// Base URL of the serve_causal server.  Default: http://localhost:8082
    pub url: String,
    /// Max tokens to generate.  Default: 200
    pub max_tokens: usize,
    /// Sampling temperature.  Default: 0.7
    pub temperature: f64,
}

impl Default for CandleEngConfig {
    fn default() -> Self {
        Self {
            url: env::var("CANDLE_ENG_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            max_tokens: env::var("CANDLE_ENG_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
            temperature: env::var("CANDLE_ENG_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CandleQueryResponse {
    answer: Option<String>,
    response: Option<String>,
}

/// Send `query` + optional `context` to the candle_eng serve_causal server
/// and return the generated text.
///
/// Env overrides: `CANDLE_ENG_URL`, `CANDLE_ENG_MAX_TOKENS`, `CANDLE_ENG_TEMPERATURE`
pub async fn pipe_to_candle_eng(
    query: &str,
    context: Option<&str>,
    cfg: &CandleEngConfig,
) -> Result<String> {
    let client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("candle_eng: failed to build HTTP client")?;

    let body = serde_json::json!({
        "query":       query,
        "context":     context.unwrap_or(""),
        "max_tokens":  cfg.max_tokens,
        "temperature": cfg.temperature,
    });

    let url = format!("{}/api/v1/query", cfg.url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("candle_eng: HTTP request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("candle_eng error {status}: {text}"));
    }

    let parsed: CandleQueryResponse = resp
        .json()
        .await
        .context("candle_eng: failed to parse response JSON")?;

    parsed
        .answer
        .or(parsed.response)
        .filter(|s| !s.is_empty())
        .context("candle_eng: empty answer in response")
}

/// Full pipeline: embed query → Qdrant search → pipe context to candle_eng.
///
/// ```text
/// query
///   └─► fastembed (local, 384-dim)
///         └─► Qdrant search  (top `limit` chunks)
///               └─► candle_eng /api/v1/query  (port 8082)
///                     └─► answer
/// ```
pub async fn embed_search_and_infer(
    qdrant: &Qdrant,
    collection: &str,
    query: &str,
    limit: u64,
    candle_cfg: &CandleEngConfig,
) -> Result<String> {
    let chunks = embed_and_search_qdrant(qdrant, collection, query, limit).await?;
    let context = if chunks.is_empty() {
        None
    } else {
        Some(chunks.join("\n\n"))
    };

    pipe_to_candle_eng(query, context.as_deref(), candle_cfg).await
}
