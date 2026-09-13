use dotenv::dotenv;
use qdrant_client::{
    // prelude::*,
    qdrant::SearchPoints,
};
use std::env;
use std::fs::metadata;
use actix_web::http::StatusCode;
use log::{debug, info};
use qdrant_client::qdrant::VectorParams;
use serde_json::json;
use reqwest::Client as HttpClient;
use uuid::Uuid;
use qdrant_client::qdrant::SearchPointsBuilder;
use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder, SearchParamsBuilder};
use qdrant_client::Qdrant;
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    fmt_message, fmt_placeholder, fmt_template,
    language_models::llm::LLM as langchainLLM,
    // llm::ollama::{openai},// add llm local
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::messages::Message,
    template_fstring,
};
use warp::Reply;
use crate::domain::models::llm::{CmdBody, RequestBody};
use crate::rag_chaining::rag_pipeline::{rag_pipeline, rag_pipeline_rfid};
use crate::shared::helperUtils::{current_time, srv_response};
use crate::WebResult;
use crate::shared::sharedUtils::{GLOBAL_ARRAY, LLM};
use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::fmt::format;
use std::sync::{Arc, Mutex};
use tokio;
use serde::{Deserialize, Serialize};
use reqwest;
use anyhow::{Result, anyhow, Context};
use chrono::{DateTime, Utc};
use crate::shared::secureUtils::generate_api_key;

//text2img, text2text, text2voice
pub async fn rfid_model(uid : String, body : CmdBody) -> WebResult<impl Reply> {
    //note: tambahkan log id_app & id_req & id_trx / id_user jika aplikasi dengan login
    info!("QUERY VEC request - start");

    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    // let config = QdrantClient::from(&qdrant_url);
    let qdrant_client = Qdrant::from_url(&*qdrant_url).build().unwrap();

    let query = body.clone().prompt;
    let response = rag_pipeline_rfid(&qdrant_client, query.as_str(), body.clone()).await;
    let mut versionModel = "version";

    info!("{} {}", format!("QUER VEC: Quick Think using flex embedding : {} {} Version {} At {} ", uid, body.clone().model, versionModel, current_time()),
             StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap());

    info!("QUERY VEC Success Result Quick Think");
    let result = &*response.unwrap();
    srv_response(format!("{}",result), StatusCode::from_u16(StatusCode::OK.as_u16()).unwrap())
}


// RFID Card/Tag data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidTag {
    pub uid: String,
    pub tag_type: RfidTagType,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub read_count: u32,
    pub last_seen: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RfidTagType {
    Mifare1K,
    Mifare4K,
    MifareUltralight,
    NTAG213,
    NTAG215,
    NTAG216,
    ISO14443A,
    ISO14443B,
    ISO15693,
    Unknown,
}

impl RfidTagType {
    pub fn from_atr(atr: &[u8]) -> Self {
        match atr {
            [0x04, ..] => RfidTagType::Mifare1K,
            [0x02, ..] => RfidTagType::Mifare4K,
            [0x44, ..] => RfidTagType::MifareUltralight,
            [0x04, 0x04, ..] => RfidTagType::NTAG213,
            [0x04, 0x02, ..] => RfidTagType::NTAG215,
            [0x04, 0x01, ..] => RfidTagType::NTAG216,
            _ => RfidTagType::Unknown,
        }
    }

    pub fn memory_size(&self) -> usize {
        match self {
            RfidTagType::Mifare1K => 1024,
            RfidTagType::Mifare4K => 4096,
            RfidTagType::MifareUltralight => 64,
            RfidTagType::NTAG213 => 180,
            RfidTagType::NTAG215 => 540,
            RfidTagType::NTAG216 => 924,
            _ => 0,
        }
    }
}

// RFID Reader configuration
#[derive(Debug, Clone)]
pub struct RfidReaderConfig {
    pub device_path: String,
    pub baud_rate: u32,
    pub timeout_ms: u64,
    pub auto_scan: bool,
    pub scan_interval_ms: u64,
    pub max_retries: u32,
}

impl Default for RfidReaderConfig {
    fn default() -> Self {
        Self {
            device_path: "/dev/ttyUSB0".to_string(), // Linux default
            baud_rate: 115200,
            timeout_ms: 5000,
            auto_scan: true,
            scan_interval_ms: 1000,
            max_retries: 3,
        }
    }
}

// RFID Reader interface
pub struct RfidReader {
    config: RfidReaderConfig,
    is_connected: bool,
    tag_cache: Arc<Mutex<HashMap<String, RfidTag>>>,
    scan_count: Arc<Mutex<u64>>,
}

