use agent_difusser::run as run_image;
use rag::run as run_rag;
use execute::run as run_execute;
use orchestrate_workflow::run as run_orchestrate_workflow;
use task::run as run_task;
use std::env;
use futures_util::{
    future::{ready, BoxFuture, FutureExt},
    TryFutureExt};
use log::{error, info, log_enabled, warn, Level, LevelFilter};
use tokio_postgres::Error;
use rag::secure::error;
use rag::shared::secureUtils::{ErrHandler, ErrMsg};
type ResultErrFramework<T> = std::result::Result<T, Error>;
use dotenv::dotenv;
use std::path::PathBuf;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;
use warp::{Filter, Reply};
use futures_util::StreamExt;

use rag::domain::models::llm::CmdBody;
use rag::rag_agent::agent_device::embedd_handler::quickThink;
// use crate::WebResult;

#[tokio::main]
async fn main() -> ResultErrFramework<()> {
    info!("Start Call Module");
    dotenv().ok();

    env_logger::init();
    log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("[RAG] pgd, Rag System Starting!");
    }
    info!("[RAG] pgd, Rag System running");

    if log_enabled!(Level::Debug) {
        info!("[RAG] welcome to Rag System");
        info!("[CMD] command ");
        info!("[CMD] cargo run --bin (show all agent option)");
        info!("[CMD] cargo run --bin rag -- args1");
        info!("[CMD] cargo run --bin rag -- rag");
        info!("[CMD] cargo run --bin rag -- rag_socket");
        info!("[CMD] cargo run --bin rag_socket (standalone)");
    }

    let r: Result<(), Error> = match env::args().nth(1).as_deref() {
        Some(arg) => run_check(arg).await,
        _           => Ok(()),
    };
    r;
    Ok(())
}

async fn run_check(arg: &str) -> ResultErrFramework<()> {
    info!("Running Args Parameter {:?}",arg);
    if arg == "rag" {
        info!("{} module start ", arg);
        ready(run_rag()).await.await;
        info!("{} rag module running", arg);
    }
    if arg == "image" {
        info!("{} module start ", arg);
        ready(run_image()).await.await;
        info!("{} module running", arg);
    }
    if arg == "task" {
        info!("{} module start ", arg);
        ready(run_task()).await.await;
        info!("{} module running", arg);
    }
    if arg == "execute" {
        info!("{} module start ",arg);
        ready(run_execute()).await.await;
        info!("{} module running", arg);
    }
    if arg == "workflow" {
        info!("{} module start ", arg);
        ready(run_orchestrate_workflow()).await.await;
        info!("{} module running", arg);
    }
    if arg == "rag_socket" {
        info!("{} module start ", arg);
        run_rag_socket().await;
        info!("{} module running", arg);
    }
    let vec_args = vec!["rag","image","task","execute","workflow","rag_socket"];
    if !vec_args.contains(&arg) { error!("Module call start not found args {} - Use \
    'rag' \
    'image' \
    'task' \
    'execute' \
    'workflow' \
    'rag_socket' ", arg); }

    Ok(())
}
//command:
//cargo run --bin (show all agent option)
//cargo run --bin rag -- args1
//cargo run --bin rag -- rag
//cargo run --bin rag -- rag_socket
//cargo run --bin rag_socket (standalone)
//

// ============== RAG Socket Server Types ==============
#[derive(Debug, Clone, Deserialize)]
pub struct RagRequest {
    pub user: String,
    pub rag_request: CmdBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub user: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub user: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJoin {
    pub user: String,
}

#[derive(Clone)]
struct SocketAppState {
    tx: Sender<ChatMessage>,
    history: Arc<RwLock<Vec<ChatMessage>>>,
    active_users: Arc<RwLock<HashMap<String, Instant>>>,
}

impl SocketAppState {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        SocketAppState {
            tx,
            history: Arc::new(RwLock::new(Vec::new())),
            active_users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn add_message(&self, message: ChatMessage) {
        self.history.write().push(message.clone());
        let mut history = self.history.write();
        if history.len() > 100 {
            history.remove(0);
        }
        let _ = self.tx.send(message);
    }

