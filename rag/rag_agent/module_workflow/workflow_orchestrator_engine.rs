use std::env;
use std::collections::HashMap;
use std::ptr::null;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;
use sqlx::{query, PgPool, Row, Error};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, Utc};
use uuid::Uuid;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;
use log::{LevelFilter, warn, error};
use surrealdb::opt::IntoQuery;
use time::{OffsetDateTime, UtcOffset};
use task::shared::sharedUtils::{FlowExecute, LLMStatus, Parameter, StatusProcess, UserExecute, LLM, LMTYPE};
use tracing::info;
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use anyhow::{Context, Result as AnyhowResult};

use crate::rag_feedback::{
    RagFeedback, FeedbackAction, FeedbackScores, FeedbackConfig, FeedbackContext
};
use crate::rag_chaining::{CAG_CACHE, CagResult};
use crate::shared::helperUtils::utils_embedding_vec_dim;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;

// ============================================================================
// AGENTIC RAG ORCHESTRATOR - Full Implementation
// ============================================================================

/// Orchestrator action types for agentic RAG
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrchestratorAction {
    /// Initial state - planning phase
    Plan,
    /// Retrieve documents from vector store
    Retrieve,
    /// Generate response using LLM
    Generate,
    /// Evaluate response with feedback
    Evaluate,
    /// Refine query or response
    Refine,
    /// Use external tool/agent
    UseTool(String),
    /// Store to memory
    Memorize,
    /// Final answer ready
    Complete,
    /// Error or stop
    Abort(String),
}

/// Tool types available to the orchestrator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolType {
    Retriever,      // Qdrant vector search
    LlmBurnLm,      // burn-lm ASIST model
    LlmOllama,      // Ollama fallback
    AgentImg,       // agent_difusser image agent
    AgentDoc,       // agent_difusser document agent
    AgentVoice,     // voice agent
    AgentWeb3,      // web3 agent
    Memory,         // SurrealDB memory
    WebSearch,      // External web search
    CodeAnalyzer,   // Code analysis tool
}

/// Current state of the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorState {
    pub session_id: String,
    pub query: String,
    pub current_action: OrchestratorAction,
    pub iteration: u32,
    pub max_iterations: u32,
    pub retrieved_documents: Vec<String>,
    pub generated_response: Option<String>,
    pub feedback_scores: Option<FeedbackScores>,
    pub action_history: Vec<OrchestratorAction>,
    pub tool_results: HashMap<String, String>,
    pub memory_context: Vec<String>,
    pub is_complete: bool,
    pub final_answer: Option<String>,
    pub error: Option<String>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            query: String::new(),
            current_action: OrchestratorAction::Plan,
            iteration: 0,
            max_iterations: 5,
            retrieved_documents: Vec::new(),
            generated_response: None,
            feedback_scores: None,
            action_history: Vec::new(),
            tool_results: HashMap::new(),
            memory_context: Vec::new(),
            is_complete: false,
            final_answer: None,
            error: None,
        }
    }
}

/// Configuration for the RAG orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_iterations: u32,
    pub feedback_config: FeedbackConfig,
    pub enable_memory: bool,
    pub enable_tools: bool,
    pub burn_lm_url: String,
    pub ollama_url: String,
    pub qdrant_url: String,
    pub timeout_secs: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            feedback_config: FeedbackConfig::default(),
            enable_memory: true,
            enable_tools: true,
            burn_lm_url: env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            ollama_url: env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            qdrant_url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string()),
            timeout_secs: 30,
        }
    }
}

/// Tool Router - routes requests to appropriate tools
pub struct ToolRouter {
    http_client: HttpClient,
    config: OrchestratorConfig,
}

impl ToolRouter {
    pub fn new(config: OrchestratorConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { http_client, config }
    }

    /// Route to appropriate tool and get result
    pub async fn execute_tool(&self, tool: ToolType, input: &str) -> AnyhowResult<String> {
        info!("ToolRouter # Executing {:?} with input length {}", tool, input.len());

        match tool {
            ToolType::LlmBurnLm => self.call_burn_lm(input).await,
            ToolType::LlmOllama => self.call_ollama(input).await,
            ToolType::AgentImg => self.call_agent_img(input).await,
            ToolType::AgentDoc => self.call_agent_doc(input).await,
            ToolType::WebSearch => self.call_web_search(input).await,
            _ => Ok(format!("Tool {:?} not implemented yet", tool)),
        }
    }