impl RfidReader {
    pub fn new(config: RfidReaderConfig) -> Self {
        Self {
            config,
            is_connected: false,
            tag_cache: Arc::new(Mutex::new(HashMap::new())),
            scan_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        println!("[RFID] Connecting to RFID reader at {}...", self.config.device_path);

        // Simulate connection (in real implementation, use serial port)
        tokio::time::sleep(Duration::from_millis(500)).await;

        self.is_connected = true;
        println!("[OK] RFID reader connected successfully");
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if self.is_connected {
            println!("[RFID] Disconnecting RFID reader...");
            self.is_connected = false;
            println!("[OK] RFID reader disconnected");
        }
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    // Scan for RFID tags
    pub async fn scan_for_tags(&self) -> Result<Vec<RfidTag>> {
        if !self.is_connected {
            return Err(anyhow!("RFID reader not connected"));
        }

        println!(" Scanning for RFID tags...");

        // Simulate scanning process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Increment scan count
        {
            let mut count = self.scan_count.lock().unwrap();
            *count += 1;
        }

        // Simulate finding tags (in real implementation, communicate with hardware)
        let mut tags = Vec::new();

        // Simulate random tag detection
        if rand::random::<f32>() > 0.3 {
            let uid = self.generate_simulated_uid();
            let tag_type = RfidTagType::Mifare1K;
            let data = self.generate_simulated_data(&tag_type);

            let tag = RfidTag {
                uid: uid.clone(),
                tag_type,
                data,
                timestamp: Utc::now(),
                read_count: 1,
                last_seen: Utc::now(),
                metadata: HashMap::new(),
            };

            // Update cache
            {
                let mut cache = self.tag_cache.lock().unwrap();
                if let Some(existing_tag) = cache.get_mut(&uid) {
                    existing_tag.read_count += 1;
                    existing_tag.last_seen = Utc::now();
                } else {
                    cache.insert(uid.clone(), tag.clone());
                }
            }

            tags.push(tag);
            println!(" Found RFID tag: {}", uid);
        }

        Ok(tags)
    }

    // Read specific tag data
    pub async fn read_tag(&self, uid: &str) -> Result<RfidTag> {
        if !self.is_connected {
            return Err(anyhow!("RFID reader not connected"));
        }

        println!(" Reading tag data for UID: {}", uid);

        // Check cache first
        {
            let cache = self.tag_cache.lock().unwrap();
            if let Some(tag) = cache.get(uid) {
                return Ok(tag.clone());
            }
        }

        // Simulate reading from hardware
        tokio::time::sleep(Duration::from_millis(300)).await;

        Err(anyhow!("Tag not found: {}", uid))
    }

    // Write data to tag
    pub async fn write_tag(&self, uid: &str, data: &[u8]) -> Result<()> {
        if !self.is_connected {
            return Err(anyhow!("RFID reader not connected"));
        }

        println!(" Writing {} bytes to tag: {}", data.len(), uid);

        // Simulate writing process
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Update cache
        {
            let mut cache = self.tag_cache.lock().unwrap();
            if let Some(tag) = cache.get_mut(uid) {
                tag.data = data.to_vec();
                tag.last_seen = Utc::now();
            }
        }

        println!(" Data written successfully to tag: {}", uid);
        Ok(())
    }

    // Get scan statistics
    pub fn get_stats(&self) -> RfidStats {
        let scan_count = *self.scan_count.lock().unwrap();
        let cache_size = self.tag_cache.lock().unwrap().len();

        RfidStats {
            total_scans: scan_count,
            unique_tags: cache_size,
            is_connected: self.is_connected,
            uptime: Duration::from_secs(60), // Simulate uptime
        }
    }

    // Clear tag cache
    pub fn clear_cache(&self) {
        let mut cache = self.tag_cache.lock().unwrap();
        cache.clear();
        println!("[OK] Tag cache cleared");
    }

    // Helper methods for simulation
    fn generate_simulated_uid(&self) -> String {
        let uid_bytes: [u8; 4] = rand::random();
        uid_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":")
    }

    fn generate_simulated_data(&self, tag_type: &RfidTagType) -> Vec<u8> {
        let size = tag_type.memory_size().min(64); // Limit for simulation
        (0..size).map(|i| (i % 256) as u8).collect()
    }
}

#[derive(Debug, Clone)]
pub struct RfidStats {
    pub total_scans: u64,
    pub unique_tags: usize,
    pub is_connected: bool,
    pub uptime: Duration,
}

// Ollama integration for RFID use cases
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_name: String,
    pub timeout_seconds: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()).to_string(),
            model_name: "llama3:latest".to_string(),
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

