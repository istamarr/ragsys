//! Agent Protocol Client for NFT Integration
//! 
//! Connects nft_pawn_chain to agent_protocol's NFT endpoints:
//! - POST /scheme/nft/flow   - Index NFT metadata in Qdrant
//! - POST /scheme/nft/search - Search NFTs by similarity
//!
//! Usage in nft_pawn_chain:
//!   let client = AgentProtocolClient::new();
//!   client.index_nft(&struk, &ipfs_cid).await?;
//!   let similar = client.search_nft("cyberpunk NFT").await?;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use log::{info, warn, error};

/// Agent Protocol API Client
pub struct AgentProtocolClient {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
}

// ============================================================================
// Request/Response Structures
// ============================================================================

/// NFT Flow Request - matches agent_protocol's NftPayload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftFlowRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NftMetadataPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_data: Option<NftAiDataPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// NFT Metadata for agent_protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftMetadataPayload {
    pub name: String,
    pub description: String,
    pub image: String,
    #[serde(default)]
    pub attributes: Vec<NftAttributePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAttributePayload {
    pub trait_type: String,
    pub value: String,
}

/// AI Data for agent_protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAiDataPayload {
    pub prompt: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub generation_params: HashMap<String, String>,
    pub creation_timestamp: String,
    #[serde(default)]
    pub creation_logs: Vec<String>,
}

/// NFT Search Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_owner: Option<String>,
}

fn default_limit() -> usize { 10 }

/// NFT Flow Response from agent_protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftFlowResponse {
    pub type_response: String,
    pub status: String,
    pub processed_count: usize,
    #[serde(default)]
    pub content: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// NFT Search Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSearchResponse {
    pub status: String,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub results: Vec<NftSearchResult>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSearchResult {
    pub id: String,
    pub score: f32,
    pub ipfs_cid: String,
    pub content: String,
    pub chunk_type: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Login request for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    pw: String,
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

// ============================================================================
// Client Implementation
// ============================================================================

impl AgentProtocolClient {
    /// Create new client with default or env-configured URL
    pub fn new() -> Self {
        let base_url = env::var("AGENT_PROTOCOL_URL")
            .unwrap_or_else(|_| "http://localhost:3030".to_string());
        
        Self {
            base_url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            auth_token: env::var("AGENT_PROTOCOL_TOKEN").ok(),
        }
    }

    /// Create client with specific URL
    pub fn with_url(url: &str) -> Self {
        Self {
            base_url: url.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            auth_token: env::var("AGENT_PROTOCOL_TOKEN").ok(),
        }
    }

    /// Authenticate and get token
    pub async fn login(&mut self, email: &str, password: &str) -> Result<String> {
        let url = format!("{}/login", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&LoginRequest {
                email: email.to_string(),
                pw: password.to_string(),
            })
            .send()
            .await
            .context("Failed to connect to agent_protocol for login")?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Login failed: {}", err));
        }

        let login_resp: LoginResponse = response.json().await
            .context("Failed to parse login response")?;
        
        self.auth_token = Some(login_resp.token.clone());
        info!("AgentProtocol: Logged in successfully");
        
        Ok(login_resp.token)
    }

    /// Set auth token directly
    pub fn set_token(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
    }

    /// Check if agent_protocol server is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/srv/healthchecker", self.base_url);
        match self.client.get(&url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    /// Index NFT in agent_protocol (stores in Qdrant for search)
    pub async fn index_nft(&self, request: NftFlowRequest) -> Result<NftFlowResponse> {
        let url = format!("{}/scheme/nft/flow", self.base_url);
        
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to connect to agent_protocol for NFT indexing")?;

        if !response.status().is_success() {
            let status = response.status();
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("NFT indexing failed ({}): {}", status, err));
        }

        let flow_resp: NftFlowResponse = response.json().await
            .context("Failed to parse NFT flow response")?;
        
        info!("AgentProtocol: Indexed {} NFT(s)", flow_resp.processed_count);
        Ok(flow_resp)
    }

    /// Search NFTs by similarity
    pub async fn search_nft(&self, query: &str, limit: usize) -> Result<NftSearchResponse> {
        let url = format!("{}/scheme/nft/search", self.base_url);
        
        let request = NftSearchRequest {
            query: query.to_string(),
            limit,
            filter_chain: None,
            filter_owner: None,
        };

        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to connect to agent_protocol for NFT search")?;

        if !response.status().is_success() {
            let status = response.status();
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("NFT search failed ({}): {}", status, err));
        }

        let search_resp: NftSearchResponse = response.json().await
            .context("Failed to parse NFT search response")?;
        
        info!("AgentProtocol: Found {} similar NFT(s)", search_resp.count);
        Ok(search_resp)
    }

    /// Search NFTs with filters
    pub async fn search_nft_filtered(
        &self, 
        query: &str, 
        limit: usize,
        chain: Option<&str>,
        owner: Option<&str>,
    ) -> Result<NftSearchResponse> {
        let url = format!("{}/scheme/nft/search", self.base_url);
        
        let request = NftSearchRequest {
            query: query.to_string(),
            limit,
            filter_chain: chain.map(|s| s.to_string()),
            filter_owner: owner.map(|s| s.to_string()),
        };

        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to connect to agent_protocol for NFT search")?;

        let search_resp: NftSearchResponse = response.json().await
            .context("Failed to parse NFT search response")?;
        
        Ok(search_resp)
    }

    /// Index NFT by IPFS CID only
    pub async fn index_nft_by_cid(&self, cid: &str, title: &str) -> Result<NftFlowResponse> {
        let request = NftFlowRequest {
            title: title.to_string(),
            cid: Some(cid.to_string()),
            cids: None,
            metadata: None,
            ai_data: None,
            token_id: None,
            owner_address: None,
            contract_address: None,
            chain: None,
        };
        
        self.index_nft(request).await
    }

    /// Index multiple CIDs
    pub async fn index_nft_batch(&self, cids: Vec<String>, title: &str) -> Result<NftFlowResponse> {
        let request = NftFlowRequest {
            title: title.to_string(),
            cid: None,
            cids: Some(cids),
            metadata: None,
            ai_data: None,
            token_id: None,
            owner_address: None,
            contract_address: None,
            chain: None,
        };
        
        self.index_nft(request).await
    }
}