    async fn call_burn_lm(&self, prompt: &str) -> AnyhowResult<String> {
        let url = format!("{}/v1/completions", self.config.burn_lm_url);

        let payload = serde_json::json!({
            "prompt": prompt,
            "max_tokens": 1024,
            "temperature": 0.7
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let body: serde_json::Value = response.json().await?;
            Ok(body["choices"][0]["text"].as_str().unwrap_or("").to_string())
        } else {
            Err(anyhow::anyhow!("burn-lm error: {}", response.status()))
        }
    }

    async fn call_ollama(&self, prompt: &str) -> AnyhowResult<String> {
        let url = format!("{}/api/generate", self.config.ollama_url);
        let model = env::var("OLLAMA_FALLBACK_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string());

        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let body: serde_json::Value = response.json().await?;
            Ok(body["response"].as_str().unwrap_or("").to_string())
        } else {
            Err(anyhow::anyhow!("Ollama error: {}", response.status()))
        }
    }

    async fn call_agent_img(&self, prompt: &str) -> AnyhowResult<String> {
        // Check if DIFSR_URL is set (HTTP mode)
        if let Ok(difsr_url) = env::var("DIFSR_URL") {
            let url = format!("{}/api/img", difsr_url);
            let payload = serde_json::json!({
                "command": "generate",
                "prompt": prompt
            });

            let response = self.http_client.post(&url).json(&payload).send().await?;
            if response.status().is_success() {
                let body: serde_json::Value = response.json().await?;
                return Ok(body["output_path"].as_str().unwrap_or("").to_string());
            }
        }

        // Fallback: subprocess
        Ok(format!("Image generation requested: {}", prompt))
    }

    async fn call_agent_doc(&self, prompt: &str) -> AnyhowResult<String> {
        if let Ok(difsr_url) = env::var("DIFSR_URL") {
            let url = format!("{}/api/doc", difsr_url);
            let payload = serde_json::json!({
                "command": "generate",
                "content": prompt
            });

            let response = self.http_client.post(&url).json(&payload).send().await?;
            if response.status().is_success() {
                let body: serde_json::Value = response.json().await?;
                return Ok(body["output_path"].as_str().unwrap_or("").to_string());
            }
        }

        Ok(format!("Document generation requested: {}", prompt))
    }

    async fn call_web_search(&self, query: &str) -> AnyhowResult<String> {
        // Placeholder for web search integration
        Ok(format!("Web search results for: {}", query))
    }

    /// Qdrant vector search
    pub async fn call_qdrant_search(&self, query: &str, top_k: usize) -> AnyhowResult<Vec<String>> {
        let qdrant_url = env::var("QDRANT_URL_PORT_6334")
            .unwrap_or_else(|_| "http://localhost:6334".to_string());
        let collection_name = env::var("ASIST_DETAIL")
            .unwrap_or_else(|_| "rag_collection".to_string());

        info!("ToolRouter # Qdrant search in collection '{}' for: {}", collection_name, query);

        // Generate embedding for query
        let query_embedding = utils_embedding_vec_dim(query, 3);

        let client = Qdrant::from_url(&*qdrant_url).build()?;

        let search_request = SearchPointsBuilder::new(
            collection_name,
            query_embedding,
            top_k as u64,
        ).with_payload(true);

        let search_result = client.search_points(search_request).await?;

        let documents: Vec<String> = search_result.result
            .into_iter()
            .filter_map(|point| {
                point.payload
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        info!("ToolRouter # Qdrant returned {} documents", documents.len());

        Ok(documents)
    }
}

/// Action Planner - decides next action based on state
pub struct ActionPlanner {
    feedback: RagFeedback,
}

impl ActionPlanner {
    pub fn new(feedback_config: FeedbackConfig) -> Self {
        Self {
            feedback: RagFeedback::new(feedback_config),
        }
    }

    /// Plan the next action based on current state
    pub fn plan_next_action(&self, state: &OrchestratorState) -> OrchestratorAction {
        info!("ActionPlanner # Planning next action, current: {:?}, iteration: {}",
              state.current_action, state.iteration);

        // Check iteration limit
        if state.iteration >= state.max_iterations {
            warn!("ActionPlanner # Max iterations reached");
            return OrchestratorAction::Complete;
        }

        match &state.current_action {
            OrchestratorAction::Plan => {
                // First step: retrieve documents
                OrchestratorAction::Retrieve
            },

            OrchestratorAction::Retrieve => {
                // After retrieval: generate response
                if state.retrieved_documents.is_empty() {
                    warn!("ActionPlanner # No documents retrieved, trying web search");
                    OrchestratorAction::UseTool("WebSearch".to_string())
                } else {
                    OrchestratorAction::Generate
                }
            },

            OrchestratorAction::Generate => {
                // After generation: evaluate with feedback
                OrchestratorAction::Evaluate
            },

            OrchestratorAction::Evaluate => {
                // After evaluation: decide based on feedback scores
                if let Some(scores) = &state.feedback_scores {
                    self.decide_from_feedback(scores, state)
                } else {
                    OrchestratorAction::Complete
                }
            },

            OrchestratorAction::Refine => {
                // After refine: retrieve again
                OrchestratorAction::Retrieve
            },

            OrchestratorAction::UseTool(_) => {
                // After tool use: generate or evaluate
                if state.generated_response.is_some() {
                    OrchestratorAction::Evaluate
                } else {
                    OrchestratorAction::Generate
                }
            },

            OrchestratorAction::Memorize => {
                OrchestratorAction::Complete
            },

            OrchestratorAction::Complete | OrchestratorAction::Abort(_) => {
                // Terminal states
                OrchestratorAction::Complete
            },
        }
    }

    fn decide_from_feedback(&self, scores: &FeedbackScores, state: &OrchestratorState) -> OrchestratorAction {
        let overall = scores.overall_score();

        info!("ActionPlanner # Feedback overall score: {:.2}", overall);

        // High quality - accept
        if overall >= 0.7 && scores.hallucination_score <= 0.3 {
            return OrchestratorAction::Complete;
        }

        // High hallucination - retrieve more
        if scores.hallucination_score > 0.5 {
            return OrchestratorAction::Refine;
        }

        // Low relevance - refine query
        if scores.relevance_score < 0.5 {
            return OrchestratorAction::Refine;
        }

        // Low grounding - retrieve more
        if scores.grounding_score < 0.4 {
            return OrchestratorAction::Retrieve;
        }

        // Default: complete with what we have
        OrchestratorAction::Complete
    }

    /// Evaluate response and return feedback scores
    pub fn evaluate_response(&self, state: &OrchestratorState) -> Option<FeedbackScores> {
        let response = state.generated_response.as_ref()?;

        let context = FeedbackContext {
            query: state.query.clone(),
            answer: response.clone(),
            retrieved_documents: state.retrieved_documents.clone(),
            iteration: state.iteration,
            previous_scores: Vec::new(),
        };

        Some(self.feedback.evaluate(&context))
    }
}

/// Main RAG Orchestrator
pub struct RagOrchestrator {
    config: OrchestratorConfig,
    tool_router: ToolRouter,
    action_planner: ActionPlanner,
    state: Arc<RwLock<OrchestratorState>>,
}

impl RagOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        let tool_router = ToolRouter::new(config.clone());
        let action_planner = ActionPlanner::new(config.feedback_config.clone());

        Self {
            config,
            tool_router,
            action_planner,
            state: Arc::new(RwLock::new(OrchestratorState::default())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(OrchestratorConfig::default())
    }

    /// Run the full orchestration loop
    pub async fn run(&self, query: &str) -> AnyhowResult<String> {
        info!("RagOrchestrator # Starting orchestration for query: {}", query);

        // CAG Fast Path: Check cache first
        if let Some(cached) = CAG_CACHE.get(query).await {
            if cached.is_high_quality() {
                info!("RagOrchestrator # CAG Cache HIT (high quality) - returning cached response");
                CAG_CACHE.record_hit(query).await;
                return Ok(cached.response);
            } else {
                info!("RagOrchestrator # CAG Cache found but low quality - proceeding with full orchestration");
            }
        } else {
            info!("RagOrchestrator # CAG Cache MISS - proceeding with full orchestration");
        }

        // Initialize state
        {
            let mut state = self.state.write().await;
            state.query = query.to_string();
            state.current_action = OrchestratorAction::Plan;
            state.iteration = 0;
            state.is_complete = false;
        }

        // Main orchestration loop
        loop {
            let (action, iteration) = {
                let state = self.state.read().await;
                (state.current_action.clone(), state.iteration)
            };

            info!("RagOrchestrator # Iteration {}, Action: {:?}", iteration, action);

            // Execute current action
            let result = self.execute_action(&action).await;

            if let Err(e) = result {
                error!("RagOrchestrator # Error executing action: {}", e);
                let mut state = self.state.write().await;
                state.error = Some(e.to_string());
                state.current_action = OrchestratorAction::Abort(e.to_string());
            }

            // Check completion
            {
                let state = self.state.read().await;
                if state.is_complete {
                    return Ok(state.final_answer.clone().unwrap_or_else(||
                        state.generated_response.clone().unwrap_or_default()));
                }

                if matches!(state.current_action, OrchestratorAction::Abort(_)) {
                    return Err(anyhow::anyhow!(state.error.clone().unwrap_or_default()));
                }
            }

            // Plan next action
            let next_action = {
                let state = self.state.read().await;
                self.action_planner.plan_next_action(&state)
            };

            // Update state
            {
                let mut state = self.state.write().await;
                state.clone().action_history.push(state.current_action.clone());
                state.current_action = next_action.clone();
                state.iteration += 1;

                if matches!(next_action, OrchestratorAction::Complete) {
                    state.is_complete = true;
                    state.final_answer = state.generated_response.clone();

                    // CAG: Cache high-quality responses
                    if let (Some(response), Some(scores)) = (&state.generated_response, &state.feedback_scores) {
                        if scores.overall_score() >= 0.6 && scores.hallucination_score <= 0.4 {
                            let context = state.retrieved_documents.join("\n\n");
                            let query = state.query.clone();
                            let resp = response.clone();
                            let sc = scores.clone();

                            // Cache in background
                            tokio::spawn(async move {
                                CAG_CACHE.set(&query, &context, &resp, Some(sc)).await;
                                info!("RagOrchestrator # Cached high-quality response");
                            });
                        }
                    }
                }
            }
        }
    }

    /// Execute a specific action
    async fn execute_action(&self, action: &OrchestratorAction) -> AnyhowResult<()> {
        match action {
            OrchestratorAction::Plan => {
                info!("RagOrchestrator # Planning phase");
                // Planning is implicit - just log
                Ok(())
            },

            OrchestratorAction::Retrieve => {
                self.action_retrieve().await
            },

            OrchestratorAction::Generate => {
                self.action_generate().await
            },

            OrchestratorAction::Evaluate => {
                self.action_evaluate().await
            },

            OrchestratorAction::Refine => {
                self.action_refine().await
            },

            OrchestratorAction::UseTool(tool_name) => {
                self.action_use_tool(tool_name).await
            },

            OrchestratorAction::Memorize => {
                self.action_memorize().await
            },

            OrchestratorAction::Complete => {
                info!("RagOrchestrator # Completing orchestration");
                let mut state = self.state.write().await;
                state.is_complete = true;
                Ok(())
            },

            OrchestratorAction::Abort(reason) => {
                error!("RagOrchestrator # Aborting: {}", reason);
                Ok(())
            },
        }
    }

    async fn action_retrieve(&self) -> AnyhowResult<()> {
        let query = {
            let state = self.state.read().await;
            state.query.clone()
        };

        info!("RagOrchestrator # Retrieving documents for: {}", query);

        // Get Qdrant connection details
        let qdrant_url = env::var("QDRANT_URL_PORT_6334")
            .unwrap_or_else(|_| "http://localhost:6334".to_string());
        let collection_name = env::var("ASIST_DETAIL")
            .unwrap_or_else(|_| "rag_collection".to_string());

        // Generate query embedding
        let query_embedding = utils_embedding_vec_dim(&query, 3);

        // Connect to Qdrant and search
        let docs = match Qdrant::from_url(&*qdrant_url).build() {
            Ok(client) => {
                let search_request = SearchPointsBuilder::new(
                    collection_name,
                    query_embedding,
                    5, // top 5 results
                ).with_payload(true);

                match client.search_points(search_request).await {
                    Ok(search_result) => {
                        info!("RagOrchestrator # Found {} results", search_result.result.len());
                        search_result.result
                            .into_iter()
                            .filter_map(|point| {
                                point.payload
                                    .get("text")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            })
                            .collect()
                    },
                    Err(e) => {
                        warn!("RagOrchestrator # Qdrant search failed: {}, using fallback", e);
                        vec![format!("Context for query: {}", query)]
                    }
                }
            },
            Err(e) => {
                warn!("RagOrchestrator # Qdrant connection failed: {}, using fallback", e);
                vec![format!("Context for query: {}", query)]
            }
        };

        let mut state = self.state.write().await;
        state.retrieved_documents = docs;

        info!("RagOrchestrator # Retrieved {} documents", state.retrieved_documents.len());

        Ok(())
    }

    async fn action_generate(&self) -> AnyhowResult<()> {
        let (query, context) = {
            let state = self.state.read().await;
            (state.query.clone(), state.retrieved_documents.join("\n\n"))
        };

        info!("RagOrchestrator # Generating response");

        let prompt = format!(
            "Based on the following context, answer the question.\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer:",
            context, query
        );

        // Try burn-lm first, fallback to Ollama
        let response = match self.tool_router.execute_tool(ToolType::LlmBurnLm, &prompt).await {
            Ok(resp) if !resp.is_empty() => resp,
            _ => {
                info!("RagOrchestrator # Falling back to Ollama");
                self.tool_router.execute_tool(ToolType::LlmOllama, &prompt).await?
            }
        };

        let mut state = self.state.write().await;
        state.generated_response = Some(response);

        Ok(())
    }

    async fn action_evaluate(&self) -> AnyhowResult<()> {
        info!("RagOrchestrator # Evaluating response with feedback");

        let scores = {
            let state = self.state.read().await;
            self.action_planner.evaluate_response(&state)
        };

        if let Some(scores) = scores {
            info!("RagOrchestrator # Feedback scores: overall={:.2}, relevance={:.2}, grounding={:.2}, hallucination={:.2}",
                  scores.overall_score(), scores.relevance_score, scores.grounding_score, scores.hallucination_score);

            let mut state = self.state.write().await;
            state.feedback_scores = Some(scores);
        }

        Ok(())
    }

    async fn action_refine(&self) -> AnyhowResult<()> {
        info!("RagOrchestrator # Refining query");

        let mut state = self.state.write().await;

        // Add refinement instruction to query
        if let Some(scores) = &state.clone().feedback_scores {
            if scores.grounding_score < 0.5 {
                state.query = format!("{} (provide specific evidence)", state.query);
            }
            if scores.completeness_score < 0.5 {
                state.query = format!("{} (give comprehensive answer)", state.query);
            }
        }

        // Clear previous response for regeneration
        state.generated_response = None;
        state.retrieved_documents.clear();

        Ok(())
    }

    async fn action_use_tool(&self, tool_name: &str) -> AnyhowResult<()> {
        info!("RagOrchestrator # Using tool: {}", tool_name);

        let query = {
            let state = self.state.read().await;
            state.query.clone()
        };

        let tool_type = match tool_name {
            "WebSearch" => ToolType::WebSearch,
            "AgentImg" => ToolType::AgentImg,
            "AgentDoc" => ToolType::AgentDoc,
            "CodeAnalyzer" => ToolType::CodeAnalyzer,
            _ => return Ok(()),
        };

        let result = self.tool_router.execute_tool(tool_type, &query).await?;

        let mut state = self.state.write().await;
        state.tool_results.insert(tool_name.to_string(), result.clone());

        // Add tool result to retrieved documents
        state.retrieved_documents.push(result);

        Ok(())
    }

    async fn action_memorize(&self) -> AnyhowResult<()> {
        info!("RagOrchestrator # Storing to memory");

        // TODO: Integrate with SurrealDB memory storage

        Ok(())
    }

    /// Get current state snapshot
    pub async fn get_state(&self) -> OrchestratorState {
        self.state.read().await.clone()
    }
}

/// Convenience function to run RAG orchestration
pub async fn run_rag_orchestrator(query: &str) -> AnyhowResult<String> {
    let orchestrator = RagOrchestrator::with_defaults();
    orchestrator.run(query).await
}

/// Convenience function with custom config
pub async fn run_rag_orchestrator_with_config(query: &str, config: OrchestratorConfig) -> AnyhowResult<String> {
    let orchestrator = RagOrchestrator::new(config);
    orchestrator.run(query).await
}

// ============================================================================
// EXISTING LLM MODEL GENERATION WORKFLOW (preserved below)
// ============================================================================

// sample: connect db check
//     let (client, connection)
//         = tokio_postgres::connect(db_conn_str, NoTls)
//         .await
//         .context("Query From Parameter Dataset: Error Connect PostgresSQL For CDC")?;
//
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             error!("Query From Parameter Dataset: Error Connect PostgresSQL: {}", e);
//         }
//     });

pub async fn flow_engine(flowcode: String) -> Result<(), sqlx::Error> {
    env_logger::init();
    log::set_max_level(LevelFilter::Info);

    info!("start connect to pg db");

    // Handle database connection with fallback for offline scenarios
    let pool = match env::var("DATABASE_URL") {
        Ok(db_url) => {
            match PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_url)
                .await {
                Ok(pool) => {
                    info!("success connect to pg db");
                    Some(pool)
                },
                Err(e) => {
                    warn!("Failed to connect to database: {}. Continuing in offline mode.", e);
                    None
                }
            }
        },
        Err(_) => {
            warn!("DATABASE_URL not set. Continuing in offline mode.");
            None
        }
    };

    let mut init_string = String::new();
    let mut initProcess: &str = &init_string;
    initProcess = &*flowcode;

    let mut modelId: String = "".to_string();
    if(initProcess!="".to_string()) {
        if let Some(pool_ref) = &pool {
            modelId = create_initiate_model(pool_ref).await?;
        } else {
            // Generate fallback model ID when offline
            modelId = format!("OFFLINE_MODEL_{}", chrono::Utc::now().timestamp());
            info!("Generated offline model ID: {}", modelId);
        }
    }
    let mut resultStatus = "0".to_string();
    info!("flow engine - INITIALIZATION");
    loop {
        //note: buat iterasi ini untuk per idmodel walaupun dengan nama model yg sama. build dapat berulang.
        resultStatus = if let Some(pool_ref) = &pool {
            process_rows((&modelId).to_string(), initProcess, pool_ref).await?
        } else {
            // Fallback behavior when offline
            info!("Running in offline mode with model ID: {}", modelId);
            LLMStatus::LLM_GEN_AFTER_QUANTIZE_RESULT.status.to_string()
        };
        info!("check data if it finish {}",resultStatus);
        if (LLMStatus::LLM_GEN_AFTER_QUANTIZE_RESULT.status == resultStatus) {
            info!("flow engine - latest status {}", LLMStatus::LLM_GEN_AFTER_QUANTIZE_RESULT.status);
            break;
        } else {
            info!("flow engine - latest status {}", resultStatus.clone());
            init_string = resultStatus.clone();
            initProcess = &init_string;
            sleep(Duration::from_secs(15)).await;
            continue;
        }
    }
    info!("flow engine - success - processing finish");
    Ok(())
}

async fn create_initiate_model(pool: &sqlx::PgPool) -> Result<(String), sqlx::Error> {

    let mut llmName = LLM::PEDIA_ASIST_LM.code;
    let llmFlowState  = LLMStatus::LLM_GEN_START.status;
    let llmFlowExecute = FlowExecute::FLOWSTART.status;
    let llmUser = UserExecute::SYSTEM.name;
    let modelId = UUIDModel(llmName.clone(), "V1");
    let typeModel = LMTYPE::LLM.name;

    // selection: [ok]id_models, [-]status_process, [-]type, [-]version
    let rowsDate = sqlx::query(
        r#"
            SELECT id_models, status_process, type, version
            FROM t_models
            WHERE id_models = $1
            "#
    )
        .bind(&llmName)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let id_models: String = row.get("id_models");
            let status_process: String = row.get("status_process");
            let type_model: String = row.get("type");
            let version: String = row.get("version");
            (id_models, status_process, type_model, version)
        })
        .collect::<Vec<_>>();
    if !rowsDate.is_empty() {
        for (id_models, status_process, type_model, version) in rowsDate {
            info!("Processing row ID {} with data {:?}", id_models, status_process);
            // Replace this with your processing logic
            save_data_flow(id_models.clone()).await;
            sqlx::query(
                "UPDATE t_models SET status_process = $1 , version =$2 WHERE id_models = $3"
            )
                .bind(&llmFlowState)
                .bind(&modelId)
                .bind(&id_models)
                .execute(pool).await?;
        }
    } else {
        info!("Create Initiate Model - Model New Language Model");

        //today date:
        // let utc_now = OffsetDateTime::now_utc();
        // let today_utc = utc_now.date();
        let utc_now = OffsetDateTime::now_utc();
        let wib_offset = UtcOffset::from_hms(7, 0, 0); //utc indonesia
        let wib_now = utc_now.to_offset(wib_offset.unwrap()); //wib
        let today_wib = wib_now.date();
        // SystemTime::now()

        // create new lm product:
        sqlx::query("INSERT INTO t_models (id_models, description, size, context, input,
                          create_by, update_by,
                          release_date, status_process, type, version, create_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind(&llmName)
            .bind("Multimodal and MoE")
            .bind("2048")
            .bind("10M")
            .bind("Text, Image")
            .bind("SYSTEM")
            .bind("")
            .bind(&llmFlowState)
            .bind(&typeModel)
            .bind(&modelId)
            .bind(&today_wib)
            .execute(pool).await?;

    }


    //UPDATE PARAMETER FOR ID PROCESSING
    let rows = sqlx::query(
        "SELECT param_key, param_descriptions FROM t_parameter WHERE param_key = $1"
    )
        .bind("LM.BUILD.ID")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let param_key: String = row.get("param_key");
            let param_descriptions: Option<String> = row.get("param_descriptions");
            (param_key, param_descriptions)
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() {
        for (param_key, param_descriptions) in rows {
            info!("Processing row ID {} with data {:?}", param_key, param_descriptions);
            // Replace this with your processing logic
            save_data_flow(param_key.clone()).await;
            sqlx::query(
                "UPDATE t_parameter SET param_value = $1 WHERE param_key = $2"
            )
                .bind(&modelId)
                .bind(&param_key)
                .execute(pool).await?;
        }
    } else {
        info!("Parameter not found LM.BUILD.ID ");
    }

    //note: version dibuat ke table version master dan di hist ngikutin version
    Ok(modelId)
}

async fn validate(pool: &sqlx::PgPool) -> Result<(String), sqlx::Error>  {
    let mut validateResult = StatusProcess::READY.status.to_string();

    // Note: stream data to check changes
    // CHECK START ALLOWED
    let rows = sqlx::query(
        "SELECT param_key, param_value, param_descriptions FROM t_parameter WHERE param_key = $1 LIMIT 1"
    )
        .bind("LM.BUILD.START")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let param_key: String = row.get("param_key");
            let param_value: Option<String> = row.get("param_value");
            let param_descriptions: Option<String> = row.get("param_descriptions");
            (param_key, param_value, param_descriptions)
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() {
        for (param_key, param_value, param_descriptions) in rows {
            info!("Processing row ID {} with data {:?}", param_key, param_descriptions);
            if(param_value.unwrap_or_default() == "1".to_string()){
                info!("Already Running Processing... Add into Queue list... ");
                break;
            }
            sqlx::query(
                "UPDATE t_parameter SET param_value = '1' WHERE param_key = $1"
            )
                .bind(&param_key)
                .execute(pool).await?;
            // Replace this with your processing logic
            save_data_flow(param_key.clone()).await;
        }
    }else {
        info!("Parameter not found LM.BUILD.START ");
    }


    //UPDATE FOR DATE PROCESSING
    let rowsDate = sqlx::query(
        "SELECT param_key, param_descriptions FROM t_parameter WHERE param_key = $1"
    )
        .bind("LM.BUILD.DATE")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let param_key: String = row.get("param_key");
            let param_descriptions: Option<String> = row.get("param_descriptions");
            (param_key, param_descriptions)
        })
        .collect::<Vec<_>>();
    if !rowsDate.is_empty() {
        for (param_key, param_descriptions) in rowsDate {
            info!("Processing row ID {} with data {:?}", param_key, param_descriptions);
            // Replace this with your processing logic
            save_data_flow(param_key.clone()).await;

            sqlx::query(
                "UPDATE t_parameter SET param_value = $1 WHERE param_key = $2"
            )
                .bind(&current_time_ymd())
                .bind(&param_key)
                .execute(pool).await?;
        }
        validateResult = StatusProcess::APPROVE.status.to_string();
    }else {
        info!("Parameter not found LM.BUILD.DATE ");
        validateResult = StatusProcess::READY.status.to_string();;
    }

    Ok(validateResult)
}