    fn update_user_activity(&self, user: &str) {
        self.active_users.write().insert(user.to_string(), Instant::now());
        let mut users = self.active_users.write();
        users.retain(|_, time| time.elapsed() < Duration::from_secs(300));
    }

    fn get_active_users(&self) -> Vec<String> {
        self.active_users.read().keys().cloned().collect()
    }
}

fn with_socket_state(
    state: SocketAppState,
) -> impl Filter<Extract = (SocketAppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn json_response<T: Serialize>(t: T) -> impl Reply {
    warp::reply::json(&t)
}

async fn run_rag_socket() {
    info!("[RAG_SOCKET] Starting RAG Socket Server!");

    let state = SocketAppState::new();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Authorization"]);

    let api_routes = {
        let state = state.clone();

        let send_message = warp::path("message")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_socket_state(state.clone()))
            .and_then(|message: UserMessage, state: SocketAppState| async move {
                let chat_message = ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    user: message.user.clone(),
                    content: message.content.clone(),
                    timestamp: Utc::now(),
                };
                state.add_message(chat_message.clone());
                state.update_user_activity(&message.user);
                Ok::<_, Infallible>(json_response(chat_message))
            });

        let get_messages = warp::path("messages")
            .and(warp::get())
            .and(with_socket_state(state.clone()))
            .and_then(|state: SocketAppState| async move {
                let history = state.history.read().clone();
                Ok::<_, Infallible>(json_response(history))
            });

        let get_active_users = warp::path("active-users")
            .and(warp::get())
            .and(with_socket_state(state.clone()))
            .and_then(|state: SocketAppState| async move {
                let users = state.get_active_users();
                Ok::<_, Infallible>(json_response(users))
            });

        let join_chat = warp::path("join")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_socket_state(state.clone()))
            .and_then(|user_join: UserJoin, state: SocketAppState| async move {
                state.update_user_activity(&user_join.user);
                let welcome_message = ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    user: "System".to_string(),
                    content: format!("{} has joined the chat!", user_join.user),
                    timestamp: Utc::now(),
                };
                state.add_message(welcome_message);
                Ok::<_, Infallible>(json_response(serde_json::json!({
                    "status": "joined",
                    "user": user_join.user
                })))
            });

        let ws_chat = warp::path("ws")
            .and(warp::ws())
            .and(with_socket_state(state.clone()))
            .map(|ws: warp::ws::Ws, state: SocketAppState| {
                ws.on_upgrade(|websocket| async move {
                    handle_websocket(websocket, state).await;
                })
            });


        let ws_rag = warp::path("rag")
            .and(warp::ws())
            .and(with_socket_state(state.clone()))
            .map(|ws: warp::ws::Ws, state: SocketAppState| {
                ws.on_upgrade(|websocket| async move {
                    handle_rag_websocket(websocket, state).await;
                })
            });

        send_message
            .or(get_messages)
            .or(get_active_users)
            .or(join_chat)
            .or(ws_chat)
            .or(ws_rag)
            .with(cors)
    };

    let health = warp::path!("health").map(|| "RAG Socket Server is running!");
    let routes = api_routes.or(health);

    println!("🔥 🚀 RAG Socket server starting on http://localhost:9091");
    println!("⚙️ Available endpoints:");
    println!("   POST   /join         - Join chat");
    println!("   POST   /message      - Send message");
    println!("   GET    /messages     - Get message history");
    println!("   GET    /active-users - Get active users");
    println!("   WS     /ws           - WebSocket for real-time chat");
    println!("   GET    /health       - Health check");
    println!("   WS     /rag          - WebSocket for real-time rag");

    warp::serve(routes).run(([0, 0, 0, 0], 9091)).await;
}