pub struct RfidAiAssistant {
    ollama_config: OllamaConfig,
    client: reqwest::Client,
}

impl RfidAiAssistant {
    pub fn new(config: OllamaConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            ollama_config: config,
            client,
        }
    }

    // Analyze RFID tag data using AI
    pub async fn analyze_tag(&self, tag: &RfidTag) -> Result<String> {
        let prompt = format!(
            "Analyze this RFID tag data:\n\
            UID: {}\n\
            Type: {:?}\n\
            Data Size: {} bytes\n\
            Read Count: {}\n\
            Last Seen: {}\n\
            \n\
            Provide insights about:\n\
            1. Tag type and capabilities\n\
            2. Potential use cases\n\
            3. Security considerations\n\
            4. Data patterns if any",
            tag.uid, tag.tag_type, tag.data.len(), tag.read_count, tag.last_seen
        );

        self.query_ollama(&prompt).await
    }

    // Generate access control recommendations
    pub async fn access_control_analysis(&self, tag: &RfidTag, context: &str) -> Result<String> {
        let prompt = format!(
            "RFID Access Control Analysis:\n\
            Tag UID: {}\n\
            Context: {}\n\
            Read Count: {}\n\
            \n\
            Provide recommendations for:\n\
            1. Access level appropriate for this tag\n\
            2. Security measures needed\n\
            3. Monitoring requirements\n\
            4. Potential risks and mitigations",
            tag.uid, context, tag.read_count
        );

        self.query_ollama(&prompt).await
    }

    // Inventory management insights
    pub async fn inventory_analysis(&self, tags: &[RfidTag]) -> Result<String> {
        let tag_summary = tags.iter()
            .map(|tag| format!("UID: {}, Type: {:?}, Reads: {}", tag.uid, tag.tag_type, tag.read_count))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "RFID Inventory Analysis:\n\
            Total Tags: {}\n\
            Tag Details:\n{}\n\
            \n\
            Provide analysis on:\n\
            1. Inventory distribution\n\
            2. Usage patterns\n\
            3. Optimization opportunities\n\
            4. Anomaly detection",
            tags.len(), tag_summary
        );

        self.query_ollama(&prompt).await
    }

    // Security audit for RFID system
    pub async fn security_audit(&self, tags: &[RfidTag], system_info: &str) -> Result<String> {
        let prompt = format!(
            "RFID Security Audit:\n\
            System: {}\n\
            Total Tags Monitored: {}\n\
            \n\
            Analyze and provide:\n\
            1. Security vulnerabilities\n\
            2. Compliance recommendations\n\
            3. Best practices implementation\n\
            4. Risk assessment\n\
            5. Monitoring improvements",
            system_info, tags.len()
        );

        self.query_ollama(&prompt).await
    }

    // Asset tracking insights
    pub async fn asset_tracking_analysis(&self, tag: &RfidTag, location: &str) -> Result<String> {
        let prompt = format!(
            "RFID Asset Tracking Analysis:\n\
            Asset Tag: {}\n\
            Current Location: {}\n\
            Movement History: {} reads\n\
            Last Activity: {}\n\
            \n\
            Provide insights on:\n\
            1. Asset utilization patterns\n\
            2. Location optimization\n\
            3. Maintenance scheduling\n\
            4. Cost analysis",
            tag.uid, location, tag.read_count, tag.last_seen
        );

        self.query_ollama(&prompt).await
    }

    async fn query_ollama(&self, prompt: &str) -> Result<String> {
        let mut options = HashMap::new();
        options.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
        options.insert("top_p".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.9).unwrap()));

        let request = OllamaRequest {
            model: self.ollama_config.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options,
        };

        let url = format!("{}/api/generate", self.ollama_config.base_url);

        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let ollama_response: OllamaResponse = response.json().await?;
            Ok(ollama_response.response)
        } else {
            Err(anyhow!("Ollama API error: {}", response.status()))
        }
    }
}

// RFID Application with various use cases
pub struct RfidApplication {
    reader: RfidReader,
    ai_assistant: RfidAiAssistant,
    use_case: RfidUseCase,
}

#[derive(Debug, Clone)]
pub enum RfidUseCase {
    AccessControl { location: String },
    InventoryManagement { warehouse: String },
    AssetTracking { facility: String },
    AttendanceSystem { organization: String },
    PaymentSystem { merchant: String },
    SecurityAudit { system_name: String },
}