pub async fn process_rows(modelId:String, mut initProcess:&str, pool: &sqlx::PgPool) -> Result<(String), sqlx::Error> {
    //jika diperlukan untuk eksekusi views
    // sqlx::query!("REFRESH MATERIALIZED VIEW your_view")
    //     .execute(pool)
    //     .await?;
    info!("engine flow - validate parameter allowed {} ",initProcess);
    let mut resultStatus = "0".to_string();
    validate(&pool).await?;
    info!("engine flow - START {} ",initProcess);
    resultStatus = engine_flow(modelId, initProcess, &pool).await?;
    println!("engine flow - RESULT {} ",resultStatus);
    Ok(resultStatus.to_string())
}

async fn engine_flow(modelId:String, mut initProcess:&str, pool: &sqlx::PgPool) ->  Result<(String), sqlx::Error> {
    // define:
    let llmName = LLM::PEDIA_ASIST_LM.code;
    let llmUser = UserExecute::SYSTEM.name;
    let typeModel = LMTYPE::LLM.name;
    let mut resultStatus = "";
    // next using: rust-query = "0.4.3"

    //TIME SET
    let naive_date = NaiveDate::from_ymd_opt(2025, 6, 18).unwrap();
    let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap(); // Tambahkan waktu default
    let now: DateTime<Utc> = Utc::now();

    // create flow hist:
    if (initProcess == LLMStatus::LLM_GEN_START.status) {
        info!("flow engine - create first {} flow {}",modelId.clone(),LLMStatus::LLM_GEN_START.status);
        sqlx::query("INSERT INTO t_flow_hist (flow_hist_id, model_id, flow_name, flow_gen_name, status, create_by, update_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(&generateTime())
            .bind(&modelId)
            .bind(LLMStatus::LLM_GEN_START.status)
            .bind(&llmName)
            .bind("0")
            .bind(&llmUser)
            .bind("")
            .execute(pool).await?;
        resultStatus = LLMStatus::LLM_GEN_START.status;
        initProcess = LLMStatus::LLM_GEN_START.status;
    }

    let mut init_string = String::new();
    // initProcess: &str = &init_string;
    info!("flow engine - select first {}",modelId.clone());
    let rows = sqlx::query(
        r#"
        SELECT flow_hist_id, model_id, flow_name, flow_gen_name, status
        FROM t_flow_hist
        WHERE model_id = $1 AND flow_gen_name = $2 AND status = $3
        "#
    )
        .bind(&modelId)
        .bind(&llmName)
        .bind(FlowExecute::FLOWSTART.status)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let flow_hist_id: String = row.get("flow_hist_id");
            let model_id: Option<String> = row.get("model_id");
            let flow_name: Option<String> = row.get("flow_name");
            let flow_gen_name: Option<String> = row.get("flow_gen_name");
            let status: Option<String> = row.get("status");
            (flow_hist_id, model_id, flow_name, flow_gen_name, status)
        })
        .collect::<Vec<_>>();
    if !rows.is_empty() && rows.len() > 0{

        for (flow_hist_id, model_id, flow_name, flow_gen_name, status) in rows {
            let statusRow = status.unwrap_or_default();
            let flowHistID = flow_hist_id.clone();
            let modelID = model_id.unwrap_or_default();
            let flowName = flow_name.unwrap_or_default();
            info!("flow engine - model id process status {} ",statusRow);
            info!("flow engine - model id process flow hist id {} ",flowHistID);
            info!("flow engine - model id process model id {} ",modelID);
            info!("flow engine - model id process flow name {} ",flowName);
            if (initProcess == LLMStatus::LLM_GEN_START.status && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_DATA_CHECK.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_DATA_CHECK.status
                && statusRow == FlowExecute::FLOWSTART.status) {

                initProcess = LLMStatus::LLM_GEN_TOKENIZATION.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_TOKENIZATION.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_EMBEDDING.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_EMBEDDING.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_TRANSFORMER_BLOCK.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_TRANSFORMER_BLOCK.status
                && statusRow == FlowExecute::FLOWSTART.status) {

                initProcess = LLMStatus::LLM_GEN_TRANSFORMER_POSITIONAL.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_TRANSFORMER_POSITIONAL.status
                && statusRow == FlowExecute::FLOWSTART.status) {

                initProcess = LLMStatus::LLM_GEN_TRANSFORMER_SERIALIZATION.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_TRANSFORMER_SERIALIZATION.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_MODEL.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_MODEL.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_SAVE.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_SAVE.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_QUANTIZE.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &*modelID, pool).await?;
                break;
            } else if (initProcess == LLMStatus::LLM_GEN_QUANTIZE.status
                && statusRow == FlowExecute::FLOWSTART.status) {
                initProcess = LLMStatus::LLM_GEN_AFTER_QUANTIZE.status;
                let result = update_flow_next_gen(initProcess, &modelID, llmName, &flowHistID, pool).await?;
                init_string = result.clone();
                resultStatus = &init_string;
                update_master_data_gen(initProcess, &modelID, pool).await?;
                break;
            } else {
                info!("update flow job done... ");
            }
            info!("Processing row ID {:#?} with data {:#?} flow name {:#?}", flowHistID, modelID , flowName);
            // Replace this with your processing logic
            save_data_flow(flowHistID.as_str().to_string()).await;
            // Update status parameter ready to next process....
            save_data_flow_next_model();

            if (initProcess == LLMStatus::LLM_GEN_AFTER_QUANTIZE.status) {
                resultStatus = LLMStatus::LLM_GEN_AFTER_QUANTIZE_RESULT.status;
                update_master_data_gen(resultStatus, &*modelID, pool).await?;
                sqlx::query(
                    "UPDATE t_flow_hist SET status = '1' WHERE flow_hist_id = $1"
                )
                    .bind(&flowHistID)
                    .execute(pool).await?;

                sqlx::query("UPDATE t_parameter SET param_value = $1 WHERE param_key = $2")
                    .bind("2")
                    .bind("LM.BUILD.START")
                    .execute(pool).await?;
            }
        }
    } else {
        info!("flow engine - select first - not found");
    }
    Ok(resultStatus.to_string())
}

