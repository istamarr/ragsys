//Move code file ini ke handler/embedding

use actix_web::http::StatusCode;
use log::{debug, error, info, warn};
use warp::Reply;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::domain::models::llm::{CmdBody, Detection, GGUFFile, LLMCreateBuildRequest, LLMRequest, RequestBody};
use crate::domain::models::login::LoginRequest;
use crate::shared::helperUtils::{current_time, srv_response, srv_response_json};
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM};
use crate::WebResult;
use std::{env, fs};
use std::path::PathBuf;
use std::error::Error;
use qdrant_client::Qdrant;
use crate::rag_chaining::base_chain;
use crate::rag_chaining::{cag_pipeline, agentic_cag_pipeline, get_cag_stats};
use crate::rag_agent::embedding::quickEmbedding;
use crate::rag_agent::handler::r#impl::load_training_data_impl::load_n_training_data;
use crate::rag_agent::workflow::{run_rag_orchestrator, RagOrchestrator, OrchestratorConfig};

// ============================================================================
// CAPABILITY MANAGEMENT STRUCTURES
// ============================================================================

/// System capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    pub version: String,
    pub capabilities: Vec<CapabilityInfo>,
    pub active_services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub code: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service: String,
    pub status: String,
    pub url: Option<String>,
}

/// Management command request
#[derive(Debug, Clone, Deserialize)]
pub struct ManagementRequest {
    pub action: String,
    pub target: String,
    pub params: Option<serde_json::Value>,
}

/// Management response
#[derive(Debug, Clone, Serialize)]
pub struct ManagementResponse {
    pub status: String,
    pub action: String,
    pub result: serde_json::Value,
    pub timestamp: String,
}