async fn handle_websocket(ws: warp::ws::WebSocket, state: SocketAppState) {
    let (mut tx, mut rx) = ws.split();
    let mut rx_broadcast = state.tx.subscribe();

    use futures_util::SinkExt;

    let history = state.history.read().clone();
    if let Ok(json) = serde_json::to_string(&history) {
        let _ = tx.send(warp::ws::Message::text(json)).await;
    }

    let state_clone = state.clone();
    tokio::task::spawn(async move {
        while let Some(result) = rx.next().await {
            if let Ok(msg) = result {
                if let Ok(text) = msg.to_str() {
                    if let Ok(user_msg) = serde_json::from_str::<UserMessage>(text) {
                        let chat_message = ChatMessage {
                            id: Uuid::new_v4().to_string(),
                            user: user_msg.user.clone(),
                            content: user_msg.content.clone(),
                            timestamp: Utc::now(),
                        };
                        state_clone.add_message(chat_message);
                        state_clone.update_user_activity(&user_msg.user);
                    }
                }
            }
        }
    });

    while let Ok(msg) = rx_broadcast.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            use futures_util::SinkExt;
            let _ = tx.send(warp::ws::Message::text(json)).await;
        }
    }
}


// RAG WebSocket handler that calls quickThink
async fn handle_rag_websocket(ws: warp::ws::WebSocket, _state: SocketAppState) {
    let (mut tx, mut rx) = ws.split();

    use futures_util::SinkExt;

    while let Some(result) = rx.next().await {
        if let Ok(msg) = result {
            if let Ok(text) = msg.to_str() {
                info!("[RAG_WS] Received message: {}", text);

                // Try to parse as RagRequest
                if let Ok(rag_req) = serde_json::from_str::<RagRequest>(text) {
                    info!("[RAG_WS] Processing RAG request for user: {}", rag_req.user);

                    // Call quickThink function
                    match quickThink(rag_req.user.clone(), rag_req.rag_request.clone()).await {
                        Ok(response) => {
                            // Extract the response body
                            let response_str = format!("{:?}", response.into_response());

                            // Create response message
                            let rag_response = serde_json::json!({
                                "type": "rag_response",
                                "user": rag_req.user,
                                "status": "success",
                                "response": response_str,
                                "timestamp": Utc::now()
                            });

                            if let Ok(json) = serde_json::to_string(&rag_response) {
                                let _ = tx.send(warp::ws::Message::text(json)).await;
                            }
                        }
                        Err(e) => {
                            error!("[RAG_WS] quickThink error: {:?}", e);

                            let error_response = serde_json::json!({
                                "type": "rag_response",
                                "user": rag_req.user,
                                "status": "error",
                                "error": format!("{:?}", e),
                                "timestamp": Utc::now()
                            });

                            if let Ok(json) = serde_json::to_string(&error_response) {
                                let _ = tx.send(warp::ws::Message::text(json)).await;
                            }
                        }
                    }
                } else {
                    // Try to parse as direct CmdBody (backward compatibility)
                    if let Ok(cmd_body) = serde_json::from_str::<CmdBody>(text) {
                        info!("[RAG_WS] Processing direct CmdBody request");

                        match quickThink("websocket_user".to_string(), cmd_body.clone()).await {
                            Ok(response) => {
                                let response_str = format!("{:?}", response.into_response());

                                let rag_response = serde_json::json!({
                                    "type": "rag_response",
                                    "user": "websocket_user",
                                    "status": "success",
                                    "response": response_str,
                                    "timestamp": Utc::now()
                                });

                                if let Ok(json) = serde_json::to_string(&rag_response) {
                                    let _ = tx.send(warp::ws::Message::text(json)).await;
                                }
                            }
                            Err(e) => {
                                error!("[RAG_WS] quickThink error: {:?}", e);

                                let error_response = serde_json::json!({
                                    "type": "rag_response",
                                    "user": "websocket_user",
                                    "status": "error",
                                    "error": format!("{:?}", e),
                                    "timestamp": Utc::now()
                                });

                                if let Ok(json) = serde_json::to_string(&error_response) {
                                    let _ = tx.send(warp::ws::Message::text(json)).await;
                                }
                            }
                        }
                    } else {
                        warn!("[RAG_WS] Invalid message format: {}", text);

                        let error_response = serde_json::json!({
                            "type": "error",
                            "message": "Invalid message format. Expected RagRequest or CmdBody",
                            "timestamp": Utc::now()
                        });

                        if let Ok(json) = serde_json::to_string(&error_response) {
                            let _ = tx.send(warp::ws::Message::text(json)).await;
                        }
                    }
                }
            }
        }
    }
}
