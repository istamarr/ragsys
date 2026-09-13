use serde_derive::{Deserialize, Serialize};
use anyhow::{Result, anyhow};


/**
* Prompt Command Request
**/
#[derive(Clone, Debug, Deserialize)]
pub struct LLMCreateBuildRequest {
    pub reqkey : String,
    pub message : String,
    pub body : LLMCreateBuild
}

#[derive(Clone, Debug, Deserialize)]
pub struct LLMCreateBuild{
    pub model : String,
    pub options : String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LLMRequest {
    pub baseUrl : String,
    pub path : String,
    pub method : String,
    pub isMultipart : bool,
    pub requestType : String,
    pub responseType : String,
    pub body :RequestBody
}

#[derive(Clone, Debug, Deserialize)]
pub struct RequestBody{
    pub model : String,
    pub prompt : String,
    pub options : String,
    pub keepAlive : String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CmdBody{
    pub model : String,
    pub prompt : String,
    pub cmd: String,
    pub tags: String,
    pub options : String,
    pub keepAlive : String,
}

/**
* File LLM
**/
#[derive(Serialize, Deserialize, Debug)]
pub struct GGUFFile {
    pub detections: Vec<Detection>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detection {
    pub class_name: String,
    pub confidence: f32,
    pub label: String,
}

/**
* Tags
**/
#[derive(Debug, Clone)]
pub struct ParsedTags {
    // doc_type: DocumentType,
    // app_name: AppName,
    pub(crate) doc_type: String,
    pub(crate) app_name: String,
    pub(crate) url: String,
    pub(crate) free_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    Fix,
    Correct,
    Flow,
    Code,
    Describe,
    Changes,
}

impl DocumentType {
    // pub fn from_str(s: &str) -> Result<Self> {
    //     match s.to_lowercase().as_str() {
    //         "fix" => Ok(DocumentType::Fix),
    //         "correct" => Ok(DocumentType::Correct),
    //         "flow" => Ok(DocumentType::Flow),
    //         "code" => Ok(DocumentType::Code),
    //         "describe" => Ok(DocumentType::Describe),
    //         "changes" => Ok(DocumentType::Changes),
    //         _ => Err(anyhow!("Invalid type '{}'. Valid options: fix, correct, flow, code, describe, changes", s)),
    //     }
    // }
    pub(crate) fn from_str(s: &str) -> Result<String> {
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

    pub fn as_str(&self) -> &'static str {
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

#[derive(Debug, Clone)]
pub enum AppName{
    App1,
    App2,
}

impl AppName {
    pub(crate) fn from_str(s: &str) -> Result<String> {
        match s.to_lowercase().as_str() {
            // "app1" => Ok(AppName::App1),
            // "app2" => Ok(AppName::App2),
            "app1" => Ok("App1".to_string()),
            "app2" => Ok("App2".to_string()),
            _ =>
                Ok(format!("Invalid appname '{:?}'. Valid options: app1, app2",s)),
            // Err(anyhow!("Invalid appname '{}'. Valid options: app1, app2", s)),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AppName::App1 => "app1",
            AppName::App2 => "app2",
        }
    }
}


