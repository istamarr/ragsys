// Response_RAG_AGP (Response RAG to Model Context and or A2A)
use serde_derive::{Deserialize, Serialize};
use crate::data::chunks::prep_module::{ProcessedData, ProcessedDataNumber};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRagAgp{//process data
    pub type_response: String,
    pub status: String,
    pub content: ProcessedDataNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRagAgpSrv{//service umum
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTokenRagAgp{
    pub uid: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTokenRagAgp{
    pub token: String,
}
