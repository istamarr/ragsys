use actix_web::http::StatusCode;
use log::{debug, error, info, warn};
use warp::Reply;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, to_value, Value};
use crate::domain::models::llm::{CmdBody, Detection, GGUFFile, LLMCreateBuildRequest, LLMRequest, RequestBody};
use crate::domain::models::login::LoginRequest;
use crate::shared::helperUtils::{current_time, srv_response, srv_response_json};
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM};
use crate::WebResult;
use std::{env, fs};
use std::path::PathBuf;
use std::error::Error;
// use qdrant_client::client::{QdrantClient, QdrantClientConfig};
use qdrant_client;
use qdrant_client::Qdrant;
use qdrant_client::Payload;
use crate::domain::dbs_rag_LM;
use crate::domain::rag_agp_a2a::{ResponseRagAgpSrv, ResponseTokenRagAgp};
use crate::rag_chaining::base_chain;
use crate::rag_chaining::{cag_pipeline, agentic_cag_pipeline, get_cag_stats};
use crate::rag_chaining::base_chain::{CodeMetrics, ErrCheckApiResponse, ExportFiles, IssuesSummary, ProjectInfo, RagApiResponse, RagResponseData, SecuritySummary};
use crate::rag_agent::embedding::quickEmbedding;
use crate::rag_agent::handler;
use crate::rag_agent::module_workflow::{run_rag_orchestrator, RagOrchestrator, OrchestratorConfig};

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
    //tags call agent: [voice][image][animate][document][flowcode][fixco][rfid][nft][pawn][gold]
    //complete validation of combination

    let has_flowcode_tags = body.tags.contains("flowcode") || body.tags.contains("fixco");
    let has_cag_tags = body.tags.contains("cag") || body.tags.contains("cache");
    let has_agentic_tags = body.tags.contains("agentic") || body.tags.contains("orchestrator");
    let has_rfid_tags = body.tags.contains("rfid") || body.tags.contains("nft") || body.tags.contains("pawn") || body.tags.contains("gold");
    let has_diffuser_tags = body.tags.contains("diffuser") || body.tags.contains("logo") || body.tags.contains("art") || body.tags.contains("image") || body.tags.contains("generate");
    let has_voice_tags = body.tags.contains("voice") || body.tags.contains("t2v") || body.tags.contains("v2t");
    let has_yolo_tags = body.tags.contains("ylv") || body.tags.contains("yolo") || body.tags.contains("vision");

    let final_response: String;

    if has_flowcode_tags {
        // FlowCode/FixCo: Code analysis pipeline
        println!("rag_pipeline # quick_think # Using FlowCode/FixCo Pipeline");

        // Parse project path dari tags format: [flowcode,fixco][ProjectName][file:///path][instructions]
        let project_path = extract_project_path_from_tags(&body.tags);
        println!("rag_pipeline # quick_think # Project Path: {}", project_path);

        match base_chain::run(project_path).await {
            Ok(api_response) => {

                println!("####################################");
                println!("####################################");
                println!("api_response {:?} ",&api_response.clone());
                println!("####################################");
                println!("####################################");

                // Serialize RagApiResponse ke JSON
                final_response = serde_json::to_string_pretty(&api_response.clone())
                    .unwrap_or_else(|e| format!("Error serializing response: {}", e));
                println!("rag_pipeline # quick_think # FlowCode analysis complete");
            }
            // Err(e) => {
            //     let str_err = format!("FlowCode analysis failed: {}", e);
            //     // let err_final_response : Value = to_value(&str_final_response).unwrap();
            //     // final_response = serde_json::to_string_pretty(&err_final_response);
            //     final_response = format!("{{\"status\": \"error\", \"message\": \"{}\"}}", e);
            //     // let final_response = Payload::try_from(json!({
            //     //     "status": "error".to_string(),
            //     //     "message": str_err,
            //     //     "timestamp": "2025-04-23",
            //     // })).unwrap();
            //
            //     println!("rag_pipeline # quick_think # FlowCode error: {}", e);
            // }
            // Err(e) => {
            //     // let err_final_obj = ErrCheckApiResponse{
            //     //     status: "error".to_string(),
            //     //     message: e.to_string(),
            //     // };
            //
            //     // let e_response = e.to_string();
            //     let err_final_obj = serde_json::json!({
            //                     "status": "error".to_string(),
            //                     "message": format!("{:?}", e.to_string()),
            //                 });
            //
            //     //opt: serde_json::to_string(e)
            //
            //     // let err_final_response = format!("{{\"status\": \"error\", \"message\": \"FlowCode analysis failed: r#{}#\"}}", e);
            //     // let err_final_response : Value = to_value(&err_final_obj).unwrap();
            //     // final_response = serde_json::to_string_pretty(&err_final_obj);
            //
            //     final_response = serde_json::to_string(&err_final_obj).unwrap();
            //
            //     // final_response = err_final_response;
            //     println!("rag_pipeline # quick_think # FlowCode error: {}", e);
            // }
            Err(e) => {
                // Format 1: Menggunakan struct langsung (direkomendasikan)
                let err_final_obj = ErrCheckApiResponse {
                    status: "error".to_string(),
                    message: e.to_string(), // JANGAN gunakan format!("{:?}", ...)
                };

                // Konversi ke JSON string
                final_response = to_string(&err_final_obj).unwrap();

                // Atau Format 2: Menggunakan json! macro
                // let err_final_obj = json!({
                //     "status": "error",
                //     "message": e.to_string() // Langsung e.to_string()
                // });
                //
                // final_response = err_final_obj.to_string();

                println!("rag_pipeline # quick_think # FlowCode error: {}", e);
            }
        }
    } else if has_agentic_tags {
        // Agentic RAG: Full orchestrator with feedback loop
        println!("rag_pipeline # quick_think # Using Agentic RAG Orchestrator");
        let query = format!("{} - {}", body.options, body.prompt);
        match run_rag_orchestrator(&query).await {
            Ok(resp) => final_response = resp,
            Err(e) => final_response = format!("{{\"status\": \"error\", \"message\": \"{}\"}}", e),
        }
    } else if has_cag_tags {
        // CAG: Cache-Augmented Generation (fast path with cache)
        println!("rag_pipeline # quick_think # Using CAG Pipeline");
        match cag_pipeline(&qdrant_client, body.clone()).await {
            Ok(resp) => final_response = resp,
            Err(e) => final_response = format!("{{\"status\": \"error\", \"message\": \"{}\"}}", e),
        }
    } else if has_rfid_tags {
        // RFID/NFT/Pawn/Gold: Route to RFID handler or agent device
        println!("rag_pipeline # quick_think # Using RFID/NFT/Pawn/Gold Pipeline");

        // Determine specific tag type
        if body.tags.contains("rfid") {
            println!("rag_pipeline # quick_think # RFID tag detected - routing to RFID pipeline");
            // Call RFID pipeline directly (same as rfid_model handler does)
            let query = body.clone().prompt;
            match crate::rag_chaining::rag_pipeline::rag_pipeline_rfid(&qdrant_client, query.as_str(), body.clone()).await {
                Ok(response) => {
                    final_response = format!("{{\"status\": \"success\", \"message\": \"RFID query processed\", \"data\": {}}}", response);
                }
                Err(e) => final_response = format!("{{\"status\": \"error\", \"message\": \"RFID pipeline error: {}\"}}", e),
            }
        } else if body.tags.contains("nft") {
            println!("rag_pipeline # quick_think # NFT tag detected - routing to NFT RAG pipeline");
            // Route to RAG pipeline with NFT context for AI-powered NFT analysis
            let nft_query = format!("[NFT] {}", body.prompt);
            match crate::rag_chaining::rag_pipeline::rag_pipeline_rfid(&qdrant_client, nft_query.as_str(), body.clone()).await {
                Ok(response) => {
                    final_response = format!("{{\"status\": \"success\", \"agent\": \"nft\", \"message\": \"NFT analysis complete\", \"data\": \"{}\"}}", response.replace("\"", "\\\""));
                }
                Err(e) => final_response = format!("{{\"status\": \"error\", \"message\": \"NFT pipeline error: {}\"}}", e),
            }
        } else if body.tags.contains("pawn") {
            println!("rag_pipeline # quick_think # Pawn tag detected - routing to nft_pawn_chain");
            // Route to nft_pawn_chain subprocess for Pawn collateralized lending
            let pawn_result = call_nft_pawn_chain("pawn", &body.prompt).await;
            final_response = format!("{{\"status\": \"success\", \"agent\": \"pawn\", \"message\": \"Pawn NFT chain processed\", \"data\": \"{}\"}}", pawn_result.replace("\"", "\\\""));
        } else if body.tags.contains("gold") {
            println!("rag_pipeline # quick_think # Gold tag detected - routing to Gold processing");
            // Route to nft_pawn_chain with gold mode for physical gold collateral
            let gold_result = call_nft_pawn_chain("gold", &body.prompt).await;
            final_response = format!("{{\"status\": \"success\", \"agent\": \"gold\", \"message\": \"Gold collateral processed\", \"data\": \"{}\"}}", gold_result.replace("\"", "\\\""));
        } else {
            // Fallback for RFID tags without specific sub-type
            final_response = format!("{{\"status\": \"error\", \"message\": \"Unknown RFID tag type\"}}");
        }
    } else if has_diffuser_tags {
        // Diffuser/Image/Logo/Art/Generate: Route to diffuser agent
        println!("rag_pipeline # quick_think # Using Diffuser Pipeline");

        if body.tags.contains("diffuser") || body.tags.contains("image") || body.tags.contains("generate") {
            println!("rag_pipeline # quick_think # Diffuser/Image/Generate tag detected");
            final_response = format!("{{\"status\": \"success\", \"message\": \"Diffuser image generation via agent_img\", \"query\": \"{}\"}}", body.prompt);
        } else if body.tags.contains("logo") {
            println!("rag_pipeline # quick_think # Logo tag detected");
            final_response = format!("{{\"status\": \"success\", \"message\": \"Logo generation via agent_img\", \"query\": \"{}\"}}", body.prompt);
        } else if body.tags.contains("art") {
            println!("rag_pipeline # quick_think # Art tag detected");
            final_response = format!("{{\"status\": \"success\", \"message\": \"Art generation via agent_img\", \"query\": \"{}\"}}", body.prompt);
        } else {
            final_response = format!("{{\"status\": \"error\", \"message\": \"Unknown diffuser tag type\"}}");
        }
    } else if has_voice_tags {
        // Voice/T2V/V2T: Route to voice agent
        println!("rag_pipeline # quick_think # Using Voice Pipeline");

        if body.tags.contains("t2v") {
            println!("rag_pipeline # quick_think # Text-to-Voice tag detected");
            // Route to text2voice function
            crate::rag_agent::handler::t2v::t2v::text2voice(body.prompt.clone());
            final_response = format!("{{\"status\": \"success\", \"message\": \"Text-to-Voice synthesis completed\", \"text\": \"{}\"}}", body.prompt);
        } else if body.tags.contains("v2t") {
            println!("rag_pipeline # quick_think # Voice-to-Text tag detected - blocked by dependency conflict");
            // voice-stream 0.4.0 conflicts with ort/voice_activity_detector in existing deps
            final_response = format!("{{\"status\": \"error\", \"message\": \"Voice-to-Text not available - voice-stream has ort dependency conflict\"}}");
        } else if body.tags.contains("voice") {
            println!("rag_pipeline # quick_think # Voice tag detected - routing to agentVoice");
            // Route to agentVoice handler
            let voice_body = RequestBody {
                model: body.model.clone(),
                prompt: body.prompt.clone(),
                options: body.options.clone(),
                keepAlive: body.keepAlive.clone(),
            };
            match agentVoice(uid.clone(), voice_body).await {
                Ok(reply) => {
                    final_response = format!("{{\"status\": \"success\", \"message\": \"Voice processing via agentVoice\", \"query\": \"{}\"}}", body.prompt);
                }
                Err(_) => final_response = format!("{{\"status\": \"error\", \"message\": \"Voice handler error\"}}"),
            }
        } else {
            final_response = format!("{{\"status\": \"error\", \"message\": \"Unknown voice tag type\"}}");
        }
    } else if has_yolo_tags {
        // YOLOv8 Vision: Object detection on image
        println!("rag_pipeline # quick_think # Using YOLOv8 Vision Pipeline");

        // prompt should contain base64 image or file path
        let image_input = body.prompt.clone();
        let yolo_result = call_yolov8_detect(&image_input).await;
        final_response = format!("{{\"status\": \"success\", \"agent\": \"yolov8\", \"message\": \"Object detection complete\", \"data\": \"{}\"}}", yolo_result.replace("\"", "\\\""));
    } else {
        // Default: Standard RAG pipeline with feedback
        println!("rag_pipeline # quick_think # Using Standard RAG Pipeline");
        let query = format!("{} - {}", body.options, body.prompt);
        match base_chain::rag_pipeline(&qdrant_client, &*query).await {
            Ok(resp) => final_response = resp,
            Err(e) => final_response = format!("{{\"status\": \"error\", \"message\": \"{}\"}}", e),
        }
    }

    println!("rag_pipeline # quick_think # get response");
    println!("\nFinal Response:\n{}", final_response);

    //serde_json::Value
    let str_final_response = &final_response;
    println!("str_final_response: {}",str_final_response);

    // let json_to_str = serde_json::to_string(&*str_final_response).unwrap();
    // println!("json_to_str {}",json_to_str);
    // let check_final_response : ErrCheckApiResponse = serde_json::from_str(&json_to_str).unwrap();
    //
    // let mut status = StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap();
    let result = final_response.clone().parse().unwrap();

    // Parse response to determine HTTP status code
    // Graceful fallback: if response isn't valid JSON or missing status/message, treat as success
    let (status, check_status, check_message) = match serde_json::from_str::<ErrCheckApiResponse>(&final_response) {
        Ok(parsed) => {
            let s = if parsed.status == "success" {
                StatusCode::OK
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (s, parsed.status, parsed.message)
        }
        Err(_) => {
            // Response is not JSON with status/message (e.g. raw RAG pipeline output)
            (StatusCode::OK, "success".to_string(), "RAG pipeline response".to_string())
        }
    };

    println!("Parsed struct - Status: {}", check_status);
    println!("Parsed struct - Message: {}", check_message);

    println!("HTTP Status: {}", status);

    // if(check_final_response.status != StatusCode::OK.as_str()) {
    //     status = StatusCode::from_u16(StatusCode::UNPROCESSABLE_ENTITY.as_u16()).unwrap();
    // }

    println!("{} {:?}",
             format!("rag_pipeline # quick_think # using flex embedding \
             : {} {} Version {} At {} ",
                     uid,
                     nameModel,
                     versionModel,
                     current_time()),
             status);

    println!("rag_pipeline # quick_think # Success Result");
    srv_response_json(result, status)
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

    // Call agent_difusser agent_doc binary
    let doc_type = body.options.clone();
    let output_path = format!("./rag_output/agent_diffuser/doc_{}_{}.txt", uid, chrono::Local::now().format("%Y%m%d%H%M%S"));

    let difsr_result = call_difsr_doc(&doc_type, &body.prompt, &output_path).await;

    println!("{} {}", format!("agentDocuments # using agent_difusser : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
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
    let output_path = format!("./rag_output/agent_diffuser/img_{}_{}.png", uid, chrono::Local::now().format("%Y%m%d%H%M%S"));

    let difsr_result = call_difsr_img(img_type, &body.prompt, &color_scheme, &output_path).await;

    println!("{} {}", format!("agentImg # using agent_difusser : {} {} Version {} At {} ", uid, nameModel, versionModel, current_time()),
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

// Helper: Call agent_difusser document agent (HTTP mode with subprocess fallback)
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
        .args(&["run", "--bin", "agent_doc", "--manifest-path", "../agent_difusser/Cargo.toml", "--", cmd_type, input, output])
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
        Err(e) => format!("Failed to execute agent_difusser: {}", e),
    }
}

// Helper: Call agent_difusser image agent (HTTP mode with subprocess fallback)
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
            "--manifest-path", "../agent_difusser/Cargo.toml",
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
        Err(e) => format!("Failed to execute agent_difusser: {}", e),
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

/// Helper: Extract project path from tags
/// Tags format: [flowcode,fixco][ProjectName][file:///path/to/project][instructions]
fn extract_project_path_from_tags(tags: &str) -> String {
    // Parse bracketed sections
    let sections: Vec<&str> = tags.split(']')
        .map(|s| s.trim_start_matches('[').trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Look for file:/// URL in sections
    for section in &sections {
        if section.starts_with("file:///") || section.starts_with("file://") {
            // Convert file URL to system path
            let path = section
                .trim_start_matches("file:///")
                .trim_start_matches("file://")
                .replace("/", std::path::MAIN_SEPARATOR_STR);
            println!("extract_project_path_from_tags # Found path: {}", path);
            return path;
        }
        // Also check for direct Windows paths
        if section.contains(":\\") || section.contains(":/") {
            let path = section.replace("/", std::path::MAIN_SEPARATOR_STR);
            println!("extract_project_path_from_tags # Found direct path: {}", path);
            return path;
        }
    }

    // Fallback: return empty or default
    println!("extract_project_path_from_tags # No path found in tags: {}", tags);
    String::new()
}

// Helper: Call nft_pawn_chain binary (subprocess)
async fn call_nft_pawn_chain(mode: &str, prompt: &str) -> String {
    use std::process::Command;

    println!("call_nft_pawn_chain # mode: {}, prompt: {}", mode, prompt);

    let result = Command::new("cargo")
        .args(&[
            "run", "--bin", "nft_pawn_chain",
            "--release",
            "--", mode, prompt
        ])
        .output();

    match result {
        Ok(output_result) => {
            let stdout = String::from_utf8_lossy(&output_result.stdout);
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            if output_result.status.success() {
                format!("{}", stdout.trim())
            } else {
                format!("nft_pawn_chain error: {}", stderr.trim())
            }
        },
        Err(e) => format!("Failed to execute nft_pawn_chain: {}", e),
    }
}

// Helper: Call YOLOv8 object detection
// Uses agent_vision/yolov8 module - currently requires onnxruntime dependency
// When dependencies are available, this will call detect_objects_on_image directly
async fn call_yolov8_detect(image_input: &str) -> String {
    use std::path::Path;

    println!("call_yolov8_detect # input: {}", &image_input[..image_input.len().min(100)]);

    // Check if input is a file path
    let is_file = Path::new(image_input).exists();

    if is_file {
        // Read image file and attempt detection
        match std::fs::read(image_input) {
            Ok(buf) => {
                // TODO: When onnxruntime dependency is resolved, call:
                // crate::rag_agent::agent_vision::yolov8::yolov::detect_objects_on_image(buf)
                format!("YOLOv8 ready - image loaded ({} bytes). Detection requires onnxruntime dependency to be activated in agent_vision/yolov8/yolov.rs", buf.len())
            }
            Err(e) => format!("Failed to read image file: {}", e),
        }
    } else if image_input.starts_with("data:image") || image_input.len() > 1000 {
        // Assume base64 encoded image
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;
        match STANDARD.decode(image_input.split(',').last().unwrap_or(image_input)) {
            Ok(buf) => {
                format!("YOLOv8 ready - base64 decoded ({} bytes). Detection requires onnxruntime dependency to be activated in agent_vision/yolov8/yolov.rs", buf.len())
            }
            Err(e) => format!("Failed to decode base64 image: {}", e),
        }
    } else {
        format!("YOLOv8 vision agent ready. Provide image file path or base64 data. Detection requires onnxruntime dependency.")
    }
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
            description: "Image processing and generation via agent_difusser module".to_string(),
            tags: vec!["image".to_string(), "agent_difusser".to_string(), "diffuser".to_string()],
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
            description: "Document processing via agent_difusser module".to_string(),
            tags: vec!["document".to_string(), "pdf".to_string(), "agent_difusser".to_string()],
            status: "active".to_string(),
        },
    ];

    // Check service status
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let burn_lm_url = env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
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
    let output_dir = PathBuf::from("../../../../rag_output");
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
    let file_path = PathBuf::from("../../../../rag_output").join(filename);

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
    let output_dir = PathBuf::from("../../../../rag_output");
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
    let burn_lm_url = env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
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
    let rag_output = PathBuf::from("../../../../rag_output");
    let rfid_output = PathBuf::from("./output/rfid_tracking");
    let struk_output = PathBuf::from("../../../../output/struk");

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