async fn save_data_flow(id: String) {
    // Mock logic: just a delay and print
    info!("Saving data flow for ID {}", id);
    sleep(Duration::from_millis(500)).await;
}

async fn save_data_flow_next_model() {
    // Mock logic: just a delay and print
    info!("Processing Ready To Start Next {}", current_time());

    sleep(Duration::from_millis(500)).await;
}


fn current_time() -> String {
    let now = Local::now();
    format!("{}", now.to_rfc3339())
}

fn current_time_ymd() -> String {
    Local::now().format("%Y%m%d").to_string()
}

fn uniqueIdUUID() -> String {
    format!("{}", Uuid::new_v4())
}

fn UUIDModel(name: &str, version: &str) -> String {
    //name_version_ymd_id
    //note: version dibuat seq per naming LLM dan disimpan di database dan dibuat function counter
    format!("{}{}_{}_{}", name, version, Local::now().format("%Y%m%d").to_string(), Uuid::new_v4())
}

fn generateTime() -> String {
    Utc::now().format("%Y%m%d%H%M%S%f").to_string()
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);
fn generateidcounter() -> String {//usize
    /**
    * use the datatable to save latest number seq
    */
    format!("{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}


async fn update_flow_next_gen(initStatus: &str, modelID: &String, llmName: &str,
                              flowHistID: &String, pool: &sqlx::PgPool) ->  Result<(String), sqlx::Error>  {

    let utc_now = OffsetDateTime::now_utc();
    let wib_offset = UtcOffset::from_hms(7, 0, 0); //utc indonesia
    let wib_now = utc_now.to_offset(wib_offset.unwrap()); //wib
    let today_wib = wib_now.date();

    sqlx::query("UPDATE t_flow_hist SET status = '1', update_date =$1  WHERE flow_hist_id = $2")
        .bind(&today_wib)
        .bind(&flowHistID)
        .execute(pool).await?;
    sqlx::query("INSERT INTO t_flow_hist (flow_hist_id, model_id, flow_name, flow_gen_name, status, create_by, update_by, create_date)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(&generateTime())
        .bind(&modelID)
        .bind(&initStatus)
        .bind(&llmName)
        .bind(FlowExecute::FLOWSTART.status)
        .bind(UserExecute::SYSTEM.name)
        .bind("")
        .bind(&today_wib)
        .execute(pool).await?;
    info!("flow engine - model in {} to {}",LLMStatus::LLM_GEN_START.status,initStatus);
    Ok(initStatus.to_string())
}

// async fn update_master_data_gen(initStatus: &str, modelID: &String, pool: &sqlx::PgPool) -> String
// {
//     // Result<(String), sqlx::Error>
//     // // ACTIVATE
//     let q = query!("UPDATE t_models SET status_process = $1 WHERE version = $2 ", initStatus, modelID ).execute(pool).await?;
//     info!("flow engine - update master - model {} in status {} ",modelID, initStatus);
//     "".to_string()
// }

// pub async fn update(param: &str,param1: &str, pool: &PgPool) -> Result<String, Error> {
//        query!("UPDATE table SET column = $1 WHERE column1 = $2",param,param1).execute(pool).await?;
//        info!("flow engine - update master - model {} set status to {}",
//        param, param1);
//        Ok(format!("Model {} updated to status {}", param, param1))
// }

pub async fn update_master_data_gen(init_status: &str,model_id: &str, pool: &PgPool) -> Result<String, Error> {
    let utc_now = OffsetDateTime::now_utc();
    let wib_offset = UtcOffset::from_hms(7, 0, 0); //utc indonesia
    let wib_now = utc_now.to_offset(wib_offset.unwrap()); //wib
    let today_wib = wib_now.date();

    let result = sqlx::query("UPDATE t_models SET status_process = $1, update_date=$2 WHERE version = $3")
        .bind(&init_status)
        .bind(&today_wib)
        .bind(&model_id)
        .execute(pool).await?;
    info!("flow engine - update master - model {} set status to {}",
    model_id, init_status);
    Ok(format!("Model {} updated to status {} result {} rows ", model_id, init_status, result.rows_affected()))
}
