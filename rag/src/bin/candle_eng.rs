//! Candle Engine - RFID AI Analysis CLI
//!
//! Usage:
//!   candle_eng ask -q "<prompt>"
//!
//! Examples:
//!   candle_eng ask -q "[No write code, direct data result] 'data': [...] create changelog metadata [rfid][Prediction]"
//!   candle_eng ask -q "[No write code, direct data result] 'data': [...] create changelog metadata [rfid][Prevention]"

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::{info, error};
use rag::rag_chaining::base_chain_fastembed;
use rag::rag_chaining::base_chain_fastembed::RagApiResponse;
use serde_json::{json, Value};
use std::env;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "candle_eng")]
#[command(author = "istamar")]
#[command(version = "1.0")]
#[command(about = "Candle Engine - RFID AI Analysis CLI", long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ask AI with prompt and RFID data
    Ask {
        /// Query prompt with RFID data context
        #[arg(short, long)]
        q: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    dotenv().ok();

    let cli = CliArgs::parse();

    match cli.command {
        Commands::Ask { q } => {
            match run_ask(q).await {
                Ok(changelog) => {
                    // Output structured changelog metadata JSON to stdout
                    println!("{}", changelog);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    error!("Candle Engine failed: {}", e);
                    println!("{{\"status\": \"error\", \"message\": \"{}\"}}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

async fn run_ask(prompt: String) -> Result<String, String> {
    info!("Candle Engine processing prompt: {}", prompt);

    // Extract RFID data from prompt if present
    let rfid_data = extract_rfid_data(&prompt);
    
    // Determine analysis type from prompt
    let analysis_type = determine_analysis_type(&prompt);
    
    // Build enhanced prompt with RFID context
    let enhanced_prompt = if let Some(data) = rfid_data.as_ref() {
        format!(
            "RFID AI Analysis Request - Type: {}\n\nRFID Data:\n{}\n\nOriginal Query:\n{}",
            analysis_type,
            serde_json::to_string_pretty(data).unwrap_or_else(|_| "Invalid JSON".to_string()),
            prompt
        )
    } else {
        prompt.clone()
    };

    // Call base_chain AI engine
    let response = base_chain_fastembed::run(enhanced_prompt).await
        .map_err(|e| format!("AI engine error: {}", e))?;

    // Generate changelog metadata
    let changelog = generate_changelog_metadata(analysis_type, &response, rfid_data.as_ref());

    Ok(changelog)
}

fn extract_rfid_data(prompt: &str) -> Option<Value> {
    // Look for 'data': [...] pattern in prompt
    if let Some(start) = prompt.find("'data':") {
        let data_section = &prompt[start..];
        if let Some(end) = data_section.find(']') {
            let json_str = format!("{{{}}}", &data_section[..=end]);
            if let Ok(value) = serde_json::from_str::<Value>(&json_str) {
                return Some(value);
            }
        }
    }
    None
}

fn determine_analysis_type(prompt: &str) -> String {
    let prompt_lower = prompt.to_lowercase();
    if prompt_lower.contains("[prediction]") || prompt_lower.contains("prediction") {
        "Prediction".to_string()
    } else if prompt_lower.contains("[prevention]") || prompt_lower.contains("prevention") {
        "Prevention".to_string()
    } else {
        "General".to_string()
    }
}

fn generate_changelog_metadata(analysis_type: String, response: &RagApiResponse, rfid_data: Option<&Value>) -> String {
    let changelog = json!({
        "type": format!("rfid_{}", analysis_type.to_lowercase()),
        "generated_at": response.timestamp.clone(),
        "status": response.status.clone(),
        "analysis_type": analysis_type.clone(),
        "rfid_data": rfid_data.cloned(),
        "ai_response": {
            "message": response.message.clone(),
            "project_name": response.data.project_name.clone(),
            "rag_response_preview": response.data.rag_response.chars().take(500).collect::<String>()
        },
        "changelog": generate_changelog_entries(analysis_type.clone(), response, rfid_data)
    });

    serde_json::to_string_pretty(&changelog).unwrap_or_else(|_| "{}".to_string())
}

fn generate_changelog_entries(analysis_type: String, response: &RagApiResponse, rfid_data: Option<&Value>) -> Vec<String> {
    let mut entries = vec![];

    entries.push(format!("[{}] Analysis completed via {}", analysis_type, response.data.project_path));

    if let Some(data) = rfid_data {
        if let Some(tag_id) = data.get("tag_id").and_then(|v| v.as_str()) {
            entries.push(format!("[{}] RFID Tag: {}", analysis_type, tag_id));
        }
    }

    if response.status == "success" {
        entries.push(format!("[{}] AI response generated successfully", analysis_type));
    } else {
        entries.push(format!("[{}] AI response: {}", analysis_type, response.message));
    }

    entries
}
