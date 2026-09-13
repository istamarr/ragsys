use dotenv::dotenv;
use log::{info, error, log_enabled, Level, LevelFilter};
use rag::rag_chaining::base_chain_fastembed;
use rag::rag_chaining::base_chain_fastembed::{RagApiResponse, Pipeline, TypePipeRag};
use std::env;
use std::process::ExitCode;
use std::path::Path;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    dotenv().ok();
    log::set_max_level(LevelFilter::Debug);

    println!("\n============================================================");
    println!("     RAG PIPELINE TEST HANDLER                              ");
    println!("     PIPELINE: FASTEMBED | QDRANT | CANDLE_ENG              ");
    println!("     TYPE_PIPE_RAG: ASIST                                   ");
    println!("     + FlowCode Analysis (flowcode,fixco option)            ");
    println!("============================================================\n");

    if log_enabled!(Level::Debug) {
        info!("Test Handler Started - Debug Mode Enabled");
    }

    // Show pipeline selection
    let pipeline = Pipeline::from_env();
    let type_pipe = TypePipeRag::from_env();
    println!("[Pipeline] PIPELINE     = {}", pipeline.as_str());
    println!("[Pipeline] TYPE_PIPE_RAG = {}", type_pipe.as_str());
    println!();

    // Show environment configuration
    println!("[Config] Environment Configuration:");
    println!("   PIPELINE:              {}", std::env::var("PIPELINE").unwrap_or_else(|_| "FASTEMBED (default)".to_string()));
    println!("   TYPE_PIPE_RAG:         {}", std::env::var("TYPE_PIPE_RAG").unwrap_or_else(|_| "ASIST (default)".to_string()));
    println!("   BURN_LM_URL:           {}", std::env::var("BURN_LM_URL").unwrap_or_else(|_| "http://localhost:9393 (default)".to_string()));
    println!("   CANDLE_ENG_URL:        {}", std::env::var("CANDLE_ENG_URL").unwrap_or_else(|_| "http://localhost:8082 (default)".to_string()));
    println!("   OLLAMA_URL:            {}", std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434 (default)".to_string()));
    println!("   OLLAMA_FALLBACK_MODEL: {}", std::env::var("OLLAMA_FALLBACK_MODEL").unwrap_or_else(|_| "ollamar/tamar:1b (default)".to_string()));
    println!("   QDRANT_URL:            {}", std::env::var("QDRANT_URL").unwrap_or_else(|_| "not set".to_string()));
    println!("   QDRANT_URL_PORT_6334:  {}", std::env::var("QDRANT_URL_PORT_6334").unwrap_or_else(|_| "http://localhost:6334 (default)".to_string()));
    println!();

    // Show FlowCode info
    println!("[FlowCode] Code Analysis Options:");
    println!("   Options: [flowcode,fixco]");
    println!("   Tags Format: [flowcode,fixco][ProjectName][file:///path][instructions]");
    println!("   Features: Project Analysis, Code Issues, Security Scan, FixCo LLM");
    println!();

    // Show pipeline flow
    println!("[Start] Starting RAG Pipeline Test...\n");
    println!("   Flow: base_chain_fastembed::run()");
    println!("         -> Pipeline::from_env() -> {}", pipeline.as_str());
    println!("         -> (if flowcode) process_flowcode_tags()");
    println!("         -> retrieve_context()   (pipeline-aware)");
    println!("         -> generate_response()  (pipeline-aware)\n");

    let args: Vec<String> = env::args().collect();
    let user_string = if args.len() > 1 {
        println!("User provided string: {}", args[1]);
        args[1].clone()
    } else {
        println!("No string provided as a command-line argument.");
        return ExitCode::FAILURE;
    };

    match base_chain_fastembed::run(user_string).await {
        Ok(response) => {
            print_api_response(&response);
            info!("RAG Pipeline completed successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("\n[ERROR] RAG Pipeline Test FAILED!");
            println!("   Error: {}", e);
            error!("RAG Pipeline failed: {}", e);

            println!("\n[Troubleshooting]:");
            println!("   1. Start Qdrant (REQUIRED for all pipelines):");
            println!("      docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant");
            println!();
            println!("   2. PIPELINE=FASTEMBED or QDRANT -> start ASIST model:");
            println!("      cd C:\\Users\\thinkpad123\\TCPNRS\\DEV_ISTA\\ai_model_server");
            println!("      cargo run --bin serve_w_candle   (port 9393)");
            println!();
            println!("   3. PIPELINE=CANDLE_ENG -> start candle_eng:");
            println!("      cd C:\\Users\\thinkpad123\\TCPNRS\\DEV_ISTA\\candle_eng");
            println!("      cargo run --bin serve_causal     (port 8082)");
            println!();
            println!("   4. Current PIPELINE={}", pipeline.as_str());
            println!("      TYPE_PIPE_RAG={}", type_pipe.as_str());

            ExitCode::FAILURE
        }
    }
}

/// Print neat API response with download links
fn print_api_response(response: &RagApiResponse) {
    println!("\n{}", "=".repeat(70));
    println!("                    RAG PIPELINE API RESPONSE                        ");
    println!("{}", "=".repeat(70));

    // Status Section
    println!("\n[STATUS]");
    println!("   Status    : {}", response.status.to_uppercase());
    println!("   Message   : {}", response.message);
    println!("   Timestamp : {}", response.timestamp);

    // Project Info Section
    println!("\n[PROJECT INFO]");
    println!("   Name      : {}", response.data.project_name);
    println!("   Path      : {}", response.data.project_path);
    println!("   Query     : {}", response.data.instructions);

    // Code Analysis Section
    if let Some(ref info) = response.data.project_info {
        println!("\n[CODE ANALYSIS]");
        println!("   Language     : {}", info.language);
        println!("   Framework    : {}", info.framework.clone().unwrap_or("N/A".to_string()));
        println!("   Total Files  : {}", info.total_files);
        println!("   Total Lines  : {}", info.total_lines);
        println!("   Dependencies : {}", info.dependencies_count);
    }

    // Code Metrics Section
    if let Some(ref metrics) = response.data.code_metrics {
        println!("\n[CODE METRICS]");
        println!("   Lines of Code          : {}", metrics.lines_of_code);
        println!("   Functions              : {}", metrics.function_count);
        println!("   Classes/Structs        : {}", metrics.class_count);
        println!("   Cyclomatic Complexity  : {:.2}", metrics.cyclomatic_complexity);
        println!("   Maintainability Index  : {:.2}", metrics.maintainability_index);
        println!("   Comment Density        : {:.2}%", metrics.comment_density);
    }

    // Issues Summary Section
    println!("\n[ISSUES SUMMARY]");
    println!("   Total Issues : {}", response.data.issues_summary.total_issues);
    println!("   Critical     : {}", response.data.issues_summary.critical);
    println!("   Warnings     : {}", response.data.issues_summary.warning);
    println!("   Info         : {}", response.data.issues_summary.info);

    // Security Summary Section
    println!("\n[SECURITY SUMMARY]");
    println!("   Vulnerabilities : {}", response.data.security_summary.vulnerabilities_count);
    println!("   High Severity   : {}", response.data.security_summary.high_severity);
    println!("   Medium Severity : {}", response.data.security_summary.medium_severity);
    println!("   Low Severity    : {}", response.data.security_summary.low_severity);

    // Diagrams Count
    println!("\n[DIAGRAMS]");
    println!("   Generated Diagrams : {}", response.data.diagrams_count);

    // Pipeline Flow Chart (Static - RAG Pipeline)
    println!("\n{}", "=".repeat(70));
    println!("                  RAG PIPELINE FLOW CHART (Static)                   ");
    println!("{}", "=".repeat(70));
    println!("{}", response.pipeline_flow_chart);

    // Project Flow Chart (Dynamic - Based on analyzed project)
    println!("\n{}", "=".repeat(70));
    println!("                PROJECT ANALYSIS FLOW CHART (Dynamic)                ");
    println!("{}", "=".repeat(70));
    println!("{}", response.project_flow_chart);

    // RAG Response Preview
    println!("\n[RAG RESPONSE]");
    let preview: String = response.data.rag_response.chars().take(500).collect();
    println!("   {}", preview);
    if response.data.rag_response.len() > 500 {
        println!("   ... (truncated, see full response in export files)");
    }

    // Export Files Section - Download Links
    println!("\n{}", "=".repeat(70));
    println!("                       DOWNLOAD LINKS                                ");
    println!("{}", "=".repeat(70));

    let rst_path = Path::new(&response.export_files.rst_file);
    let org_path = Path::new(&response.export_files.org_file);
    let json_path = Path::new(&response.export_files.json_file);

    println!("\n[EXPORT FILES]");
    println!("   RST Report  : {}", response.export_files.rst_file);
    println!("   ORG Report  : {}", response.export_files.org_file);
    println!("   JSON Data   : {}", response.export_files.json_file);

    // File URI format for clickable links
    println!("\n[FILE LINKS (copy to browser)]");
    if let Ok(rst_abs) = std::fs::canonicalize(rst_path) {
        println!("   RST  : file:///{}", rst_abs.display().to_string().replace("\\", "/"));
    }
    if let Ok(org_abs) = std::fs::canonicalize(org_path) {
        println!("   ORG  : file:///{}", org_abs.display().to_string().replace("\\", "/"));
    }
    if let Ok(json_abs) = std::fs::canonicalize(json_path) {
        println!("   JSON : file:///{}", json_abs.display().to_string().replace("\\", "/"));
    }

    println!("\n{}", "=".repeat(70));
    println!("                    PIPELINE COMPLETED SUCCESSFULLY                  ");
    println!("{}", "=".repeat(70));
    println!();
}