impl RfidApplication {
    pub fn new(reader_config: RfidReaderConfig, ollama_config: OllamaConfig, use_case: RfidUseCase) -> Self {
        Self {
            reader: RfidReader::new(reader_config),
            ai_assistant: RfidAiAssistant::new(ollama_config),
            use_case,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!(" Initializing RFID Application...");
        println!("Use Case: {:?}", self.use_case);

        self.reader.connect().await?;
        println!(" RFID Application initialized successfully");
        Ok(())
    }

    pub async fn run_continuous_scan(&self) -> Result<()> {
        println!(" Starting continuous RFID scanning...");

        loop {
            match self.reader.scan_for_tags().await {
                Ok(tags) => {
                    for tag in tags {
                        self.process_tag(&tag).await?;
                    }
                }
                Err(e) => {
                    eprintln!(" Scan error: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_millis(self.reader.config.scan_interval_ms)).await;
        }
    }

    async fn process_tag(&self, tag: &RfidTag) -> Result<()> {
        println!("\n Processing tag: {}", tag.uid);

        match &self.use_case {
            RfidUseCase::AccessControl { location } => {
                let analysis = self.ai_assistant.access_control_analysis(tag, location).await?;
                println!(" Access Control Analysis:\n{}", analysis);
            }
            RfidUseCase::InventoryManagement { warehouse } => {
                let analysis = self.ai_assistant.inventory_analysis(&[tag.clone()]).await?;
                println!(" Inventory Analysis:\n{}", analysis);
            }
            RfidUseCase::AssetTracking { facility } => {
                let analysis = self.ai_assistant.asset_tracking_analysis(tag, facility).await?;
                println!(" Asset Tracking Analysis:\n{}", analysis);
            }
            RfidUseCase::AttendanceSystem { organization } => {
                println!(" Attendance recorded for {} at {}", tag.uid, organization);
                let analysis = self.ai_assistant.analyze_tag(tag).await?;
                println!(" Tag Analysis:\n{}", analysis);
            }
            RfidUseCase::PaymentSystem { merchant } => {
                println!(" Payment processed for {} at {}", tag.uid, merchant);
                let analysis = self.ai_assistant.analyze_tag(tag).await?;
                println!(" Payment Analysis:\n{}", analysis);
            }
            RfidUseCase::SecurityAudit { system_name } => {
                let analysis = self.ai_assistant.security_audit(&[tag.clone()], system_name).await?;
                println!(" Security Audit:\n{}", analysis);
            }
        }

        Ok(())
    }

    pub fn get_stats(&self) -> RfidStats {
        self.reader.get_stats()
    }

    pub fn shutdown(&mut self) {
        println!(" Shutting down RFID Application...");
        self.reader.disconnect();
        println!(" RFID Application shutdown complete");
    }
}

// Interactive CLI interface
async fn run_interactive_mode() -> Result<()> {
    println!("\n RFID Interactive Mode");
    println!("========================");

    let reader_config = RfidReaderConfig::default();
    let ollama_config = OllamaConfig::default();
    let use_case = RfidUseCase::AccessControl { location: "Main Office".to_string() };

    let mut app = RfidApplication::new(reader_config, ollama_config, use_case);
    app.initialize().await?;

    println!("\nCommands:");
    println!("  scan    - Scan for RFID tags");
    println!("  stats   - Show statistics");
    println!("  clear   - Clear tag cache");
    println!("  config  - Change configuration");
    println!("  help    - Show this help");
    println!("  quit    - Exit application");

    loop {
        print!("\nRFID> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "scan" => {
                match app.reader.scan_for_tags().await {
                    Ok(tags) => {
                        if tags.is_empty() {
                            println!("No tags found");
                        } else {
                            for tag in tags {
                                app.process_tag(&tag).await?;
                            }
                        }
                    }
                    Err(e) => println!(" Scan failed: {}", e),
                }
            }
            "stats" => {
                let stats = app.get_stats();
                println!(" RFID Statistics:");
                println!("  Total Scans: {}", stats.total_scans);
                println!("  Unique Tags: {}", stats.unique_tags);
                println!("  Connected: {}", stats.is_connected);
                println!("  Uptime: {:?}", stats.uptime);
            }
            "clear" => {
                app.reader.clear_cache();
            }
            "config" => {
                println!("Current configuration:");
                println!("  Use Case: {:?}", app.use_case);
                println!("  Reader Connected: {}", app.reader.is_connected());
            }
            "help" => {
                println!("Available commands:");
                println!("  scan    - Scan for RFID tags");
                println!("  stats   - Show statistics");
                println!("  clear   - Clear tag cache");
                println!("  config  - Show configuration");
                println!("  help    - Show this help");
                println!("  quit    - Exit application");
            }
            "quit" | "exit" => {
                app.shutdown();
                println!(" Goodbye!");
                break;
            }
            "" => continue,
            _ => println!(" Unknown command. Type 'help' for available commands."),
        }
    }

    Ok(())
}

// dev scenarios
async fn run_dev_scenarios() -> Result<()> {
    println!("Running RFID dev Scenarios");
    println!("==============================");

    let scenarios = vec![
        ("Access Control", RfidUseCase::AccessControl { location: "Server Room".to_string() }),
        ("Inventory Management", RfidUseCase::InventoryManagement { warehouse: "Warehouse A".to_string() }),
        ("Asset Tracking", RfidUseCase::AssetTracking { facility: "Manufacturing Plant".to_string() }),
        ("Attendance System", RfidUseCase::AttendanceSystem { organization: "Tech Corp".to_string() }),
    ];

    for (name, use_case) in scenarios {
        println!("\n dev: {}", name);
        println!("{}=", "=".repeat(name.len() + 7));

        let reader_config = RfidReaderConfig::default();
        let ollama_config = OllamaConfig::default();

        let mut app = RfidApplication::new(reader_config, ollama_config, use_case);

        if let Err(e) = app.initialize().await {
            println!("Failed to initialize {}: {}", name, e);
            continue;
        }

        // Simulate some tag scans
        for _ in 0..3 {
            match app.reader.scan_for_tags().await {
                Ok(tags) => {
                    for tag in tags {
                        app.process_tag(&tag).await?;
                    }
                }
                Err(e) => println!("Scan error: {}", e),
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let stats = app.get_stats();
        println!(" dev Stats: {} scans, {} unique tags", stats.total_scans, stats.unique_tags);

        app.shutdown();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("RFID Scanner with AI Integration");
    println!("===================================");

    println!("\nSelect mode:");
    println!("1. Interactive Mode");
    println!("2. dev Scenarios");
    println!("3. Continuous Scanning");
    println!("4. Exit");

    print!("\nEnter choice (1-4): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            run_interactive_mode().await?;
        }
        "2" => {
            run_dev_scenarios().await?;
        }
        "3" => {
            // Start Init Data Chunk's Vec
            let uid= generate_api_key();
            let body= CmdBody{
                model : "llama3".to_string(),
                prompt : format!("save_log_hist {:?}",uid).to_string(),
                cmd: "".to_string(),
                tags: "".to_string(),
                options : "".to_string(),
                keepAlive : "".to_string(),
            };
            rfid_model(uid.parse()?, body).await.expect("Fill Request Chunks Data");
            // Setup IO's
            let reader_config = RfidReaderConfig::default();
            let ollama_config = OllamaConfig::default();
            let use_case = RfidUseCase::AccessControl { location: "Main Entrance".to_string() };

            let mut app = RfidApplication::new(reader_config, ollama_config, use_case);
            app.initialize().await?;

            println!("Starting continuous scanning (Ctrl+C to stop)...");
            app.run_continuous_scan().await?;
        }
        "4" => {
            println!("Goodbye!");
        }
        _ => {
            println!("Invalid choice. Exiting...");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rfid_reader_connection() {
        let config = RfidReaderConfig::default();
        let mut reader = RfidReader::new(config);

        assert!(!reader.is_connected());

        let result = reader.connect().await;
        assert!(result.is_ok());
        assert!(reader.is_connected());

        reader.disconnect();
        assert!(!reader.is_connected());
    }

    #[tokio::test]
    async fn test_tag_scanning() {
        let config = RfidReaderConfig::default();
        let mut reader = RfidReader::new(config);

        reader.connect().await.unwrap();

        let tags = reader.scan_for_tags().await.unwrap();
        // Tags may or may not be found in simulation
        assert!(tags.len() <= 10); // Reasonable upper bound
    }

    #[test]
    fn test_tag_type_detection() {
        let mifare_atr = [0x04, 0x00];
        let tag_type = RfidTagType::from_atr(&mifare_atr);
        assert!(matches!(tag_type, RfidTagType::Mifare1K));

        let unknown_atr = [0xFF, 0xFF];
        let tag_type = RfidTagType::from_atr(&unknown_atr);
        assert!(matches!(tag_type, RfidTagType::Unknown));
    }

    #[test]
    fn test_memory_sizes() {
        assert_eq!(RfidTagType::Mifare1K.memory_size(), 1024);
        assert_eq!(RfidTagType::MifareUltralight.memory_size(), 64);
        assert_eq!(RfidTagType::NTAG213.memory_size(), 180);
    }
}
