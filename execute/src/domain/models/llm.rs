use serde_derive::{Deserialize, Serialize};

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
