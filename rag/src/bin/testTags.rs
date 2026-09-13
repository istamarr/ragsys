use clap::{Parser, Subcommand};
use colored::*;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow, bail};
use log::{error, info};

#[derive(Parser)]
#[command(name = "rag-cli")]
#[command(about = "A RAG CLI tool with tag validation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        /// Input text with tags: [type][appname][url][freetext]
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        file: Option<String>,
    },
    Query {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "3")]
        top_k: usize,
    },
    List,
    Validate {
        input: String,
    },
}

// Tag validation structures
#[derive(Debug, Clone, PartialEq)]
enum DocumentType {
    Fix,
    Correct,
    Flow,
    Code,
    Describe,
    Changes,
}

impl DocumentType {
    fn from_str(s: &str) -> Result<String> {
        match s.to_lowercase().as_str() {
            "fix" => Ok("Fix".to_string()),
            "correct" => Ok("Correct".to_string()),
            "flow" => Ok("Flow".to_string()),
            "code" => Ok("Code".to_string()),
            "describe" => Ok("Describe".to_string()),
            "changes" => Ok("Changes".to_string()),
            _ =>
                Ok(format!("Invalid type '{}'. Valid options: fix, correct, flow, code, describe, changes", s)),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            DocumentType::Fix => "fix",
            DocumentType::Correct => "correct",
            DocumentType::Flow => "flow",
            DocumentType::Code => "code",
            DocumentType::Describe => "describe",
            DocumentType::Changes => "changes",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AppName {
    App1,
    App2,
}

impl AppName {
    fn from_str(s: &str) -> Result<String> {
        match s.to_lowercase().as_str() {
            "app1" => Ok("App1".to_string()),
            "app2" => Ok("App2".to_string()),
            // "app2" => Ok(AppName::App2),
            _ =>
            Ok(format!("Invalid appname '{:?}'. Valid options: app1, app2",s)),
            // println!(format!("Invalid appname '{:?}'. Valid options: app1, app2",s)),
            // Err(bail!("Invalid appname '{}'. Valid options: app1, app2", s)),
            // Err(s) => error!("{} # failed to parse type_q as i32: {}",log.clone(), s);
            // Err(e) => println!("Invalid appname '{}'. Valid options: app1, app2", e),
        }
    }

    // pub fn convert_redis_result(result: redis::RedisResult<String>) -> String {
    //     match result {
    //         Ok(value) => value,
    //         Err(e) => format!("Error: {}", e),
    //     }
    // }


    fn as_str(&self) -> &'static str {
        match self {
            AppName::App1 => "app1",
            AppName::App2 => "app2",
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedTags {
    // doc_type: DocumentType,
    doc_type: String,
    // app_name: AppName,
    app_name: String,
    url: String,
    free_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    id: String,
    content: String,
    doc_type: String,
    app_name: String,
    url: String,
    free_text: String,
    metadata: HashMap<String, String>,
}

// Tag parser and validator: Apis And Command
struct TagValidator;

impl TagValidator {
    fn parse_and_validate(input: &str) -> Result<ParsedTags> {
        let log = " Validate Input Tags".to_string();
        // Regex pattern [type][appname][url][freetext]
        let re = Regex::new(r"^\[([^\]]+)\]\[([^\]]+)\]\[([^\]]*)\]\[([^\]]*)\](.*)$")?;

        if let Some(caps) = re.captures(input.trim()) {
            info!("{} ~ Capture Tags", log.clone());
            let type_str = caps.get(1).unwrap().as_str();
            let app_str = caps.get(2).unwrap().as_str();
            let url_str = caps.get(3).unwrap().as_str();
            let free_text_str = caps.get(4).unwrap().as_str();
            let remaining = caps.get(5).unwrap().as_str().trim();

            // Validate type
            let doc_type = DocumentType::from_str(type_str)?;
            info!("{} ~ Validate type {:?}", log.clone(), doc_type);

            // Validate app name
            let app_name = AppName::from_str(app_str)?;
            info!("{} ~ Validate app name {:?}", log.clone(), app_name);

            let mut full_free_text = free_text_str.to_string();
            info!("{} ~ Combine and remaining content {:?}", log.clone(), full_free_text);

            if !remaining.is_empty() {
                if !full_free_text.is_empty() {
                    full_free_text.push(' ');
                }
                full_free_text.push_str(remaining);
            }

            Ok(ParsedTags {
                doc_type,
                app_name,
                url: url_str.to_string(),
                free_text: full_free_text,
            })
        } else {
            error!("Invalid tag format. Expected: [type][appname][url][freetext]");
            Err(anyhow!("Invalid tag format. Expected: [type][appname][url][freetext]"))
        }
    }

    fn validate_and_display(input: &str) -> Result<()> {
        match Self::parse_and_validate(input) {
            Ok(tags) => {
                println!("{}", " Valid tag format!".green().bold());
                println!("  {}: {}", "Type".cyan().bold(), tags.doc_type.as_str());
                println!("  {}: {}", "App Name".cyan().bold(), tags.app_name.as_str());
                println!("  {}: {}", "URL".cyan().bold(), if tags.url.is_empty() { "(empty)" } else { &tags.url });
                println!("  {}: {}", "Free Text".cyan().bold(), if tags.free_text.is_empty() { "(empty)" } else { &tags.free_text });
                Ok(())
            }
            Err(e) => {
                println!("{}", " Invalid tag format!".red().bold());
                println!("  {}: {}", "Error".red().bold(), e);
                println!("\n{}", "Expected format:".yellow().bold());
                println!("  [type][appname][url][freetext]");
                println!("\n{}", "Valid types:".yellow().bold());
                println!("  fix, correct, flow, code, describe, changes");
                println!("\n{}", "Valid app names:".yellow().bold());
                println!("  app1, app2");
                println!("\n{}", "Example:".yellow().bold());
                println!("  [fix][app1][https://example.com][This is a bug fix for the login system]");
                Err(e)
            }
        }
    }
}

// Simple in-memory vector store
#[derive(Default)]
struct DocumentStore {
    documents: Vec<Document>,
    next_id: usize,
}

impl DocumentStore {
    fn new() -> Self {
        Self::default()
    }

    fn add_document(&mut self, tags: ParsedTags, full_content: String) -> String {
        let id = format!("doc_{}", self.next_id);
        self.next_id += 1;

        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), tags.doc_type.as_str().to_string());
        metadata.insert("app".to_string(), tags.app_name.as_str().to_string());

        let document = Document {
            id: id.clone(),
            content: full_content,
            doc_type: tags.doc_type.as_str().to_string(),
            app_name: tags.app_name.as_str().to_string(),
            url: tags.url,
            free_text: tags.free_text,
            metadata,
        };

        self.documents.push(document);
        id
    }

    fn search(&self, query: &str, top_k: usize) -> Vec<&Document> {
        let query_lower = query.to_lowercase();
        let mut scored_docs: Vec<(&Document, usize)> = self
            .documents
            .iter()
            .map(|doc| {
                let content_lower = doc.content.to_lowercase();
                let free_text_lower = doc.free_text.to_lowercase();

                let content_score = query_lower
                    .split_whitespace()
                    .map(|word| content_lower.matches(word).count())
                    .sum::<usize>();

                let free_text_score = query_lower
                    .split_whitespace()
                    .map(|word| free_text_lower.matches(word).count())
                    .sum::<usize>();

                let type_score = if content_lower.contains(&doc.doc_type) { 5 } else { 0 };
                let app_score = if content_lower.contains(&doc.app_name) { 3 } else { 0 };

                (doc, content_score + free_text_score * 2 + type_score + app_score)
            })
            .collect();

        scored_docs.sort_by(|a, b| b.1.cmp(&a.1));
        scored_docs
            .into_iter()
            .filter(|(_, score)| *score > 0)
            .take(top_k)
            .map(|(doc, _)| doc)
            .collect()
    }

    fn list_all(&self) -> &[Document] {
        &self.documents
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.documents)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_from_file(&mut self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            self.documents = serde_json::from_str(&content)?;
            self.next_id = self.documents.len();
        }
        Ok(())
    }
}

// LLM Client (simplified for dev)
struct LLMClient {
    client: Client,
}

impl LLMClient {
    fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn generate_response(&self, context: &str, query: &str) -> Result<String> {
        // In a real implementation, you'd call an actual LLM API
        // For dev purposes, we'll create a simple response
        Ok(format!(
            "Based on the retrieved documents, here's the answer to your query '{}':\n\nContext used:\n{}\n\nThis is a simulated response. In a real implementation, this would be generated by an LLM.",
            query, context
        ))
    }
}

struct RAGSystem {
    store: DocumentStore,
    llm: LLMClient,
}

impl RAGSystem {
    fn new() -> Self {
        Self {
            store: DocumentStore::new(),
            llm: LLMClient::new(),
        }
    }

    fn load_data(&mut self, file_path: &str) -> Result<()> {
        self.store.load_from_file(file_path)
    }

    fn save_data(&self, file_path: &str) -> Result<()> {
        self.store.save_to_file(file_path)
    }

    fn add_document(&mut self, input: &str) -> Result<String> {
        let tags = TagValidator::parse_and_validate(input)?;
        let doc_id = self.store.add_document(tags, input.to_string());
        Ok(doc_id)
    }

    async fn query(&self, query: &str, top_k: usize) -> Result<String> {
        let relevant_docs = self.store.search(query, top_k);

        if relevant_docs.is_empty() {
            return Ok("No relevant documents found.".to_string());
        }

        let context = relevant_docs
            .iter()
            .map(|doc| format!("[{}][{}] {}", doc.doc_type, doc.app_name, doc.free_text))
            .collect::<Vec<_>>()
            .join("\n\n");

        self.llm.generate_response(&context, query).await
    }

    fn list_documents(&self) -> &[Document] {
        self.store.list_all()
    }
}


// run ini dengan command prompt cmd
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut rag_system = RAGSystem::new();
    let data_file = "rag_data.json";

    // Load existing data
    if let Err(e) = rag_system.load_data(data_file) {
        println!("{}", format!("Warning: Could not load existing data: {}", e).yellow());
    }

    match cli.command {
        Commands::Add { input, file } => {
            match rag_system.add_document(&input) {
                Ok(doc_id) => {
                    println!("{}", format!(" Document added with ID: {}", doc_id).green().bold());

                    // Save to file if specified
                    if let Some(file_path) = file {
                        rag_system.save_data(&file_path)?;
                        println!("{}", format!(" Data saved to: {}", file_path).green());
                    } else {
                        rag_system.save_data(data_file)?;
                        println!("{}", format!(" Data saved to: {}", data_file).green());
                    }
                }
                Err(e) => {
                    println!("{}", format!(" Failed to add document: {}", e).red().bold());
                    TagValidator::validate_and_display(&input)?;
                }
            }
        }

        Commands::Query { query, top_k } => {
            println!("{}", format!("Searching for: '{}'", query).cyan().bold());

            match rag_system.query(&query, top_k).await {
                Ok(response) => {
                    println!("\n{}", "Response:".green().bold());
                    println!("{}", response);
                }
                Err(e) => {
                    println!("{}", format!("Query failed: {}", e).red().bold());
                }
            }
        }

        Commands::List => {
            let docs = rag_system.list_documents();

            if docs.is_empty() {
                println!("{}", "No documents found.".yellow());
            } else {
                println!("{}", format!("Found {} document(s):", docs.len()).cyan().bold());

                for (i, doc) in docs.iter().enumerate() {
                    println!("\n{}. {} ({})",
                             (i + 1).to_string().cyan().bold(),
                             doc.id.green(),
                             format!("[{}][{}]", doc.doc_type, doc.app_name).blue()
                    );
                    println!("   URL: {}", if doc.url.is_empty() { "(empty)".dimmed().to_string() } else { doc.url.clone() });
                    println!("   Text: {}",
                             if doc.free_text.len() > 100 {
                                 format!("{}...", &doc.free_text[..100])
                             } else {
                                 doc.free_text.clone()
                             }
                    );
                }
            }
        }

        Commands::Validate { input } => {
            TagValidator::validate_and_display(&input)?;
        }
    }

    Ok(())
}


// test dengan internal code function
#[cfg(test)]
mod tests {
    use std::env;
    use dotenv::dotenv;
    use log::{log_enabled, Level, LevelFilter};
    use super::*;

