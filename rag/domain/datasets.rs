use serde_derive::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;
use parquet::file::reader::{FileReader, SerializedFileReader};
use std::collections::HashMap;
use std::env;
use bevy::reflect::TypeData;
use parquet::record::{Row, RowAccessor};
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct ReasoningChain {
//     pub root_cause: String,
//     pub flow: Vec<String>,
//     pub solver: String,
//     pub result: String,
// }
//
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Datasets {
//     pub title: String,
//     pub tags: Vec<String>,
//     pub components: Vec<String>,
//     pub attributes: Vec<String>,
//     pub interface: Vec<String>,
//     pub seq: String,
//     pub data: String,
//     pub source: String,
//     pub date: String,
//     pub similiarity_score: String,
//     pub precision_score: String,
//     pub troubleshot: String,
//     pub instruction_hint: String,
//     pub response_struct_result: String,
//     pub flow_name: String,
//     pub min_length: usize,
//     pub max_length: usize,
//     pub conversation_fields: (),
//     pub root_cause_fields: Vec<String>,
//     pub flow_fields: Vec<String>,
//     pub solver_fields: Vec<String>,
//     pub result_fields: Vec<String>,
//     pub auto_detect: bool,
//     pub force_reasoning: bool,
// }
//
// impl Default for Datasets {
//     fn default() -> Self {
//         Self {
//             title: "".to_string(),
//             tags: vec![
//                 "qdrant".to_string(),
//                 "embedding".to_string(),
//                 "pipeline".to_string(),
//                 "datatable".to_string(),
//             ],
//             components: vec![],
//             attributes: vec![],
//             interface: vec![],
//             seq: "".to_string(),
//             data: "".to_string(),
//             source: "".to_string(),
//             date: "".to_string(),
//             similiarity_score: "".to_string(),
//             precision_score: "".to_string(),
//             troubleshot: "".to_string(),
//             instruction_hint: "".to_string(),
//             response_struct_result: "".to_string(),
//             min_length: 10,
//             max_length: 8192,
//             flow_name: "".to_string(),
//             conversation_fields: (),
//
//             root_cause_fields: vec![
//                 "question".to_string(), "instruction".to_string(), "prompt".to_string(),
//                 "input".to_string(), "problem".to_string(), "query".to_string(),
//                 "text".to_string(), "content".to_string(), "human".to_string(),
//                 "user".to_string(), "system".to_string(),
//             ],
//             flow_fields: vec![
//                 "thinking".to_string(), "reasoning".to_string(), "steps".to_string(),
//                 "chain_of_thought".to_string(), "analysis".to_string(), "thoughts".to_string(),
//                 "rationale".to_string(), "explanation".to_string(),
//             ],
//             solver_fields: vec![
//                 "solution".to_string(), "method".to_string(), "approach".to_string(),
//                 "strategy".to_string(), "plan".to_string(),
//             ],
//             result_fields: vec![
//                 "answer".to_string(), "response".to_string(), "output".to_string(),
//                 "result".to_string(), "completion".to_string(), "assistant".to_string(),
//                 "gpt".to_string(), "bot".to_string(), "reply".to_string(),
//             ],
//
//             auto_detect: true,
//             force_reasoning: true,
//         }
//     }
// }
//
// pub struct DatasetParentsUnstructurer {
//     pub config: Datasets,
//     pub output_file: BufWriter<File>,
//     pub processed_count: usize,
//     pub skipped_count: usize,
//     pub detected_patterns: HashMap<String, usize>,
// }
//
// impl DatasetParentsUnstructurer {
//     pub fn new(output_path: &Path, config: Datasets) -> Result<Self> {
//         let file = OpenOptions::new()
//             .create(true)
//             .write(true)
//             .truncate(true)
//             .open(output_path)?;
//
//         let output_file = BufWriter::new(file);
//
//         Ok(Self {
//             config,
//             output_file,
//             processed_count: 0,
//             skipped_count: 0,
//             detected_patterns: HashMap::new(),
//         })
//     }
//
//     pub fn process_dataset(&mut self, input_path: &Path) -> Result<()> {
//         let extension = input_path
//             .extension()
//             .and_then(|s| s.to_str())
//             .unwrap_or("");
//
//         println!("Processing dataset: {}", input_path.display());
//         println!("Format detected: {}", extension);
//         println!("Auto-detect mode: {}", self.config.auto_detect);
//         println!("Force reasoning: {}", self.config.force_reasoning);
//
//         match extension {
//             "jsonl" | "json" => self.process_jsonl(input_path)?,
//             "parquet" => self.process_parquet(input_path)?,
//             "csv" => self.process_csv(input_path)?,
//             _ => return Err(anyhow!("Unsupported file format: {}", extension)),
//         }
//
//         self.output_file.flush()?;
//
//         println!("Processing Complete");
//         println!("Processed: {} Processed", self.processed_count);
//         println!("Skipped: {} Skipped", self.skipped_count);
//         println!("Detected Patterns");
//         for (pattern, count) in &self.detected_patterns {
//             println!(" - {}: {} times", pattern, count);
//         }
//
//         Ok(())
//     }
//
//     fn process_jsonl(&mut self, input_path: &Path) -> Result<()> {
//         let file = File::open(input_path)?;
//         let reader = BufReader::new(file);
//
//         for (line_num, line) in reader.lines().enumerate() {
//             let line = line?;
//
//             if line.trim().is_empty() {
//                 continue;
//             }
//
//             match serde_json::from_str::<Value>(&line) {
//                 Ok(json) => {
//                     if let Some(chain) = self.universal_extract(&json) {
//                         self.write_reasoning(&chain)?;
//                     } else {
//                         self.skipped_count += 1;
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
//                     self.skipped_count += 1;
//                 }
//             }
//
//             if (line_num + 1) % 1000 == 0 {
//                 println!("  Processed {} lines...", line_num + 1);
//             }
//         }
//
//         Ok(())
//     }
//
//     fn process_parquet(&mut self, input_path: &Path) -> Result<()> {
//         let file = File::open(input_path)?;
//         let reader = SerializedFileReader::new(file)?;
//
//         let metadata = reader.metadata();
//         let num_rows = metadata.file_metadata().num_rows();
//         println!("Total rows: {}", num_rows);
//
//         let mut row_iter = reader.get_row_iter(None)?;
//         let mut row_count = 0;
//
//         while let Some(record) = row_iter.next() {
//             let record = record;
//
//             let json_value = self.parquet_row_to_json(&record);
//             if let Some(chain) = self.universal_extract(&json_value) {
//                 self.write_reasoning(&chain)?;
//             } else {
//                 self.skipped_count += 1;
//             }
//
//             row_count += 1;
//             if row_count % 1000 == 0 {
//                 println!("Processed {} rows", row_count);
//             }
//         }
//
//         Ok(())
//     }
//
//     fn process_csv(&mut self, input_path: &Path) -> Result<()> {
//         let file = File::open(input_path)?;
//         let mut reader = csv::Reader::from_reader(file);
//
//         let headers = reader.headers()?.clone();
//         println!("CSV headers: {:?}", headers);
//
//         for (idx, result) in reader.records().enumerate() {
//             let record = result?;
//
//             let json_value = self.csv_row_to_json(&headers, &record);
//             if let Some(chain) = self.universal_extract(&json_value) {
//                 self.write_reasoning(&chain)?;
//             } else {
//                 self.skipped_count += 1;
//             }
//
//             if (idx + 1) % 1000 == 0 {
//                 println!("Processed {} rows", idx + 1);
//             }
//         }
//
//         Ok(())
//     }
//
//     fn universal_extract(&mut self, json: &Value) -> Option<ReasoningChain> {
//         if let Some(chain) = self.explicit_mapping(json) {
//             self.record_pattern("explicit_mapping");
//             return Some(chain);
//         }
//
//         if let Some(chain) = self.conversation_format(json) {
//             self.record_pattern("conversation_format");
//             return Some(chain);
//         }
//
//         if let Some(chain) = self.instruction_response(json) {
//             self.record_pattern("instruction_response");
//             return Some(chain);
//         }
//
//         if let Some(chain) = self.nested_extraction(json) {
//             self.record_pattern("nested_structure");
//             return Some(chain);
//         }
//
//         if let Some(chain) = self.key_value_analysis(json) {
//             self.record_pattern("key_value_analysis");
//             return Some(chain);
//         }
//
//         if self.config.force_reasoning {
//             if let Some(chain) = self.extract_with_reasoning(json) {
//                 self.record_pattern("forced_extraction");
//                 return Some(chain);
//             }
//         }
//
//         None
//     }
//
//     fn explicit_mapping(&self, json: &Value) -> Option<ReasoningChain> {
//         let root_cause = self.extract_field(json, &self.config.root_cause_fields)?;
//         let result = self.extract_field(json, &self.config.result_fields)?;
//
//         let flow = self.extract_flow_field(json);
//         let solver = self.extract_field(json, &self.config.solver_fields)
//             .unwrap_or_else(|| self.generate_solver(&root_cause, &flow));
//
//         Some(ReasoningChain {
//             root_cause,
//             flow,
//             solver,
//             result,
//         })
//     }
//
//     fn conversation_format(&self, json: &Value) -> Option<ReasoningChain> {
//         let conv_fields = vec!["messages",
//                                "conversation",
//                                "turns",
//                                "dialogue"];
//
//         for field in &conv_fields {
//             if let Some(messages) = json.get(field).and_then(|v| v.as_array()) {
//                 let mut user_msg = String::new();
//                 let mut assistant_msg = String::new();
//                 let mut thinking_steps = Vec::new();
//
//                 for msg in messages {
//                     let role = msg.get("role")
//                         .or_else(|| msg.get("from"))
//                         .and_then(|v| v.as_str())
//                         .unwrap_or("");
//
//                     let content = msg.get("content")
//                         .or_else(|| msg.get("value"))
//                         .or_else(|| msg.get("text"))
//                         .and_then(|v| v.as_str())
//                         .unwrap_or("");
//
//                     match role {
//                         "user" => {
//                             user_msg = content.to_string();
//                         }
//                         "ais" => {
//                             assistant_msg = content.to_string();
//                         }
//                         "system" => {//thinking
//                             thinking_steps.push(content.to_string());
//                         }
//                         _ => {}
//                     }
//                 }
//
//                 if !user_msg.is_empty() && !assistant_msg.is_empty() {
//                     return Some(ReasoningChain {
//                         root_cause: user_msg.clone(),
//                         flow: if thinking_steps.is_empty() {
//                             self.generate_flow_from_qa(&user_msg, &assistant_msg)
//                         } else {
//                             thinking_steps
//                         },
//                         solver: self.infer_solver_from_content(&user_msg, &assistant_msg),
//                         result: assistant_msg,
//                     });
//                 }
//             }
//         }
//
//         None
//     }
//
//     fn instruction_response(&self, json: &Value) -> Option<ReasoningChain> {
//         let inst_fields = ["instruction", "input", "prompt", "query"];
//         let resp_fields = ["output", "response", "answer", "completion"];
//
//         for inst_f in &inst_fields {
//             if let Some(instruction) = json.get(inst_f).and_then(|v| v.as_str()) {
//                 for resp_f in &resp_fields {
//                     if let Some(response) = json.get(resp_f).and_then(|v| v.as_str()) {
//                         return Some(ReasoningChain {
//                             root_cause: instruction.to_string(),
//                             flow: self.generate_flow_from_qa(instruction, response),
//                             solver: self.infer_solver_from_content(instruction, response),
//                             result: response.to_string(),
//                         });
//                     }
//                 }
//             }
//         }
//
//         None
//     }
//
//     fn nested_extraction(&self, json: &Value) -> Option<ReasoningChain> {
//         let mut texts = Vec::new();
//         self.extract_all_text_recursive(json, &mut texts, 0);
//
//         if texts.len() >= 2 {
//             let root_cause = texts[0].clone();
//             let result = texts[texts.len() - 1].clone();
//
//             let middle_texts: Vec<String> = texts[1..texts.len()-1]
//                 .iter()
//                 .map(|s| s.to_string())
//                 .collect();
//
//             let flow = if middle_texts.is_empty() {
//                 self.generate_flow_from_qa(&root_cause, &result)
//             } else {
//                 middle_texts
//             };
//
//             return Some(ReasoningChain {
//                 root_cause: root_cause.clone(),
//                 flow: flow.clone(),
//                 solver: self.generate_solver(&root_cause.clone(), &flow),
//                 result,
//             });
//         }
//
//         None
//     }
//
//     fn key_value_analysis(&self, json: &Value) -> Option<ReasoningChain> {
//         if let Some(obj) = json.as_object() {
//             let mut pairs: Vec<(String, String)> = obj.iter()
//                 .filter_map(|(k, v)| {
//                     v.as_str().map(|s| (k.clone(), s.to_string()))
//                 })
//                 .filter(|(_, v)| v.len() >= self.config.min_length)
//                 .collect();
//
//             if pairs.len() >= 2 {
//                 // Sort by semantic importance
//                 pairs.sort_by(|a, b| {
//                     let a_score = self.semantic_importance(&a.0);
//                     let b_score = self.semantic_importance(&b.0);
//                     b_score.partial_cmp(&a_score).unwrap()
//                 });
//
//                 let root_cause = pairs[0].1.clone();
//                 let result = pairs[1].1.clone();
//
//                 return Some(ReasoningChain {
//                     root_cause: root_cause.clone(),
//                     flow: self.generate_flow_from_qa(&root_cause, &result),
//                     solver: self.generate_solver(&root_cause, &[]),
//                     result,
//                 });
//             }
//         }
//
//         None
//     }
//
//     fn extract_with_reasoning(&self, json: &Value) -> Option<ReasoningChain> {
//         let mut all_text = Vec::new();
//         self.extract_all_text_recursive(json, &mut all_text, 0);
//
//         if all_text.is_empty() {
//             return None;
//         }
//
//         let combined_text = all_text.join(" ");
//         if combined_text.len() < self.config.min_length {
//             return None;
//         }
//
//         let (question, answer) = self.split_text_intelligently(&combined_text);
//         Some(ReasoningChain {
//             root_cause: question.clone(),
//             flow: self.generate_flow_from_qa(&question, &answer),
//             solver: self.infer_solver_from_content(&question, &answer),
//             result: answer,
//         })
//     }
// //helper
//     fn extract_field(&self, json: &Value, field_names: &[String]) -> Option<String> {
//         for field_name in field_names {
//             if let Some(value) = json.get(field_name) {
//                 if let Some(text) = value.as_str() {
//                     if text.trim().len() >= self.config.min_length {
//                         return Some(text.trim().to_string());
//                     }
//                 }
//             }
//         }
//         None
//     }
//
//     fn extract_flow_field(&self, json: &Value) -> Vec<String> {
//         for field_name in &self.config.flow_fields {
//             if let Some(value) = json.get(field_name) {
//                 if let Some(arr) = value.as_array() {
//                     let steps: Vec<String> = arr.iter()
//                         .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
//                         .filter(|s| !s.is_empty())
//                         .collect();
//                     if !steps.is_empty() {
//                         return steps;
//                     }
//                 }
//                 if let Some(text) = value.as_str() {
//                     return self.split_into_steps(text);
//                 }
//             }
//         }
//         vec![]
//     }
//
//     fn split_into_steps(&self, text: &str) -> Vec<String> {
//         text.lines()
//             .map(|line| line.trim())
//             .filter(|line| !line.is_empty())
//             .map(|line| {
//                 line.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ':' || c == '-')
//                     .trim()
//                     .to_string()
//             })
//             .filter(|s| !s.is_empty())
//             .collect()
//     }
//
//     fn extract_all_text_recursive(&self, value: &Value, texts: &mut Vec<String>, depth: usize) {
//         if depth > 5 {
//             return;
//         }
//
//         match value {
//             Value::String(s) if s.len() >= self.config.min_length => {
//                 texts.push(s.trim().to_string());
//             }
//             Value::Array(arr) => {
//                 for item in arr {
//                     self.extract_all_text_recursive(item, texts, depth + 1);
//                 }
//             }
//             Value::Object(obj) => {
//                 for (_key, val) in obj {
//                     self.extract_all_text_recursive(val, texts, depth + 1);
//                 }
//             }
//             _ => {}
//         }
//     }
//
//     fn generate_flow_from_qa(&self, question: &str, answer: &str) -> Vec<String> {
//         let mut steps = Vec::new();
//         let q_lower = question.to_lowercase();
//         if q_lower.contains("how") {
//             steps.push("Identify".to_string());
//             steps.push("Sequential steps".to_string());
//         } else if q_lower.contains("why") {
//             steps.push("Analyze causes".to_string());
//             steps.push("Explain".to_string());
//         } else if q_lower.contains("what") {
//             steps.push("Define".to_string());
//             steps.push("Provide".to_string());// detail and context
//         } else if q_lower.contains("code") || q_lower.contains("implement") {
//             steps.push("Requirements".to_string());
//             steps.push("Design".to_string());
//             steps.push("Implement".to_string());
//         } else {
//             steps.push("Parse question components".to_string());
//             steps.push("Apply knowledge".to_string());
//         }
//
//         steps.push("Analizer answer".to_string());
//         steps
//     }
//
//     fn infer_solver_from_content(&self, question: &str, answer: &str) -> String {
//         let q_lower = question.to_lowercase();
//         let a_lower = answer.to_lowercase();
//
//         if q_lower.contains("code") || a_lower.contains("```") || a_lower.contains("function") {
//             "Code implementation".to_string()
//         } else if q_lower.contains("calculate") || q_lower.contains("math") {
//             "Computation".to_string()
//         } else if q_lower.contains("explain") || q_lower.contains("describe") {
//             "Analytical".to_string()
//         } else if q_lower.contains("compare") || q_lower.contains("difference") {
//             "Comparative".to_string()
//         } else if q_lower.contains("debug") || q_lower.contains("fix") {
//             "Troubleshooting".to_string()
//         } else {
//             "Systematic".to_string()
//         }
//     }
//
//     fn generate_solver(&self, root_cause: &str, flow: &[String]) -> String {
//         if !flow.is_empty() {
//             format!("Multi-step ({} stages)", flow.len())
//         } else {
//             self.infer_solver_from_content(root_cause, "")
//         }
//     }
//
//     fn semantic_importance(&self, key: &str) -> f32 {
//         let important_keywords = [
//             ("question", 10.0), ("instruction", 10.0), ("input", 9.0),
//             ("answer", 10.0), ("response", 10.0), ("output", 9.0),
//             ("content", 7.0), ("text", 6.0), ("data", 5.0),
//         ];
//
//         let key_lower = key.to_lowercase();
//         for (keyword, score) in &important_keywords {
//             if key_lower.contains(keyword) {
//                 return *score;
//             }
//         }
//         1.0
//     }
//
//     fn split_text_intelligently(&self, text: &str) -> (String, String) {
//         let mid = text.len() / 2;
//         let split_point = text[..mid].rfind(['.', '?', '!', '\n'])
//             .map(|p| p + 1)
//             .unwrap_or(mid);
//
//         let question = text[..split_point].trim().to_string();
//         let answer = text[split_point..].trim().to_string();
//
//         (question, answer)
//     }
//
//     fn record_pattern(&mut self, pattern: &str) {
//         *self.detected_patterns.entry(pattern.to_string()).or_insert(0) += 1;
//     }
//
//     fn parquet_row_to_json(&self, row: &parquet::record::Row) -> Value {
//         let mut map = serde_json::Map::new();
//
//         for (name, field) in row.get_column_iter() {
//             if let Some(text) = self.field_to_string(field) {
//                 map.insert(name.clone(), Value::String(text));
//             }
//         }
//
//         Value::Object(map)
//     }
//
//     fn field_to_string(&self, field: &parquet::record::Field) -> Option<String> {
//         match field {
//             parquet::record::Field::Str(s) => Some(s.clone()),
//             parquet::record::Field::Bytes(b) => {
//                 String::from_utf8(b.data().to_vec()).ok()
//             }
//             _ => None,
//         }
//     }
//
//     fn csv_row_to_json(&self, headers: &csv::StringRecord, row: &csv::StringRecord) -> Value {
//         let mut map = serde_json::Map::new();
//
//         for (idx, value) in row.iter().enumerate() {
//             if let Some(header) = headers.get(idx) {
//                 map.insert(header.to_string(), Value::String(value.to_string()));
//             }
//         }
//
//         Value::Object(map)
//     }
//
//     fn write_reasoning(&mut self, chain: &ReasoningChain) -> Result<()> {
//         if chain.root_cause.len() < self.config.min_length ||
//             chain.result.len() < self.config.min_length {
//             self.skipped_count += 1;
//             return Ok(());
//         }
//
//         let total_length = chain.root_cause.len() +
//             chain.flow.iter().map(|s| s.len()).sum::<usize>() +
//             chain.solver.len() +
//             chain.result.len();
//
//         if total_length > self.config.max_length {
//             self.skipped_count += 1;
//             return Ok(());
//         }
//
//         // Output format
//         let output = serde_json::json!({
//             "root_cause": chain.root_cause,
//             "flow": chain.flow,
//             "solver": chain.solver,
//             "result": chain.result,
//             "formatted": format!(
//                 "<root_cause>\n{}\n</root_cause>\n\n<flow>\n{}\n</flow>\n\n<solver>\n{}\n</solver>\n\n<result>\n{}\n</result>",
//                 chain.root_cause,
//                 chain.flow.iter()
//                     .enumerate()
//                     .map(|(i, step)| format!("{}. {}", i + 1, step))
//                     .collect::<Vec<_>>()
//                     .join("\n"),
//                 chain.solver,
//                 chain.result
//             )
//         });
//
//         writeln!(self.output_file, "{}", output)?;
//         self.processed_count += 1;
//
//         Ok(())
//     }
// }

// ---
// pub async fn example_usage() -> Result<()> {
//     // Configuration
//     let mut config = Datasets {
//         title: "My Reasoning Dataset".to_string(),
//         source: "HuggingFace".to_string(),
//         flow_name: "reasoning_pipeline".to_string(),
//
//         enable_qdrant: true,
//         qdrant_url: "http://localhost:6334".to_string(),
//         qdrant_collection: "reasoning_dataset".to_string(),
//         vector_size: 384,
//
//         auto_detect: true,
//         force_reasoning: true,
//         min_length: 10,
//         max_length: 8192,
//
//         ..Default::default()
//     };
//
//     let output_path = Path::new("output_reasoning.jsonl");
//     let mut processor = DatasetParentsUnstructurer::new(output_path, config)?;
//     processor.init_qdrant().await?;
//     // processor.process_dataset(Path::new("dataset.jsonl")).await?;
//     processor.process_parquet_directory(Path::new("./parquet_datasets")).await?;
//
//     Ok(())
// }
// --
use qdrant_client::{qdrant::{
    vectors_config::Config, CreateCollection, Distance, PointStruct, UpsertPointsBuilder,
    VectorParams, VectorsConfig, Value as QdrantValue, Struct as QdrantStruct,
}, Qdrant};
use uuid::Uuid;

// Universal reasoning chain structure
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReasoningChain {
    pub root_cause: String,
    pub flow: Vec<String>,
    pub solver: String,
    pub result: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Datasets {
    pub title: String,
    pub tags: Vec<String>,
    pub components: Vec<String>,
    pub attributes: Vec<String>,
    pub interface: Vec<String>,
    pub seq: String,
    pub data: String,
    pub source: String,
    pub date: String,
    pub similiarity_score: String,
    pub precision_score: String,
    pub troubleshot: String,
    pub instruction_hint: String,
    pub response_struct_result: String,
    pub flow_name: String,
    pub min_length: usize,
    pub max_length: usize,
    pub conversation_fields: (),

    pub root_cause_fields: Vec<String>,
    pub flow_fields: Vec<String>,
    pub solver_fields: Vec<String>,
    pub result_fields: Vec<String>,
    pub auto_detect: bool,  // Enable automatic structure detection
    pub force_reasoning: bool,  // Generate reasoning even if not present
    pub qdrant_url: String,
    pub qdrant_collection: String,
    pub vector_size: usize,
    pub enable_qdrant: bool,
}

impl Default for Datasets {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            tags: vec![
                "qdrant".to_string(),
                "embedding".to_string(),
                "pipeline".to_string(),
                "datatable".to_string(),
            ],
            components: vec![],
            attributes: vec![],
            interface: vec![],
            seq: "".to_string(),
            data: "".to_string(),
            source: "".to_string(),
            date: "".to_string(),
            similiarity_score: "".to_string(),
            precision_score: "".to_string(),
            troubleshot: "".to_string(),
            instruction_hint: "".to_string(),
            response_struct_result: "".to_string(),
            min_length: 10,
            max_length: 8192,
            flow_name: "".to_string(),
            conversation_fields: (),

            root_cause_fields: vec![
                "question".to_string(), "instruction".to_string(), "prompt".to_string(),
                "input".to_string(), "problem".to_string(), "query".to_string(),
                "text".to_string(), "content".to_string(), "human".to_string(),
                "user".to_string(), "system".to_string(),
            ],
            flow_fields: vec![
                "thinking".to_string(), "reasoning".to_string(), "steps".to_string(),
                "chain_of_thought".to_string(), "analysis".to_string(), "thoughts".to_string(),
                "rationale".to_string(), "explanation".to_string(),
            ],
            solver_fields: vec![
                "solution".to_string(), "method".to_string(), "approach".to_string(),
                "strategy".to_string(), "plan".to_string(),
            ],
            result_fields: vec![
                "answer".to_string(), "response".to_string(), "output".to_string(),
                "result".to_string(), "completion".to_string(), "assistant".to_string(),
                "gpt".to_string(), "bot".to_string(), "reply".to_string(),
            ],

            auto_detect: true,
            force_reasoning: true,

            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".to_string()).to_string(),
            qdrant_collection: "reasoning_dataset".to_string(),
            vector_size: 384,  // Default for all-MiniLM-L6-v2
            enable_qdrant: false,
        }
    }
}



pub struct DatasetParentsUnstructurer {
    pub config: Datasets,
    pub output_file: BufWriter<File>,
    pub processed_count: usize,
    pub skipped_count: usize,
    pub detected_patterns: HashMap<String, usize>,
    pub qdrant_client: Option<Qdrant>,
    pub batch_buffer: Vec<ReasoningChain>,
    pub batch_size: usize,
}

impl DatasetParentsUnstructurer {
    pub fn new(output_path: &Path, config: Datasets) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)?;

        let output_file = BufWriter::new(file);

        Ok(Self {
            config,
            output_file,
            processed_count: 0,
            skipped_count: 0,
            detected_patterns: HashMap::new(),
            qdrant_client: None,
            batch_buffer: Vec::new(),
            batch_size: 100,  // Upsert 100 at a time
        })
    }

    pub fn clear_process(&mut self, param:&str) -> Result<()> {
        self.processed_count = 0;
        self.process_dataset("".as_ref());
        Ok(())
    }

    pub async fn init_qdrant(&mut self) -> Result<()> {
        if !self.config.enable_qdrant {
            return Ok(());
        }

        println!("Connecting to Qdrant at: {}", self.config.qdrant_url);
        let client = Qdrant::from_url(&self.config.qdrant_url).build()?;
        // let client = Qdrant::from(&self.config.qdrant_url).build()?;

        let collections = client.list_collections().await?;
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.config.qdrant_collection);

        if !collection_exists {
            println!("Creating collection: {}", self.config.qdrant_collection);
                client.create_collection(CreateCollection {
                    collection_name: self.config.qdrant_collection.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: self.config.vector_size as u64,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                }).await?;
            println!("[OK] Collection created successfully");
        } else {
            println!("[OK] Collection already exists");
        }

        self.qdrant_client = Some(client);
        Ok(())
    }

    pub async fn process_dataset(&mut self, input_path: &Path) -> Result<()> {
        let extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        println!("Processing dataset: {}", input_path.display());
        println!("Format detected: {}", extension);
        println!("Auto-detect mode: {}", self.config.auto_detect);
        println!("Force reasoning: {}", self.config.force_reasoning);
        println!("Qdrant enabled: {}", self.config.enable_qdrant);

        match extension {
            "jsonl" | "json" => self.process_jsonl(input_path).await?,
            "parquet" => self.process_parquet(input_path).await?,
            "csv" => self.process_csv(input_path).await?,
            _ => return Err(anyhow!("Unsupported file format: {}", extension)),
        }

        // self.flush_batch().await?;
        self.output_file.flush()?;

        println!(" Processed: {} Processed", self.processed_count);
        println!(" Skipped: {} Skipped", self.skipped_count);
        for (pattern, count) in &self.detected_patterns {
            println!(" - {}: {} times", pattern, count);
        }

        self.clear_process("");

        Ok(())
    }

    pub async fn process_parquet_directory(&mut self, dir_path: &Path) -> Result<()> {
        println!("Scanning directory: {}", dir_path.display());

        let mut parquet_files = Vec::new();

        if dir_path.is_dir() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "parquet" {
                            parquet_files.push(path);
                        }
                    }
                }
            }
        } else {
            return Err(anyhow!("Path is not a directory: {}", dir_path.display()));
        }

        println!("Found {} parquet files", parquet_files.len());
        for (idx, file_path) in parquet_files.iter().enumerate() {
            println!("\n[{}/{}] Processing: {}", idx + 1, parquet_files.len(), file_path.display());
            self.process_parquet(file_path).await?;
        }

        // self.flush_batch().await?;
        self.output_file.flush()?;

        println!(" Total files processed: {}", parquet_files.len());
        println!(" Total samples: {}", self.processed_count);
        println!(" Total skipped: {}", self.skipped_count);

        self.clear_process("");

        Ok(())
    }

    async fn process_jsonl(&mut self, input_path: &Path) -> Result<()> {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&line) {
                Ok(json) => {
                    if let Some(chain) = self.universal_extract(&json) {
                        self.write_reasoning_sample(&chain)?;
                    } else {
                        self.skipped_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    self.skipped_count += 1;
                }
            }

            if (line_num + 1) % 1000 == 0 {
                println!("  Processed {} lines...", line_num + 1);
            }
        }

        Ok(())
    }

    async fn process_parquet(&mut self, input_path: &Path) -> Result<()> {
        let file = File::open(input_path)?;
        let reader = SerializedFileReader::new(file)?;

        let metadata = reader.metadata();
        let num_rows = metadata.file_metadata().num_rows();
        println!("  Total rows: {}", num_rows);

        let mut row_iter = reader.get_row_iter(None)?;
        let mut row_count = 0;

        while let Some(record) = row_iter.next() {
            let unwrapped_record = record.unwrap();
            let json_value = self.parquet_row_to_json(&unwrapped_record);//expected `&Result<Row>`, but found `&parquet::errors::Result<Row>`
            if let Some(chain) = self.universal_extract(&json_value) {
                self.write_reasoning_sample(&chain)?;
            } else {
                self.skipped_count += 1;
            }

            row_count += 1;
            if row_count % 1000 == 0 {
                println!("  Processed {} rows...", row_count);
            }
        }

        Ok(())
    }

    async fn process_csv(&mut self, input_path: &Path) -> Result<()> {
        let file = File::open(input_path)?;
        let mut reader = csv::Reader::from_reader(file);

        let headers = reader.headers()?.clone();
        println!("CSV headers: {:?}", headers);

        for (idx, result) in reader.records().enumerate() {
            let record = result?;

            let json_value = self.csv_row_to_json(&headers, &record);
            if let Some(chain) = self.universal_extract(&json_value) {
                self.write_reasoning_sample(&chain)?;
            } else {
                self.skipped_count += 1;
            }

            if (idx + 1) % 1000 == 0 {
                println!("Processed {} rows...", idx + 1);
            }
        }

        Ok(())
    }

    fn universal_extract(&mut self, json: &Value) -> Option<ReasoningChain> {
        if let Some(chain) = self.explicit_mapping(json) {
            self.record_pattern("explicit_mapping");
            return Some(chain);
        }

        if let Some(chain) = self.conversation_format(json) {
            self.record_pattern("conversation_format");
            return Some(chain);
        }

        if let Some(chain) = self.instruction_response(json) {
            self.record_pattern("instruction_response");
            return Some(chain);
        }

        if let Some(chain) = self.nested_extraction(json) {
            self.record_pattern("nested_structure");
            return Some(chain);
        }

        if let Some(chain) = self.key_value_analysis(json) {
            self.record_pattern("key_value_analysis");
            return Some(chain);
        }

        if self.config.force_reasoning {
            if let Some(chain) = self.force_extract_with_reasoning(json) {
                self.record_pattern("forced_extraction");
                return Some(chain);
            }
        }

        None
    }

    fn explicit_mapping(&self, json: &Value) -> Option<ReasoningChain> {
        let root_cause = self.extract_field(json, &self.config.root_cause_fields)?;
        let result = self.extract_field(json, &self.config.result_fields)?;

        let flow = self.extract_flow_field(json);
        let solver = self.extract_field(json, &self.config.solver_fields)
            .unwrap_or_else(|| self.generate_solver(&root_cause, &flow));

        Some(ReasoningChain {
            root_cause,
            flow,
            solver,
            result,
        })
    }

    fn conversation_format(&self, json: &Value) -> Option<ReasoningChain> {
        let conv_fields = vec!["messages", "conversation", "turns", "dialogue"];

        for field in &conv_fields {
            if let Some(messages) = json.get(field).and_then(|v| v.as_array()) {
                let mut user_msg = String::new();
                let mut assistant_msg = String::new();
                let mut thinking_steps = Vec::new();

                for msg in messages {
                    let role = msg.get("role")
                        .or_else(|| msg.get("from"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let content = msg.get("content")
                        .or_else(|| msg.get("value"))
                        .or_else(|| msg.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    match role {
                        "user" | "question" => {
                            user_msg = content.to_string();
                        }
                        "assistant" | "ais" | "answer" => {
                            assistant_msg = content.to_string();
                        }
                        "system" | "thinking" => {
                            thinking_steps.push(content.to_string());
                        }
                        _ => {}
                    }
                }

                if !user_msg.is_empty() && !assistant_msg.is_empty() {
                    let flow = if thinking_steps.is_empty() {
                        self.generate_flow_from_qa(&user_msg, &assistant_msg)
                    } else {
                        thinking_steps
                    };

                    let solver = self.infer_solver_from_content(&user_msg, &assistant_msg);

                    return Some(ReasoningChain {
                        root_cause: user_msg,
                        flow,
                        solver,
                        result: assistant_msg,
                    });
                }
            }
        }

        None
    }

    fn instruction_response(&self, json: &Value) -> Option<ReasoningChain> {
        let inst_fields = ["instruction", "input", "prompt", "query"];
        let resp_fields = ["output", "response", "answer", "completion"];

        for inst_f in &inst_fields {
            if let Some(instruction) = json.get(inst_f).and_then(|v| v.as_str()) {
                for resp_f in &resp_fields {
                    if let Some(response) = json.get(resp_f).and_then(|v| v.as_str()) {
                        return Some(ReasoningChain {
                            root_cause: instruction.to_string(),
                            flow: self.generate_flow_from_qa(instruction, response),
                            solver: self.infer_solver_from_content(instruction, response),
                            result: response.to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    fn nested_extraction(&self, json: &Value) -> Option<ReasoningChain> {
        let mut texts = Vec::new();
        self.extract_all_text_recursive(json, &mut texts, 0);

        if texts.len() >= 2 {
            let root_cause = texts[0].clone();
            let result = texts[texts.len() - 1].clone();

            let middle_texts: Vec<String> = texts[1..texts.len()-1]
                .iter()
                .map(|s| s.to_string())
                .collect();

            let flow = if middle_texts.is_empty() {
                self.generate_flow_from_qa(&root_cause, &result)
            } else {
                middle_texts
            };

            let solver = self.generate_solver(&root_cause, &flow);

            return Some(ReasoningChain {
                root_cause,
                flow,
                solver,
                result,
            });
        }

        None
    }

    fn key_value_analysis(&self, json: &Value) -> Option<ReasoningChain> {
        if let Some(obj) = json.as_object() {
            let mut pairs: Vec<(String, String)> = obj.iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| (k.clone(), s.to_string()))
                })
                .filter(|(_, v)| v.len() >= self.config.min_length)
                .collect();

            if pairs.len() >= 2 {
                pairs.sort_by(|a, b| {
                    let a_score = self.semantic_importance(&a.0);
                    let b_score = self.semantic_importance(&b.0);
                    b_score.partial_cmp(&a_score).unwrap()
                });

                let root_cause = pairs[0].1.clone();
                let result = pairs[1].1.clone();

                return Some(ReasoningChain {
                    root_cause: root_cause.clone(),
                    flow: self.generate_flow_from_qa(&root_cause, &result),
                    solver: self.generate_solver(&root_cause, &[]),
                    result,
                });
            }
        }

        None
    }

    fn force_extract_with_reasoning(&self, json: &Value) -> Option<ReasoningChain> {
        let mut all_text = Vec::new();
        self.extract_all_text_recursive(json, &mut all_text, 0);

        if all_text.is_empty() {
            return None;
        }

        let combined_text = all_text.join(" ");
        if combined_text.len() < self.config.min_length {
            return None;
        }
        let (question, answer) = self.split_text_intelligently(&combined_text);
        Some(ReasoningChain {
            root_cause: question.clone(),
            flow: self.generate_flow_from_qa(&question, &answer),
            solver: self.infer_solver_from_content(&question, &answer),
            result: answer,
        })
    }

    fn extract_field(&self, json: &Value, field_names: &[String]) -> Option<String> {
        for field_name in field_names {
            if let Some(value) = json.get(field_name) {
                if let Some(text) = value.as_str() {
                    if text.trim().len() >= self.config.min_length {
                        return Some(text.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_flow_field(&self, json: &Value) -> Vec<String> {
        for field_name in &self.config.flow_fields {
            if let Some(value) = json.get(field_name) {
                if let Some(arr) = value.as_array() {
                    let steps: Vec<String> = arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !steps.is_empty() {
                        return steps;
                    }
                }
                if let Some(text) = value.as_str() {
                    return self.split_into_steps(text);
                }
            }
        }
        vec![]
    }

    fn split_into_steps(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ':' || c == '-')
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn extract_all_text_recursive(&self, value: &Value, texts: &mut Vec<String>, depth: usize) {
        if depth > 5 {
            return;
        }

        match value {
            Value::String(s) if s.len() >= self.config.min_length => {
                texts.push(s.trim().to_string());
            }
            Value::Array(arr) => {
                for item in arr {
                    self.extract_all_text_recursive(item, texts, depth + 1);
                }
            }
            Value::Object(obj) => {
                for (_key, val) in obj {
                    self.extract_all_text_recursive(val, texts, depth + 1);
                }
            }
            _ => {}
        }
    }

    fn generate_flow_from_qa(&self, question: &str, answer: &str) -> Vec<String> {
        let mut steps = Vec::new();
        let q_lower = question.to_lowercase();
        if q_lower.contains("how") {
            steps.push("Identify".to_string());
            steps.push("Break down".to_string());
        } else if q_lower.contains("why") {
            steps.push("Analyze".to_string());
            steps.push("Explain".to_string());
        } else if q_lower.contains("what") {
            steps.push("Define".to_string());
            steps.push("Provide".to_string());//context
        } else if q_lower.contains("code") || q_lower.contains("implement") {
            steps.push("Requirements".to_string());
            steps.push("Design".to_string());
            steps.push("Implement".to_string());
        } else {
            steps.push("Parse Prompt".to_string());
            steps.push("Apply IMplement".to_string());
        }

        steps.push("Implement Result".to_string());
        steps
    }

    fn infer_solver_from_content(&self, question: &str, answer: &str) -> String {
        let q_lower = question.to_lowercase();
        let a_lower = answer.to_lowercase();

        if q_lower.contains("code") || a_lower.contains("```") || a_lower.contains("function") {
            "Programming".to_string()
        } else if q_lower.contains("calculate") || q_lower.contains("math") {
            "Computation".to_string()
        } else if q_lower.contains("explain") || q_lower.contains("describe") {
            "Analytical".to_string()
        } else if q_lower.contains("compare") || q_lower.contains("difference") {
            "Comparative".to_string()
        } else if q_lower.contains("debug") || q_lower.contains("fix") {
            "Troubleshooting".to_string()
        } else {
            "Systematic".to_string()
        }
    }

    fn generate_solver(&self, root_cause: &str, flow: &[String]) -> String {
        if !flow.is_empty() {
            format!("Multi-step approach ({} stages)", flow.len())
        } else {
            self.infer_solver_from_content(root_cause, "")
        }
    }

    fn semantic_importance(&self, key: &str) -> f32 {
        let important_keywords = [
            ("question", 10.0), ("instruction", 10.0), ("input", 9.0),
            ("answer", 10.0), ("response", 10.0), ("output", 9.0),
            ("content", 7.0), ("text", 6.0), ("data", 5.0),
        ];

        let key_lower = key.to_lowercase();
        for (keyword, score) in &important_keywords {
            if key_lower.contains(keyword) {
                return *score;
            }
        }
        1.0
    }

    fn split_text_intelligently(&self, text: &str) -> (String, String) {
        let mid = text.len() / 2;
        let split_point = text[..mid].rfind(['.', '?', '!', '\n'])
            .map(|p| p + 1)
            .unwrap_or(mid);

        let question = text[..split_point].trim().to_string();
        let answer = text[split_point..].trim().to_string();

        (question, answer)
    }

    fn record_pattern(&mut self, pattern: &str) {
        *self.detected_patterns.entry(pattern.to_string()).or_insert(0) += 1;
    }

    fn parquet_row_to_json(&self, row: &Row) -> Value {//expected `&Result<Row>`, but found `&parquet::errors::Result<Row>`
        let mut map = serde_json::Map::new();
        let data_clone = row.clone(); // Creates another Arc pointing to the same Mutex
        {
            let reignite_row = row;
        }
        let owned_row = row.clone();
        for (name, field) in owned_row.get_column_iter() {
            if let Some(text) = self.field_to_string(&field) {
                map.insert(name.clone(), Value::String(text));
            }
        }

        Value::Object(map)
    }

    fn field_to_string(&self, field: &parquet::record::Field) -> Option<String> {
        match field {
            parquet::record::Field::Str(s) => Some(s.clone()),
            parquet::record::Field::Bytes(b) => {
                String::from_utf8(b.data().to_vec()).ok()
            }
            _ => None,
        }
    }

    fn csv_row_to_json(&self, headers: &csv::StringRecord, row: &csv::StringRecord) -> Value {
        let mut map = serde_json::Map::new();

        for (idx, value) in row.iter().enumerate() {
            if let Some(header) = headers.get(idx) {
                map.insert(header.to_string(), Value::String(value.to_string()));
            }
        }

        Value::Object(map)
    }

    fn write_reasoning_sample(&mut self, chain: &ReasoningChain) -> Result<()> {
        if chain.root_cause.len() < self.config.min_length ||
            chain.result.len() < self.config.min_length {
            self.skipped_count += 1;
            return Ok(());
        }

        let total_length = chain.root_cause.len() +
            chain.flow.iter().map(|s| s.len()).sum::<usize>() +
            chain.solver.len() +
            chain.result.len();

        if total_length > self.config.max_length {
            self.skipped_count += 1;
            return Ok(());
        }

        let output = serde_json::json!({
            "root_cause": chain.root_cause,
            "flow": chain.flow,
            "solver": chain.solver,
            "result": chain.result,
            "formatted": format!(
                "<root_cause>\n{}\n</root_cause>\n\n<flow>\n{}\n</flow>\n\n<solver>\n{}\n</solver>\n\n<result>\n{}\n</result>",
                chain.root_cause,
                chain.flow.iter()
                    .enumerate()
                    .map(|(i, step)| format!("{}. {}", i + 1, step))
                    .collect::<Vec<_>>()
                    .join("\n"),
                chain.solver,
                chain.result
            )
        });

        writeln!(self.output_file, "{}", output)?;
        self.processed_count += 1;

        Ok(())
    }
}