// ============================================================================
// Helper Functions for StrukGadaiNft Integration
// ============================================================================

/// Convert StrukGadaiNft to NftFlowRequest for indexing
/// This function should be called after IPFS storage to index in Qdrant
pub fn struk_to_nft_request(
    struk_nomor: &str,
    ipfs_cid: &str,
    nama_koleksi: &str,
    token_id: &str,
    contract_address: &str,
    owner_address: &str,
    description: &str,
    blockchain: &str,
    attributes: Vec<(String, String)>,
) -> NftFlowRequest {
    let attr_payload: Vec<NftAttributePayload> = attributes
        .into_iter()
        .map(|(trait_type, value)| NftAttributePayload { trait_type, value })
        .collect();

    NftFlowRequest {
        title: struk_nomor.to_string(),
        cid: Some(ipfs_cid.to_string()),
        cids: None,
        metadata: Some(NftMetadataPayload {
            name: format!("{} #{}", nama_koleksi, token_id),
            description: description.to_string(),
            image: format!("ipfs://{}", ipfs_cid),
            attributes: attr_payload,
            ai_data: None,
        }),
        ai_data: None,
        token_id: Some(token_id.to_string()),
        owner_address: Some(owner_address.to_string()),
        contract_address: Some(contract_address.to_string()),
        chain: Some(blockchain.to_string()),
    }
}

/// Create AI data payload for NFT that was AI-generated
pub fn create_ai_data_payload(
    prompt: &str,
    model: &str,
    agent_id: Option<&str>,
    logs: Vec<String>,
) -> NftAiDataPayload {
    NftAiDataPayload {
        prompt: prompt.to_string(),
        model: model.to_string(),
        model_version: None,
        agent_id: agent_id.map(|s| s.to_string()),
        generation_params: HashMap::new(),
        creation_timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        creation_logs: logs,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let client = AgentProtocolClient::new();
        assert!(!client.base_url.is_empty());
    }

    #[test]
    fn test_struk_to_request() {
        let request = struk_to_nft_request(
            "SGN-20241213-TEST",
            "QmTestCID123",
            "Test Collection",
            "1",
            "0x1234",
            "0x5678",
            "Test NFT",
            "ethereum",
            vec![("Rarity".to_string(), "Rare".to_string())],
        );
        
        assert_eq!(request.title, "SGN-20241213-TEST");
        assert!(request.cid.is_some());
        assert!(request.metadata.is_some());
    }
}
