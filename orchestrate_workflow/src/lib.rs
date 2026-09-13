use std::collections::HashMap;
use std::sync::Arc;
use dotenv::dotenv;
use warp::Rejection;
use crate::secure::error::Error;
use crate::secure::error;
use wasm_bindgen::prelude::*;
use crate::domain::models::user::User;
use crate::shared::secureUtils::ErrHandler;
pub mod secure;
pub mod shared;
pub mod domain;
pub mod modules;

type Result<T> = std::result::Result<T, error::Error>;
type ResultInclude<T> = std::result::Result<T, ErrHandler>;
type ResultErrFramework<T> = std::result::Result<T, Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

pub async fn run() -> ResultErrFramework<()> {
    dotenv().ok();
    println!("Running Orchestration and Workflow...");
    Ok(())
}

use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    UserCreated { user_id: u32, email: String },
    UserUpdated { user_id: u32 },
    OrderPlaced { order_id: u32, user_id: u32, amount: f64 },
    PaymentProcessed { order_id: u32, success: bool },
    NotificationSent { user_id: u32, message: String },
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Event, event_bus: Arc<EventBus>);
}

#[derive(Clone)]
pub struct EventBus {
    sender: mpsc::UnboundedSender<(Event, Arc<EventBus>)>,
    handlers: Arc<tokio::sync::RwLock<Vec<Arc<dyn EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Arc<Self> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<(Event, Arc<EventBus>)>();
        let handlers: Arc<tokio::sync::RwLock<Vec<Arc<dyn EventHandler>>>> = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        let handlers_clone = handlers.clone();

        let event_bus = Arc::new(Self { sender, handlers });
        let event_bus_clone = event_bus.clone();

        tokio::spawn(async move {
            while let Some((event, bus)) = receiver.recv().await {
                let handlers = handlers_clone.read().await;
                for handler in handlers.iter() {
                    handler.handle(event.clone(), bus.clone()).await;
                }
            }
        });

        event_bus
    }

    pub async fn publish(&self, event: Event) {
        let bus_arc = Arc::new(EventBus {
            sender: self.sender.clone(),
            handlers: self.handlers.clone(),
        });
        let _ = self.sender.send((event, bus_arc));
    }

    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }
}

pub struct EventBusV2 {
    sender: mpsc::UnboundedSender<Event>,
    handlers: Arc<tokio::sync::RwLock<Vec<Arc<dyn EventHandlerV2>>>>,
}

#[async_trait::async_trait]
pub trait EventHandlerV2: Send + Sync {
    async fn handle(&self, event: Event);
}

impl EventBusV2 {
    pub fn new() -> Arc<Self> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Event>();
        let handlers: Arc<tokio::sync::RwLock<Vec<Arc<dyn EventHandlerV2>>>> = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        let handlers_clone = handlers.clone();

        let event_bus = Arc::new(Self { sender, handlers });

        // Spawn background task to process events
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let handlers = handlers_clone.read().await;
                for handler in handlers.iter() {
                    handler.handle(event.clone()).await;
                }
            }
        });

        event_bus
    }

    pub async fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub async fn subscribe(&self, handler: Arc<dyn EventHandlerV2>) {
        self.handlers.write().await.push(handler);
    }
}