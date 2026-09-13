// use std::fs;
// use serde::{Deserialize, Serialize};
// use std::io::{self, Write};
// use crate::global::GLOBAL_ARRAY;
//
// pub fn get_value(key: &str) -> Option<String> {
//     let array = GLOBAL_ARRAY.lock().unwrap();
//     for (k, v) in array.iter() {
//         if k == key {
//             return Some(v.clone());
//         }
//     }
//     None
// }
//
// pub fn getLabelTag() -> String {
//     if let Some(value) = get_value("key1") {
//         return value;
//     } else {
//         println!("Value not found for the given key");
//         return "Default Value".to_string(); // Return a default value or an empty string
//     }
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct GGUFFile {
//     detections: Vec<Detection>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct Detection {
//     class_name: String,
//     confidence: f32,
//     label: String,
// }
//
// pub(crate) fn load_gguf_file(filepath: &str) -> Result<GGUFFile, Box<dyn std::error::Error>> {
//     let file_content = fs::read(filepath)?;
//     let gguf: GGUFFile = bincode::deserialize(&file_content)?;
//     Ok(gguf)
// }
//
// fn parse_question(question: &str, detections: &[(String, f32, String)]) -> Option<String> {
//     for word in question.split_whitespace() {
//         for (class_name, _, _) in detections {
//             if word.eq_ignore_ascii_case(class_name) {
//                 return Some(class_name.clone());
//             }
//         }
//     }
//     None
// }
//
// fn generate_response(detections: &[(String, f32, String)], class_name: &str) -> String {
//     for (name, _, label) in detections {
//         if name == class_name {
//             return format!("The label for '{}' is '{}'", name, label);
//         }
//     }
//     "Label not found.".to_string()
// }
//
// fn process_detections(detections: &[Detection]) -> Vec<(String, f32, String)> {
//     detections
//         .iter()
//         .map(|d| (d.class_name.clone(), d.confidence, d.label.clone()))
//         .collect()
// }
//
// pub(crate) fn handleQsAsr(question: &str) -> String {
//     let gguf_file_path = "model/llm_pgd_v1.gguf";
//     let gguf_data = match load_gguf_file(gguf_file_path) {
//         Ok(data) => data,
//         Err(e) => {
//             println!("Failed to load gguf file: {}", e);
//             return "Error loading file".to_string();
//         }
//     };
//
//     let label_result = getLabelTag();
//     let input_question = label_result;
//     let response = handle_model(input_question, &gguf_data.detections);
//     println!("Bot: {}", response);
//     response
// }
//
// fn handle_model(question: String, detections: &[Detection]) -> String {
//     let processed_data = process_detections(detections);
//     if let Some(class_name) = parse_question(question.trim(), &processed_data) {
//         generate_response(&processed_data, &class_name)
//     } else {
//         "I cannot understand the tag question.".to_string()
//     }
// }
//
//
