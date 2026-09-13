use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex}; // Changed: use tokio::sync::Mutex
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::JoinHandle;
use uuid::Uuid;

use orchestrate_workflow::domain::models::ag_o_w::{Action, ActionResult, Event};
use orchestrate_workflow::shared::sharedUtils::LLM;

#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    async fn execute(&self, action: &Action) -> Result<serde_json::Value, String>;
    fn name(&self) -> &'static str;
}

pub struct RpaAgent {
    adapters: HashMap<String, Arc<dyn Adapter>>,
    local_queue: Arc<Mutex<Vec<Action>>>,
    audit: Arc<Mutex<Vec<ActionResult>>>,
    event_tx: Sender<Event>,
    event_rx: Arc<Mutex<Receiver<Event>>>,
}

impl RpaAgent {
    pub fn new(eventtx: Sender<Event>, eventrx: Receiver<Event>) -> Self {
        Self {
            adapters: HashMap::new(),
            local_queue: Arc::new(Mutex::new(Vec::new())),
            audit: Arc::new(Mutex::new(Vec::new())),
            event_tx: eventtx,
            event_rx: Arc::new(Mutex::new(eventrx)),
        }
    }

    pub fn register_adapter<A: Adapter + 'static>(&mut self, kind: &str, adapter: A) {
        self.adapters.insert(kind.to_string(), Arc::new(adapter));
    }

    pub async fn enqueue_local(&self, action: Action) {
        let mut q = self.local_queue.lock().await; // Changed to .await
        q.push(action);
    }

    pub async fn publish(&self, ev: Event) {
        let _ = self.event_tx.send(ev).await;
    }

    pub fn run_loop(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let mut thread_state = false;

                // Lock and receive event
                let opt = {
                    let mut rx = self.event_rx.lock().await; // Changed to .await
                    rx.recv().await
                };

                if let Some(ev) = opt {
                    match ev {
                        Event::Action(action) => {
                            self.handle_action(action).await;
                        }
                        Event::ActionResult(ar) => {
                            let mut audit = self.audit.lock().await; // Changed to .await
                            audit.push(ar);
                            thread_state = true;
                        }
                    }
                } else {
                    // Channel closed
                    println!("Event channel closed, exiting run_loop");
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;

                // Process local queue if we just handled an ActionResult
                if thread_state {
                    let next = {
                        let mut q = self.local_queue.lock().await; // Changed to .await
                        q.pop()
                    };
                    if let Some(action) = next {
                        self.handle_action(action).await;
                    }
                }
            }
        })
    }

    async fn handle_action(&self, action: Action) {
        let traceid = Uuid::new_v4().to_string();
        let mut attempts = 0u8;
        let mut last_err = None;
        let mut success = false;
        let mut output = None;

        while attempts <= action.retry {
            attempts += 1;
            let adapter_opt = self.adapters.get(&action.kind).cloned();
            match adapter_opt {
                Some(adapter) => {
                    match adapter.execute(&action).await {
                        Ok(o) => {
                            success = true;
                            output = Some(o.clone());

                            let ar = ActionResult {
                                id: action.id.clone(),
                                success: true,
                                output: Some(o),
                                error: None,
                                trace_id: traceid.clone(),
                            };
                            let _ = self.publish(Event::ActionResult(ar)).await;
                            break;
                        }
                        Err(e) => {
                            last_err = Some(e);
                            sleep(Duration::from_millis(200u64 * attempts as u64)).await;
                        }
                    }
                }
                None => {
                    last_err = Some(format!("No adapter for kind '{}'", action.kind));
                    break;
                }
            }
        }

        if !success {
            let ar = ActionResult {
                id: action.id.clone(),
                success: false,
                output,
                error: last_err,
                trace_id: traceid,
            };
            let _ = self.publish(Event::ActionResult(ar)).await;
        }
    }

    pub async fn get_audit(&self) -> Vec<ActionResult> {
        let a = self.audit.lock().await; // Changed to .await
        a.clone()
    }
}

pub struct LlmCreatorAdapter {
    pub endpoint: String,
}

#[async_trait::async_trait]
impl Adapter for LlmCreatorAdapter {
    fn name(&self) -> &'static str { "llm_creator" }

    async fn execute(&self, action: &Action) -> Result<serde_json::Value, String> {
        let modelname = action.params.get("modelname")
            .and_then(|v| v.as_str()).unwrap_or("unnamed");
        sleep(Duration::from_millis(300)).await;

        Ok(serde_json::json!({
            "model": modelname,
            "artifacturi": format!("s3://models/{}/artifact.tar.gz", modelname),
            "status": "created"
        }))
    }
}

pub struct RagIngestAdapter {
    pub rag_endpoint: String,
}

#[async_trait::async_trait]
impl Adapter for RagIngestAdapter {
    fn name(&self) -> &'static str { "rag_ingest" }

    async fn execute(&self, action: &Action) -> Result<serde_json::Value, String> {
        let artifact = action.params.get("artifacturi")
            .and_then(|v| v.as_str()).ok_or("missing artifacturi".to_string())?;
        let docmeta = action.params.get("metadata").cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        sleep(Duration::from_millis(200)).await;

        Ok(serde_json::json!({
            "ingested": artifact,
            "indexid": Uuid::new_v4().to_string(),
            "metadata": docmeta
        }))
    }
}

#[tokio::main]
async fn main() {
    let (agenttx, agentrx) = mpsc::channel::<Event>(64);
    let mut rpa = RpaAgent::new(agenttx.clone(), agentrx);

    rpa.register_adapter("model-llm", LlmCreatorAdapter {
        endpoint: "http://10.253.247.105:9292/model".into()
    });
    rpa.register_adapter("ragingest", RagIngestAdapter {
        rag_endpoint: "http://10.253.247.105:9090/".into()
    });

    let rpa = Arc::new(rpa);
    let handle = rpa.clone().run_loop();

    let llm_action = Action {
        id: "create-model-1".into(),
        kind: "model-llm".into(),
        params: serde_json::json!({
            "modelname": "finance-model",
            "train_data": "s3://datasets/d1"
        }),
        idempotency_key: Some("create-finance-asist-v1".into()),
        retry: 3,
    };

    rpa.publish(Event::Action(llm_action)).await;

    sleep(Duration::from_millis(800)).await;

    let audits = rpa.get_audit().await; // Changed to .await
    assert!(audits.len() >= 1, "Expected at least one audit entry");

    let created = audits.iter()
        .find(|a| a.id == "create-model-1" && a.success)
        .expect("Expected successful model creation");

    let artifacturi = created.output.as_ref()
        .and_then(|o| o.get("artifacturi"))
        .and_then(|v| v.as_str())
        .expect("artifact missing");

    let rag_action = Action {
        id: "ingest-1".into(),
        kind: "ragingest".into(),
        params: serde_json::json!({
            "artifacturi": artifacturi,
            "metadata": {
                "source": "model-llm",
                "model": "finance-model"
            }
        }),
        idempotency_key: Some("ingest-finance-asist-v1".into()),
        retry: 2,
    };

    rpa.publish(Event::Action(rag_action)).await;

    sleep(Duration::from_millis(500)).await;

    let audits = rpa.get_audit().await; // Changed to .await
    let ingest_rec = audits.iter().find(|a| a.id == "ingest-1" && a.success);
    assert!(ingest_rec.is_some(), "Expected successful RAG ingest");

    println!("Workflow completed successfully!");
    handle.abort();
}