pub async fn quickThink(uid : String, body : CmdBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    println!("rag_pipeline # quick_think # Start Quick Think");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    // if body.model == LLM::DEEP_SEEK_R1.code {  nameModel= LLM::DEEP_SEEK_R1.code; versionModel = LLM::DEEP_SEEK_R1.version; }
    // else if body.model == LLM::PA_LM.code {  nameModel= LLM::PA_LM.code; versionModel = LLM::PA_LM.version; }
    // else if body.model == LLM::NOMIC_EMBED_TEXT.code {  nameModel= LLM::NOMIC_EMBED_TEXT.code; versionModel = LLM::NOMIC_EMBED_TEXT.version; }
    // else { nameModel= LLM::PA_LM.code; versionModel = LLM::PA_LM.version; }

    let model_version = format!("{}:{}",LLM::PEDIA_ASIST_LM.code,LLM::PEDIA_ASIST_LM.version);

    if body.model == model_version {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_ASIST_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check quickThink - embedd_handler"); }

    // let mut result = quickEmbedding::run_quick_embedding(body.clone());
    println!("rag_pipeline # quick_think # Query - Waiting for Response ... ");
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    // let config = QdrantClient::from(&qdrant_url);
    println!("rag_pipeline # quick_think # QdrantClient config");
    let qdrant_client = Qdrant::from_url(&*qdrant_url).build().unwrap();

    //option/cmd: [analytics][score]-->[inference][insight] && [score][rating]
    //tags call agent: [voice][image][animate][document][flowcode][fixco]
    //complete validation of combination

    let mut response = Ok("".to_string());
    let has_flowcode_tags = body.tags.contains("flowcode") || body.tags.contains("fixco");
    let has_cag_tags = body.tags.contains("cag") || body.tags.contains("cache");
    let has_agentic_tags = body.tags.contains("agentic") || body.tags.contains("orchestrator");

    if has_flowcode_tags {
        // FlowCode/FixCo: Code analysis pipeline
        let query = format!("{:?}", body.prompt);
        response = unsafe { Err(base_chain::run(query).await
            .expect("quickThink: base_chain::run("))
            .expect("quickThink: response") };
    } else if has_agentic_tags {
        // Agentic RAG: Full orchestrator with feedback loop
        println!("rag_pipeline # quick_think # Using Agentic RAG Orchestrator");
        let query = format!("{} - {}", body.options, body.prompt);
        response = run_rag_orchestrator(&query).await
            .map_err(|e| e.to_string());
    } else if has_cag_tags {
        // CAG: Cache-Augmented Generation (fast path with cache)
        println!("rag_pipeline # quick_think # Using CAG Pipeline");
        response = cag_pipeline(&qdrant_client, body.clone()).await
            .map_err(|e| e.to_string());
    } else {
        // Default: Standard RAG pipeline with feedback
        let query = format!("{} - {}", body.options, body.prompt);
        response = unsafe { base_chain::rag_pipeline(&qdrant_client, &*query).await };
    }
    println!("rag_pipeline # quick_think # get response");
    println!("\nFinal Response:\n{:?}", response);

    println!("{} {}", format!("rag_pipeline # quick_think # using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
                 StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("rag_pipeline # quick_think # Success Result");
    let mut result = "";
    //Text
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

pub async fn agentVoice(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    println!("Start Quick Think");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    if body.model == LLM::PEDIA_ASIST_LM.code {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_AIS_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check quickThinkVoice - embedd_handler"); }

    //note: resp in multipart byte code and stream from ui
    // let mut result = quickEmbedding::run_quick_embedding(body.clone());

    println!("{} {}", format!("Quick Think using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("Success Result Quick Think");

    let mut result = "";
    //Ubah ke multipart
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}


/** acccess ke difuser
** agentAnimation
** agentDocuments
** agentImg
***/
pub async fn agentAnimation(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    println!("Start Quick Think");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    if body.model == LLM::PEDIA_ASIST_LM.code {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_AIS_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check quickThinkAnimation - embedd_handler"); }

    //note: resp in multipart byte code and stream from ui
    let mut result = quickEmbedding::run_quick_embedding(body.clone());

    println!("{} {}", format!("Quick Think using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("Success Result Quick Think");

    let mut result = "";
    //ubah ke multipart
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

//acccess ke difuser - Document Agent
pub async fn agentDocuments(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    println!("agentDocuments # Start - uid: {}", uid);

    let mut nameModel = "";
    let mut versionModel = "";
    if body.model == LLM::PEDIA_ASIST_LM.code {
        nameModel= LLM::PEDIA_ASIST_LM.code;
        versionModel = LLM::PEDIA_AIS_LM.version;
    } else {
        error!("Model Defined Isnt Available, Please Check agentDocuments - embedd_handler");
    }

    // Call difsr agent_doc binary
    let doc_type = body.options.clone();
    let output_path = format!("./difsr_printed/doc_{}_{}.txt", uid, chrono::Local::now().format("%Y%m%d%H%M%S"));

    let difsr_result = call_difsr_doc(&doc_type, &body.prompt, &output_path).await;

    println!("{} {}", format!("agentDocuments # using difsr : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("agentDocuments # Success");

    let response = serde_json::json!({
        "status": "success",
        "agent": "document",
        "uid": uid,
        "output_path": output_path,
        "result": difsr_result,
        "timestamp": current_time()
    });

    srv_response(response.to_string(), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

//acccess ke difuser - Image Agent
pub async fn agentImg(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    println!("agentImg # Start - uid: {}", uid);

    let mut nameModel = "";
    let mut versionModel = "";
    if body.model == LLM::PEDIA_ASIST_LM.code {
        nameModel= LLM::PEDIA_ASIST_LM.code;
        versionModel = LLM::PEDIA_AIS_LM.version;
    } else {
        error!("Model Defined Isnt Available, Please Check agentImg - embedd_handler");
    }

    // Parse options for image generation
    let img_type = if body.options.contains("logo") { "logo" }
                   else if body.options.contains("art") { "art" }
                   else { "generate" };
    let color_scheme = extract_color_scheme(&body.options);
    let output_path = format!("./difsr_printed/img_{}_{}.png", uid, chrono::Local::now().format("%Y%m%d%H%M%S"));

    let difsr_result = call_difsr_img(img_type, &body.prompt, &color_scheme, &output_path).await;

    println!("{} {}", format!("agentImg # using difsr : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("agentImg # Success");

    let response = serde_json::json!({
        "status": "success",
        "agent": "image",
        "uid": uid,
        "output_path": output_path,
        "result": difsr_result,
        "timestamp": current_time()
    });

    srv_response(response.to_string(), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

// Helper: Call difsr document agent (HTTP mode with subprocess fallback)
async fn call_difsr_doc(doc_type: &str, input: &str, output: &str) -> String {
    // Ensure output directory exists
    std::fs::create_dir_all("./difsr_printed").ok();

    let cmd_type = match doc_type {
        t if t.contains("pdf") => "pdf",
        t if t.contains("notes") => "notes",
        _ => "generate",
    };

    // Try HTTP mode first if DIFSR_URL is set
    if let Ok(difsr_url) = std::env::var("DIFSR_URL") {
        println!("call_difsr_doc # Using HTTP mode: {}", difsr_url);
        match call_difsr_doc_http(&difsr_url, cmd_type, input, output).await {
            Ok(result) => return result,
            Err(e) => {
                println!("call_difsr_doc # HTTP failed, fallback to subprocess: {}", e);
            }
        }
    }

    // Fallback to subprocess mode
    println!("call_difsr_doc # Using subprocess mode");
    call_difsr_doc_subprocess(cmd_type, input, output)
}

// HTTP mode for document agent
async fn call_difsr_doc_http(base_url: &str, doc_type: &str, input: &str, output: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/doc", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "doc_type": doc_type,
        "input": input,
        "output": output
    });

    let response = client.post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if json["status"] == "success" {
        Ok(format!("Document generated: {}", json["output_path"].as_str().unwrap_or("")))
    } else {
        Err(json["message"].as_str().unwrap_or("Unknown error").to_string())
    }
}

// Subprocess mode for document agent
fn call_difsr_doc_subprocess(cmd_type: &str, input: &str, output: &str) -> String {
    use std::process::Command;

    let result = Command::new("cargo")
        .args(&["run", "--bin", "agent_doc", "--manifest-path", "../difsr/Cargo.toml", "--", cmd_type, input, output])
        .output();

    match result {
        Ok(output_result) => {
            let stdout = String::from_utf8_lossy(&output_result.stdout);
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            if output_result.status.success() {
                format!("Document generated: {}", stdout)
            } else {
                format!("Error: {}", stderr)
            }
        },
        Err(e) => format!("Failed to execute difsr: {}", e),
    }
}

// Helper: Call difsr image agent (HTTP mode with subprocess fallback)
async fn call_difsr_img(img_type: &str, prompt: &str, color: &str, output: &str) -> String {
    // Ensure output directory exists
    std::fs::create_dir_all("./difsr_printed").ok();

    let gen_type = match prompt.to_lowercase().as_str() {
        p if p.contains("plasma") => "plasma",
        p if p.contains("diffusion") => "diffusion",
        p if p.contains("fractal") || p.contains("mandelbrot") => "mandelbrot",
        p if p.contains("waves") => "waves",
        p if p.contains("geometric") => "geometric",
        p if p.contains("circular") || p.contains("circle") => "circular",
        p if p.contains("polygon") => "polygon",
        p if p.contains("grid") => "grid",
        _ => "diffusion",
    };

    // Try HTTP mode first if DIFSR_URL is set
    if let Ok(difsr_url) = std::env::var("DIFSR_URL") {
        println!("call_difsr_img # Using HTTP mode: {}", difsr_url);
        match call_difsr_img_http(&difsr_url, img_type, &gen_type, color, output).await {
            Ok(result) => return result,
            Err(e) => {
                println!("call_difsr_img # HTTP failed, fallback to subprocess: {}", e);
            }
        }
    }

    // Fallback to subprocess mode
    println!("call_difsr_img # Using subprocess mode");
    call_difsr_img_subprocess(img_type, &gen_type, color, output)
}

// HTTP mode for image agent
async fn call_difsr_img_http(base_url: &str, img_type: &str, gen_type: &str, color: &str, output: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/img", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "img_type": img_type,
        "gen_type": gen_type,
        "color": color,
        "output": output
    });

    let response = client.post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if json["status"] == "success" {
        Ok(format!("Image generated: {}", json["output_path"].as_str().unwrap_or("")))
    } else {
        Err(json["message"].as_str().unwrap_or("Unknown error").to_string())
    }
}

// Subprocess mode for image agent
fn call_difsr_img_subprocess(img_type: &str, gen_type: &str, color: &str, output: &str) -> String {
    use std::process::Command;

    let result = Command::new("cargo")
        .args(&[
            "run", "--bin", "agent_img",
            "--manifest-path", "../difsr/Cargo.toml",
            "--", img_type, gen_type,
            "--color", color,
            "--output", output
        ])
        .output();

    match result {
        Ok(output_result) => {
            let stdout = String::from_utf8_lossy(&output_result.stdout);
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            if output_result.status.success() {
                format!("Image generated: {}", stdout)
            } else {
                format!("Error: {}", stderr)
            }
        },
        Err(e) => format!("Failed to execute difsr: {}", e),
    }
}

// Helper: Extract color scheme from options
fn extract_color_scheme(options: &str) -> String {
    let lower = options.to_lowercase();
    if lower.contains("rainbow") { "rainbow".to_string() }
    else if lower.contains("ocean") { "ocean".to_string() }
    else if lower.contains("fire") { "fire".to_string() }
    else if lower.contains("forest") { "forest".to_string() }
    else if lower.contains("sunset") { "sunset".to_string() }
    else if lower.contains("electric") { "electric".to_string() }
    else if lower.contains("pastel") { "pastel".to_string() }
    else if lower.contains("gray") || lower.contains("grayscale") { "grayscale".to_string() }
    else { "rainbow".to_string() }
}

/**
note:
 * img(encode base64) - ok,
 * voice, stream byte &/ stream repo (minio)
 * vid/anim, stream byte &/ stream repo (minio)
 * text - ok
 */

pub async fn deepThink(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    println!("Start Quick Think");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    if body.model == LLM::PEDIA_ASIST_LM.code {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_AIS_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check deepThink - embedd_handler"); }

    //note: resp in multipart byte code and stream from ui
    let mut result = quickEmbedding::run_quick_embedding(body.clone());

    println!("{} {}", format!("Quick Think using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("Success Result Quick Think");

    let mut result = "";
    //ubah ke multipart
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}


/**
* Blockchain Embed
* NFT Embed
**/
pub async fn agentNFT(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    println!("Start Deep Think For Decentralized (Blockchain)");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    if body.model == LLM::PEDIA_ASIST_LM.code {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_AIS_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check deepBlockChainThink - embedd_handler"); }

    //note: resp in multipart byte code and stream from ui
    let mut result = quickEmbedding::run_quick_embedding(body.clone());

    println!("{} {}", format!("Quick Think using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("Success Result Deep Think");

    let mut result = "";
    //ubah ke multipart
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

pub async fn agentWeb3(uid : String, body : RequestBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    // check req if it NFT's
    println!("Start Quick Think For Decentralized (Blockchain)");

    let mut nameModel = "";
    let mut versionModel = "";
    // LLM::PEDIA_AIS_LM.code
    if body.model == LLM::PEDIA_ASIST_LM.code {  nameModel= LLM::PEDIA_ASIST_LM.code; versionModel =LLM::PEDIA_AIS_LM.version; }
    else { error!("Model Defined Isnt Available, Please Check quickBlockChainThink - embedd_handler"); }

    //note: resp in multipart byte code and stream from ui
    let mut result = quickEmbedding::run_quick_embedding(body.clone());

    println!("{} {}", format!("Quick Think using flex embedding : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    println!("Success Result Quick Think");

    let mut result = "";
    //ubah ke multipart
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

// ============================================================================
// FUNCTIONALITY MANAGEMENT APIs
// ============================================================================

/// Get system capabilities - lists all available functions
pub async fn getCapabilities(uid: String) -> WebResult<impl Reply> {
    println!("getCapabilities # Start - uid: {}", uid);

    let capabilities = vec![
        CapabilityInfo {
            name: "FlowCode Analysis".to_string(),
            code: "flowcode".to_string(),
            description: "Analyze project code structure, generate diagrams, detect issues".to_string(),
            tags: vec!["flowcode".to_string(), "analysis".to_string(), "diagram".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "FixCo Code Fix".to_string(),
            code: "fixco".to_string(),
            description: "Automated code fixing, security scanning, metrics analysis".to_string(),
            tags: vec!["fixco".to_string(), "fix".to_string(), "security".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "RAG Pipeline".to_string(),
            code: "rag".to_string(),
            description: "Retrieval-Augmented Generation for intelligent responses".to_string(),
            tags: vec!["rag".to_string(), "embedding".to_string(), "query".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "NFT Pawn Chain".to_string(),
            code: "nft_pawn".to_string(),
            description: "NFT collateralized lending with IPFS storage".to_string(),
            tags: vec!["nft".to_string(), "pawn".to_string(), "blockchain".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "RFID Collateral Tracking".to_string(),
            code: "rfid".to_string(),
            description: "Physical collateral tracking with RFID - location, condition, delivery".to_string(),
            tags: vec!["rfid".to_string(), "tracking".to_string(), "collateral".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "Voice Agent".to_string(),
            code: "voice".to_string(),
            description: "Text-to-voice synthesis via t2v module".to_string(),
            tags: vec!["voice".to_string(), "t2v".to_string(), "speech".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "Image Agent".to_string(),
            code: "image".to_string(),
            description: "Image processing and generation via difsr module".to_string(),
            tags: vec!["image".to_string(), "difsr".to_string(), "diffuser".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "Animation Agent".to_string(),
            code: "animate".to_string(),
            description: "Animation generation via agentAnimation".to_string(),
            tags: vec!["animate".to_string(), "video".to_string(), "motion".to_string()],
            status: "active".to_string(),
        },
        CapabilityInfo {
            name: "Document Agent".to_string(),
            code: "document".to_string(),
            description: "Document processing via difsr module".to_string(),
            tags: vec!["document".to_string(), "pdf".to_string(), "difsr".to_string()],
            status: "active".to_string(),
        },
    ];

    // Check service status
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let burn_lm_url = env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:9393".to_string());
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ipfs_url = env::var("IPFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());

    let active_services = vec![
        ServiceStatus {
            service: "Qdrant Vector DB".to_string(),
            status: "configured".to_string(),
            url: Some(qdrant_url),
        },
        ServiceStatus {
            service: "burn-lm ASIST".to_string(),
            status: "configured".to_string(),
            url: Some(burn_lm_url),
        },
        ServiceStatus {
            service: "Ollama LLM".to_string(),
            status: "configured".to_string(),
            url: Some(ollama_url),
        },
        ServiceStatus {
            service: "IPFS Local".to_string(),
            status: "configured".to_string(),
            url: Some(ipfs_url),
        },
    ];

    let system_caps = SystemCapabilities {
        version: "1.0.0".to_string(),
        capabilities,
        active_services,
    };

    let response = serde_json::to_string_pretty(&system_caps).unwrap_or_default();
    println!("getCapabilities # Success");

    srv_response(response, StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

/// Management endpoint for system functions
pub async fn manageFunction(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    println!("manageFunction # Start - uid: {} cmd: {}", uid, body.cmd);

    let result = match body.cmd.to_lowercase().as_str() {
        // Export management
        "list_exports" => list_export_files(),
        "get_export" => get_export_file(&body.prompt),
        "clean_exports" => clean_old_exports(7),

        // RFID management
        "rfid_list" => list_rfid_records(),
        "rfid_scan" => scan_rfid_tag(&body.prompt),

        // System management
        "health_check" => health_check().await,
        "system_info" => get_system_info(),

        _ => {
            serde_json::json!({
                "error": "Unknown command",
                "available_commands": [
                    "list_exports", "get_export", "clean_exports",
                    "rfid_list", "rfid_scan",
                    "health_check", "system_info"
                ]
            })
        }
    };

    let response = ManagementResponse {
        status: "success".to_string(),
        action: body.cmd.clone(),
        result,
        timestamp: current_time(),
    };

    let json_response = serde_json::to_string_pretty(&response).unwrap_or_default();
    println!("manageFunction # Success - action: {}", body.cmd);

    srv_response(json_response, StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}

// ============================================================================
// MANAGEMENT HELPER FUNCTIONS
// ============================================================================

/// List all export files in rag_output directory
fn list_export_files() -> serde_json::Value {
    let output_dir = PathBuf::from("./rag_output");
    let mut files: Vec<serde_json::Value> = Vec::new();

    if let Ok(entries) = fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy().to_string();
                let extension = path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                files.push(serde_json::json!({
                    "filename": filename_str,
                    "extension": extension,
                    "size_bytes": size,
                    "path": path.to_string_lossy().to_string(),
                    "download_link": format!("file:///{}", path.canonicalize()
                        .unwrap_or(path.clone())
                        .display()
                        .to_string()
                        .replace("\\", "/"))
                }));
            }
        }
    }

    serde_json::json!({
        "directory": output_dir.to_string_lossy().to_string(),
        "total_files": files.len(),
        "files": files
    })
}

/// Get specific export file content
fn get_export_file(filename: &str) -> serde_json::Value {
    let file_path = PathBuf::from("./rag_output").join(filename);

    if file_path.exists() {
        match fs::read_to_string(&file_path) {
            Ok(content) => serde_json::json!({
                "filename": filename,
                "exists": true,
                "content": content,
                "size_bytes": content.len()
            }),
            Err(e) => serde_json::json!({
                "filename": filename,
                "exists": true,
                "error": format!("Cannot read file: {}", e)
            })
        }
    } else {
        serde_json::json!({
            "filename": filename,
            "exists": false,
            "error": "File not found"
        })
    }
}

/// Clean export files older than specified days
fn clean_old_exports(days: i64) -> serde_json::Value {
    let output_dir = PathBuf::from("./rag_output");
    let mut deleted: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    let cutoff = chrono::Local::now() - chrono::Duration::days(days);

    if let Ok(entries) = fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<chrono::Local> = modified.into();
                    if modified_time < cutoff {
                        if let Some(filename) = path.file_name() {
                            match fs::remove_file(&path) {
                                Ok(_) => deleted.push(filename.to_string_lossy().to_string()),
                                Err(e) => errors.push(format!("{}: {}", filename.to_string_lossy(), e)),
                            }
                        }
                    }
                }
            }
        }
    }

    serde_json::json!({
        "action": "clean_exports",
        "days_threshold": days,
        "deleted_count": deleted.len(),
        "deleted_files": deleted,
        "errors": errors
    })
}

/// List RFID tracking records
fn list_rfid_records() -> serde_json::Value {
    let tracking_file = PathBuf::from("./output/rfid_tracking/tracking_index.json");

    if tracking_file.exists() {
        match fs::read_to_string(&tracking_file) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(records) => serde_json::json!({
                        "status": "success",
                        "records": records
                    }),
                    Err(_) => serde_json::json!({
                        "status": "error",
                        "message": "Failed to parse tracking records"
                    })
                }
            },
            Err(e) => serde_json::json!({
                "status": "error",
                "message": format!("Cannot read tracking file: {}", e)
            })
        }
    } else {
        serde_json::json!({
            "status": "empty",
            "message": "No RFID tracking records found",
            "records": []
        })
    }
}

/// Scan specific RFID tag
fn scan_rfid_tag(tag_id: &str) -> serde_json::Value {
    let tracking_file = PathBuf::from("./output/rfid_tracking/tracking_index.json");

    if tracking_file.exists() {
        if let Ok(content) = fs::read_to_string(&tracking_file) {
            if let Ok(records) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                for record in records {
                    if let Some(rfid_tag) = record.get("rfid_tag") {
                        if let Some(id) = rfid_tag.get("tag_id") {
                            if id.as_str() == Some(tag_id) {
                                return serde_json::json!({
                                    "status": "found",
                                    "tag_id": tag_id,
                                    "record": record
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    serde_json::json!({
        "status": "not_found",
        "tag_id": tag_id,
        "message": "RFID tag not found in tracking system"
    })
}

/// Health check for connected services
async fn health_check() -> serde_json::Value {
    let mut services: Vec<serde_json::Value> = Vec::new();

    // Check Qdrant
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_status = check_service_health(&qdrant_url).await;
    services.push(serde_json::json!({
        "service": "Qdrant",
        "url": qdrant_url,
        "status": qdrant_status
    }));

    // Check Ollama
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_status = check_service_health(&ollama_url).await;
    services.push(serde_json::json!({
        "service": "Ollama",
        "url": ollama_url,
        "status": ollama_status
    }));

    // Check burn-lm
    let burn_lm_url = env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:9393".to_string());
    let burn_lm_status = check_service_health(&burn_lm_url).await;
    services.push(serde_json::json!({
        "service": "burn-lm ASIST",
        "url": burn_lm_url,
        "status": burn_lm_status
    }));

    serde_json::json!({
        "timestamp": current_time(),
        "services": services
    })
}

async fn check_service_health(url: &str) -> String {
    match reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                "healthy".to_string()
            } else {
                format!("unhealthy ({})", resp.status())
            }
        },
        Err(_) => "unreachable".to_string(),
    }
}

/// Get CAG cache statistics
pub async fn get_cag_cache_stats(uid: String) -> WebResult<impl Reply> {
    info!("get_cag_cache_stats # Request from uid: {}", uid);

    let stats = get_cag_stats().await;

    let response = serde_json::json!({
        "status": "success",
        "cache_stats": {
            "total_entries": stats.total_entries,
            "high_quality_entries": stats.high_quality_entries,
            "expired_entries": stats.expired_entries
        },
        "timestamp": current_time()
    });

    srv_response_json(response, StatusCode::OK)
}


/// Agentic RAG endpoint - full orchestrator with feedback loop
pub async fn agentic_rag(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("agentic_rag # Starting for uid: {}", uid);

    let query = format!("{} - {}", body.options, body.prompt);

    let result = run_rag_orchestrator(&query).await;

    match result {
        Ok(response) => {
            let json_response = serde_json::json!({
                "status": "success",
                "response": response,
                "pipeline": "agentic_orchestrator",
                "timestamp": current_time()
            });
            srv_response_json(json_response, StatusCode::OK)
        },
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "error": e.to_string(),
                "timestamp": current_time()
            });
            srv_response_json(json_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// CAG Pipeline endpoint - cache-augmented generation
pub async fn cag_rag(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("cag_rag # Starting for uid: {}", uid);

    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    // let config = QdrantClient::from(&qdrant_url);
    let qdrant_client = Qdrant::from_url(&*qdrant_url).build();

    match qdrant_client {
        Ok(client) => {
            let result = cag_pipeline(&client, body).await;

            match result {
                Ok(response) => {
                    let json_response = serde_json::json!({
                        "status": "success",
                        "response": response,
                        "pipeline": "cag",
                        "timestamp": current_time()
                    });
                    srv_response_json(json_response, StatusCode::OK)
                },
                Err(e) => {
                    let json_response = serde_json::json!({
                        "status": "error",
                        "error": e.to_string(),
                        "timestamp": current_time()
                    });
                    srv_response_json(json_response, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "error": format!("Qdrant connection failed: {}", e),
                "timestamp": current_time()
            });
            srv_response_json(json_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get system information
fn get_system_info() -> serde_json::Value {
    let rag_output = PathBuf::from("./rag_output");
    let rfid_output = PathBuf::from("./output/rfid_tracking");
    let struk_output = PathBuf::from("./output/struk");

    let rag_files = fs::read_dir(&rag_output).map(|d| d.count()).unwrap_or(0);
    let rfid_exists = rfid_output.exists();
    let struk_exists = struk_output.exists();

    serde_json::json!({
        "version": "1.0.0",
        "name": "RAG System with NFT Pawn & RFID Tracking",
        "timestamp": current_time(),
        "directories": {
            "rag_output": {
                "path": rag_output.to_string_lossy().to_string(),
                "file_count": rag_files
            },
            "rfid_tracking": {
                "path": rfid_output.to_string_lossy().to_string(),
                "exists": rfid_exists
            },
            "struk_output": {
                "path": struk_output.to_string_lossy().to_string(),
                "exists": struk_exists
            }
        },
        "environment": {
            "QDRANT_URL": env::var("QDRANT_URL").unwrap_or_else(|_| "not set".to_string()),
            "OLLAMA_URL": env::var("OLLAMA_URL").unwrap_or_else(|_| "not set".to_string()),
            "BURN_LM_URL": env::var("BURN_LM_URL").unwrap_or_else(|_| "not set".to_string()),
            "IPFS_API_URL": env::var("IPFS_API_URL").unwrap_or_else(|_| "not set".to_string())
        }
    })
}
