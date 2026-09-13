//agent workflow and orchestrate:
//rpa, feedback and execute, and pipelines
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use tokio::time::{sleep, Duration};
use crate::domain::models::ag_o_w::{Action, ActionResult};

#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    async fn execute(&self, action: &Action) -> Result<serde_json::Value, String>;
    fn name(&self) -> &'static str;
}

pub struct RpaAgent {
    adapters: HashMap<String, Arc<dyn Adapter>>,
    // nanti pakai redis
    queue: Arc<Mutex<Vec<Action>>>,
    audit: Arc<Mutex<Vec<ActionResult>>>,
}

impl RpaAgent {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            queue: Arc::new(Mutex::new(Vec::new())),
            audit: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_adapter<A: Adapter + 'static>(&mut self, kind: &str, adapter: A) {
        self.adapters.insert(kind.to_string(), Arc::new(adapter));
    }

    pub fn enqueue(&self, action: Action) {
        let mut q = self.queue.lock().unwrap();
        q.push(action);
    }

    pub async fn run_loop(self: Arc<Self>) {
        loop {
            let next = {
                let mut q = self.queue.lock().unwrap();
                q.pop()
            };

            if let Some(action) = next {
                let agent = self.clone();
                tokio::spawn(async move {
                    let traceid = Uuid::new_v4().to_string();
                    let mut attempts = 0u8;
                    let mut last_err = None;
                    let mut success = false;
                    let mut output = None;

                    while attempts <= action.retry {
                        attempts += 1;
                        let adapter_opt = agent.adapters.get(&action.kind).cloned();
                        match adapter_opt {
                            Some(adapter) => {
                                match adapter.execute(&action).await {
                                    Ok(o) => {
                                        success = true;
                                        output = Some(o);
                                        break;
                                    }
                                    Err(e) => {
                                        last_err = Some(e);
                                        // backoff
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

                    let result = ActionResult {
                        id: action.id.clone(),
                        success,
                        output,
                        error: last_err,
                        trace_id: traceid,
                    };

                    let mut audit = agent.audit.lock().unwrap();
                    audit.push(result);
                });
            } else {
                // idle sleep
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    pub fn get_audit(&self) -> Vec<ActionResult> {
        let a = self.audit.lock().unwrap();
        a.clone()
    }
}

pub struct HttpAdapter {
    pub base: String,
}

#[async_trait::async_trait]
impl Adapter for HttpAdapter {
    fn name(&self) -> &'static str { "http" }

    async fn execute(&self, action: &Action) -> Result<serde_json::Value, String> {
        // reqwest and map params into JSON/URL, handle auth
        Ok(serde_json::json!({"postedto": self.base, "action_id": action.id}))
    }
}


