use crate::data::chunks::prep_module::ProcessedData;
use crate::rag_agent::handler::flow_code::fixco_llm::fixco_module;

// fixco
pub fn prep_module_q_prompt_fixco(results: &[(&str, ProcessedData)], list_q_payload: &str) -> Result<String, anyhow::Error>  {
    let list_q_available = vec![
        list_q_payload,
    ];

    for (i, question) in list_q_available.iter().enumerate() {
        println!("{}. {}", i + 1, question);
    }
    println!();

    let result = "";
    for question in &list_q_available {
        println!("CMDPrompt: {}", question);
        println!("{}", "-".repeat(60));
        let result = generic_q_prompt_fixco(question, results);
        println!("Result: {}", result);
        println!();
    }

    Ok(result.to_string())
}

pub fn extract_keywords_fixco(question: &str) -> Vec<String> {//common ke utils
    let stop_words = vec![
        "create", "make", "code", "fix", "fixing", "build", "boilerplate",
        "review", "command", "program", "programming", "dev", "devel", "development",
        "app", "application", "backend", "frontend", "debugging", "error",
        "fullstack", "source", "ui", "sql", "query",
        "linter", "git", "css", "html", "rust", "python", "c", "c++",
        "ruby", "js", "javascript", "syntax", "hardcode", "algorithm"
    ];//total ke bahasa indo, "java", "go", "kotlin"
    question.split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty() && !stop_words.contains(&word.as_str()))
        .collect()
}

pub fn extract_keywords_fixco_lang(question: &str) -> Vec<String> {//common ke utils
    let stop_words = vec![
        "rust", "python", "c", "c++",
        "ruby", "js", "javascript",
    ];//total ke bahasa indo, "java", "go", "kotlin"
    question.split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty() && !stop_words.contains(&word.as_str()))
        .collect()
}

//make in utils
fn generic_q_prompt_fixco(question: &str, results: &[(&str, ProcessedData)]) -> String {
    let question_lower = question.to_lowercase();
    let keywords = extract_keywords_fixco(&question_lower);
    let keywords_lang = extract_keywords_fixco_lang(&question_lower);

    //make in utils
    let mut relevant_chunks = Vec::new();
    for (file_path, data) in results {
        for chunk in &data.chunks {
            let chunk_lower = chunk.content.to_lowercase();
            let relevance_score = crate::data::chunks::prep_module::calculate_enhanced_relevance(&chunk_lower, &keywords, &question_lower);

            if relevance_score > 0 {
                relevant_chunks.push((file_path, chunk, relevance_score));
            }
        }
    }
    relevant_chunks.sort_by(|a, b| b.2.cmp(&a.2));

    if relevant_chunks.is_empty() {
        return "No relevant information found in the parsed documents.".to_string();
    }
    let (source_file, best_chunk, score) = &relevant_chunks[0];

    // let answer = crate::rag_agent::chunks::prep_module::extract_generic_q_prompt_result(&question_lower, &best_chunk.content, &keywords);

    println!("integrated to fixco_pipeline ");

    // let mut code_result = "".to_string();

    let code_result = fixco_module(keywords_lang);


    format!("{:?} (Source: {:?}, Relevance: {:?})",
            code_result.unwrap().to_string(), source_file, score)
}
