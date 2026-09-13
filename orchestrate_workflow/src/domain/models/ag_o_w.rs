use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub kind: String,
    pub params: serde_json::Value,
    pub idempotency_key: Option<String>,
    pub retry: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct ActionResult {
    pub id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    Action(Action),
    ActionResult(ActionResult),
    // ApprovalRequested
}
