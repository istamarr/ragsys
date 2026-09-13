// //! check meta data , compare datetime, updated package contains files and folder.
// use reqwest::Client;
// use serde::Serialize;
// use crate::shared::sharedUtils;

// #[derive(Serialize)]
// struct AiAnalysisRequest {
//     prompt: String,
// }

// async fn pipe_package_update(changes: &[Change], ai_api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let client = Client::new();

//     // Format changes as a readable summary
//     let change_summary = format_changes_for_ai(changes);

//     let prompt = format!(
//         "extention|metadata|datetime|count folder|count files|weights|{}",
//         change_summary
//     );//Risk level (Low/Medium/High)\n\4. Specific classes or files that developers should review\n\n\

//     let response = client
//         .post("pipe_model_serve")
//         .header("Authorization", format!("Bearer {}", ai_api_key))
//         .json(&serde_json::json!({
//             "model": sharedUtils::LLM::PEDIA_ASIST_LM.code,
//             "messages": [
//                 {"role": "system", "content": "analyze JAR/WAR package differences summary"},
//                 {"role": "user", "content": prompt}
//             ],
//             "temperature": 0.3
//         }))
//         .send()
//         .await?;

//     let body = response.json::<serde_json::Value>().await?;
//     Ok(body["choices"][0]["message"]["content"].as_str().unwrap_or("No analysis").to_string())
// }

// Logo / emblem generation has been moved to:
//   agent_difusser/src/handler/dfsr_emblem.rs
//
// Call `dfsr_emblem::ensure_emblem_files(dir, name, EmblemSource::Vector)`
// or   `dfsr_emblem::ensure_emblem_files_ai(dir, name, prompt, url, key).await`

fn main() {}