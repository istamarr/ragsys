use std::sync::{Arc};
use orchestrate_workflow::domain::models::ag_o_w::{Action};
use orchestrate_workflow::modules::agent_o_wflow_module::{HttpAdapter, RpaAgent};

#[tokio::main]
async fn main() {
    println!("Starting Agent Ocs WFlow");

    let mut rpa = RpaAgent::new();
    rpa.register_adapter("http", HttpAdapter{ base: "https://localhost/rag/rpa_agent".into() });

    let arc = Arc::new(rpa);
    let runner = arc.clone();
    tokio::spawn(async move {
        runner.run_loop().await;
    });

    arc.enqueue(Action {
        id: "a_number1".into(),
        kind: "http".into(),
        params: serde_json::json!({"pipeline": "/rag"}),
        idempotency_key: None,
        retry: 2,
    });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("Log: {:#?}", arc.get_audit());
}