    // using functional
    #[test]
    fn test_valid_tag_parsing() {
        env_logger::init();
        dotenv().ok();
        log::set_max_level(LevelFilter::Debug);
        if log_enabled!(Level::Debug) {
            info!("test_valid_tag_parsing: [fix][app1][https://example.com][This is a bug fix]");
        }

        let input = "[fix][app1][https://example.com][This is a bug fix]";
        let result = TagValidator::parse_and_validate(input).unwrap();

        assert_eq!(result.doc_type, "Fix".to_string());
        assert_eq!(result.app_name, "App1".to_string());
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.free_text, "This is a bug fix");
    }

    #[test]
    fn test_invalid_type() {
        env_logger::init();
        dotenv().ok();
        log::set_max_level(LevelFilter::Debug);
        if log_enabled!(Level::Debug) {
            info!("test_invalid_type: [invalid][app1][url][text]");
        }

        let input = "[invalid][app1][url][text]";
        assert!(TagValidator::parse_and_validate(input).is_err());
    }

    #[test]
    fn test_invalid_app() {//istamar
        env_logger::init();
        dotenv().ok();
        log::set_max_level(LevelFilter::Debug);
        if log_enabled!(Level::Debug) {
            info!("test_invalid_app: [fix][app3][url][text]");
        }
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }

        let input = "[fix][app3][url][text]";
        assert!(TagValidator::parse_and_validate(input).is_err());
    }

    #[test]
    fn test_empty_fields() {
        let input = "[describe][app2][][Empty URL but valid]";
        let result = TagValidator::parse_and_validate(input).unwrap();

        assert_eq!(result.doc_type, "Describe".to_string());
        assert_eq!(result.app_name, "App2");
        assert_eq!(result.url, "");
        assert_eq!(result.free_text, "Empty URL but valid");
    }
}

// ---
// # Valid examples:
// [fix][app1][https://example.com][Fixed the navbar issue]
// [code][app2][][New user registration function]
// [describe][app1][/docs/api][API documentation update]
// [changes][app2][https://github.com/pr/456][Updated database schema]
//
// # Invalid examples (will be rejected):
// [invalid_type][app1][url][text]
// [fix][app3][url][text]
// fix][app1][url][text]
// [fix[app1][url][text]
// ---
