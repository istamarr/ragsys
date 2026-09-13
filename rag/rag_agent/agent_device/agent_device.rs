// FIWARE Agent Handler for RFID to Qdrant Integration
// Handles FIWARE NGSI-LD/NGSIv2 RFID entities and stores them in local Qdrant vector database

use actix_web::http::StatusCode;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
// use qdrant_client::prelude::*;
use qdrant_client::qdrant::{CreateCollection, Distance, PointStruct, SearchPoints, SearchPointsBuilder, VectorParams, VectorsConfig, Filter, Condition, FieldCondition, Match, vectors_config::Config, Value as QdrantValue, PointId, QueryPointsBuilder};
use qdrant_client::Qdrant;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value, Value};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::Reply;

use crate::domain::models::llm::CmdBody;
use crate::data::result_data::saveLogDetailKnowledge::upsert_to_qdrant;
use crate::shared::helperUtils::{current_time, srv_response};
use crate::shared::secureUtils::generate_api_key;
use crate::WebResult;

// ============================================================================
// FIWARE CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiwareConfig {
    pub context_broker_url: String,
    pub fiware_service: String,
    pub fiware_service_path: String,
    pub subscription_endpoint: String,
    pub timeout_seconds: u64,
    pub poll_interval_ms: u64,
}

impl Default for FiwareConfig {
    fn default() -> Self {
        Self {
            context_broker_url: env::var("FIWARE_CB_URL")
                .unwrap_or_else(|_| "http://localhost:1026".to_string()),
            fiware_service: env::var("FIWARE_SERVICE")
                .unwrap_or_else(|_| "openiot".to_string()),
            fiware_service_path: env::var("FIWARE_SERVICE_PATH")
                .unwrap_or_else(|_| "/".to_string()),
            subscription_endpoint: env::var("FIWARE_SUBSCRIPTION_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8090/fiware/notify".to_string()),
            timeout_seconds: 30,
            poll_interval_ms: 5000,
        }
    }
}

// ============================================================================
// FIWARE NGSI-LD / NGSIv2 ENTITY STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgsiEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    #[serde(flatten)]
    pub attributes: HashMap<String, NgsiAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NgsiAttribute {
    Simple(Value),
    Structured {
        #[serde(rename = "type")]
        attr_type: Option<String>,
        value: Value,
        metadata: Option<HashMap<String, Value>>,
    },
}

impl NgsiAttribute {
    pub fn get_value(&self) -> &Value {
        match self {
            NgsiAttribute::Simple(v) => v,
            NgsiAttribute::Structured { value, .. } => value,
        }
    }
}

// RFID-specific FIWARE entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiwareRfidEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub rfid_uid: String,
    pub tag_type: String,
    pub location: Option<FiwareGeoLocation>,
    pub timestamp: DateTime<Utc>,
    pub read_count: u32,
    pub signal_strength: Option<f64>,
    pub reader_id: Option<String>,
    pub zone: Option<String>,
    pub status: String,
    pub metadata: HashMap<String, String>,
    pub raw_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiwareGeoLocation {
    #[serde(rename = "type")]
    pub geo_type: String,
    pub coordinates: Vec<f64>,
}

// FIWARE Subscription structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiwareSubscription {
    pub description: String,
    pub subject: SubscriptionSubject,
    pub notification: SubscriptionNotification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttling: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSubject {
    pub entities: Vec<EntityPattern>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<SubscriptionCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "idPattern", skip_serializing_if = "Option::is_none")]
    pub id_pattern: Option<String>,
    #[serde(rename = "type")]
    pub entity_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionCondition {
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionNotification {
    pub http: NotificationHttp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<Vec<String>>,
    #[serde(rename = "attrsFormat", skip_serializing_if = "Option::is_none")]
    pub attrs_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationHttp {
    pub url: String,
}

// Notification payload from FIWARE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiwareNotification {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
    pub data: Vec<NgsiEntity>,
}

// ============================================================================
// QDRANT VECTOR CONFIGURATION
// ============================================================================

const FIWARE_RFID_COLLECTION: &str = "fiware_rfid_tags";
const VECTOR_DIMENSION: u64 = 384; // nomic-embed-text dimension

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidVectorPayload {
    pub fiware_id: String,
    pub rfid_uid: String,
    pub tag_type: String,
    pub location_lat: Option<f64>,
    pub location_lon: Option<f64>,
    pub zone: String,
    pub reader_id: String,
    pub status: String,
    pub timestamp: String,
    pub read_count: u32,
    pub signal_strength: f64,
    pub raw_text: String,
}

// ============================================================================
// FIWARE AGENT HANDLER
// ============================================================================

pub struct FiwareAgentHandler {
    config: FiwareConfig,
    http_client: HttpClient,
    qdrant_client: Arc<RwLock<Option<Qdrant>>>,
    entity_cache: Arc<RwLock<HashMap<String, FiwareRfidEntity>>>,
}

impl FiwareAgentHandler {
    pub fn new(config: FiwareConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| HttpClient::new());

        Self {
            config,
            http_client,
            qdrant_client: Arc::new(RwLock::new(None)),
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize Qdrant connection and collection
    pub async fn init_qdrant(&self) -> Result<()> {
        let qdrant_url = env::var("QDRANT_URL")
            .unwrap_or_else(|_| "http://localhost:6334".to_string());

        info!("[FIWARE] Connecting to Qdrant at {}", qdrant_url);

        let client = Qdrant::from_url(&*qdrant_url)
            .build()
            .map_err(|e| anyhow!("Failed to create Qdrant client: {}", e))?;

        // Check if collection exists, create if not
        let collections = client.list_collections().await
            .map_err(|e| anyhow!("Failed to list collections: {}", e))?;

        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == FIWARE_RFID_COLLECTION);

        if !collection_exists {
            info!("[FIWARE] Creating collection: {}", FIWARE_RFID_COLLECTION);

            client.create_collection(
                qdrant_client::qdrant::CreateCollectionBuilder::new(FIWARE_RFID_COLLECTION)
                    .vectors_config(VectorParams {
                        size: VECTOR_DIMENSION,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })
            ).await
            .map_err(|e| anyhow!("Failed to create collection: {}", e))?;

            info!("[FIWARE] Collection {} created successfully", FIWARE_RFID_COLLECTION);
        }

        let mut qdrant_lock = self.qdrant_client.write().await;
        *qdrant_lock = Some(client);

        info!("[FIWARE] Qdrant initialized successfully");
        Ok(())
    }

    /// Create subscription for RFID entities in FIWARE Context Broker
    pub async fn create_rfid_subscription(&self) -> Result<String> {
        let subscription = FiwareSubscription {
            description: "RFID Tag updates subscription for RAG system".to_string(),
            subject: SubscriptionSubject {
                entities: vec![
                    EntityPattern {
                        id: None,
                        id_pattern: Some("urn:ngsi-ld:RFIDTag:.*".to_string()),
                        entity_type: "RFIDTag".to_string(),
                    },
                    EntityPattern {
                        id: None,
                        id_pattern: Some("urn:ngsi-ld:Asset:.*".to_string()),
                        entity_type: "Asset".to_string(),
                    },
                ],
                condition: Some(SubscriptionCondition {
                    attrs: vec![
                        "rfidUID".to_string(),
                        "location".to_string(),
                        "status".to_string(),
                        "readCount".to_string(),
                    ],
                }),
            },
            notification: SubscriptionNotification {
                http: NotificationHttp {
                    url: self.config.subscription_endpoint.clone(),
                },
                attrs: Some(vec![
                    "rfidUID".to_string(),
                    "tagType".to_string(),
                    "location".to_string(),
                    "zone".to_string(),
                    "readerId".to_string(),
                    "status".to_string(),
                    "readCount".to_string(),
                    "signalStrength".to_string(),
                ]),
                attrs_format: Some("normalized".to_string()),
            },
            expires: None,
            throttling: Some(1),
        };

        let url = format!("{}/v2/subscriptions", self.config.context_broker_url);

        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Fiware-Service", &self.config.fiware_service)
            .header("Fiware-ServicePath", &self.config.fiware_service_path)
            .json(&subscription)
            .send()
            .await
            .context("Failed to create FIWARE subscription")?;

        if response.status().is_success() {
            let subscription_id = response
                .headers()
                .get("Location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split('/').last().unwrap_or("unknown").to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            info!("[FIWARE] Subscription created: {}", subscription_id);
            Ok(subscription_id)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!("Failed to create subscription: {}", error_text))
        }
    }

    /// Query RFID entities from FIWARE Context Broker
    pub async fn query_rfid_entities(&self, entity_type: &str) -> Result<Vec<NgsiEntity>> {
        let url = format!(
            "{}/v2/entities?type={}&options=keyValues",
            self.config.context_broker_url, entity_type
        );

        let response = self.http_client
            .get(&url)
            .header("Fiware-Service", &self.config.fiware_service)
            .header("Fiware-ServicePath", &self.config.fiware_service_path)
            .send()
            .await
            .context("Failed to query FIWARE entities")?;

        if response.status().is_success() {
            let entities: Vec<NgsiEntity> = response.json().await
                .context("Failed to parse FIWARE entities")?;
            info!("[FIWARE] Retrieved {} entities of type {}", entities.len(), entity_type);
            Ok(entities)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow!("Failed to query entities: {}", error_text))
        }
    }

    /// Process notification from FIWARE (webhook handler)
    pub async fn process_notification(&self, notification: FiwareNotification) -> Result<usize> {
        info!("[FIWARE] Processing notification with {} entities", notification.data.len());

        let mut processed_count = 0;

        for entity in notification.data {
            match self.entity_to_rfid(&entity) {
                Ok(rfid_entity) => {
                    // Store in Qdrant
                    if let Err(e) = self.store_rfid_to_qdrant(&rfid_entity).await {
                        error!("[FIWARE] Failed to store entity {}: {}", entity.id, e);
                        continue;
                    }

                    // Update cache
                    let mut cache = self.entity_cache.write().await;
                    cache.insert(rfid_entity.rfid_uid.clone(), rfid_entity);

                    processed_count += 1;
                }
                Err(e) => {
                    warn!("[FIWARE] Failed to convert entity {}: {}", entity.id, e);
                }
            }
        }

        info!("[FIWARE] Processed {} entities successfully", processed_count);
        Ok(processed_count)
    }

    /// Convert NGSI entity to RFID entity
    fn entity_to_rfid(&self, entity: &NgsiEntity) -> Result<FiwareRfidEntity> {
        let rfid_uid = self.extract_string_attr(entity, "rfidUID")
            .or_else(|| self.extract_string_attr(entity, "rfid_uid"))
            .unwrap_or_else(|| entity.id.clone());

        let tag_type = self.extract_string_attr(entity, "tagType")
            .or_else(|| self.extract_string_attr(entity, "tag_type"))
            .unwrap_or_else(|| "Unknown".to_string());

        let location = self.extract_location(entity);

        let read_count = self.extract_number_attr(entity, "readCount")
            .or_else(|| self.extract_number_attr(entity, "read_count"))
            .unwrap_or(1) as u32;

        let signal_strength = self.extract_float_attr(entity, "signalStrength")
            .or_else(|| self.extract_float_attr(entity, "signal_strength"));

        let reader_id = self.extract_string_attr(entity, "readerId")
            .or_else(|| self.extract_string_attr(entity, "reader_id"));

        let zone = self.extract_string_attr(entity, "zone");

        let status = self.extract_string_attr(entity, "status")
            .unwrap_or_else(|| "active".to_string());

        let timestamp = self.extract_timestamp(entity)
            .unwrap_or_else(Utc::now);

        Ok(FiwareRfidEntity {
            id: entity.id.clone(),
            entity_type: entity.entity_type.clone(),
            rfid_uid,
            tag_type,
            location,
            timestamp,
            read_count,
            signal_strength,
            reader_id,
            zone,
            status,
            metadata: HashMap::new(),
            raw_data: None,
        })
    }

    /// Extract string attribute from entity
    fn extract_string_attr(&self, entity: &NgsiEntity, attr_name: &str) -> Option<String> {
        entity.attributes.get(attr_name).and_then(|attr| {
            match attr.get_value() {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            }
        })
    }

    /// Extract numeric attribute from entity
    fn extract_number_attr(&self, entity: &NgsiEntity, attr_name: &str) -> Option<i64> {
        entity.attributes.get(attr_name).and_then(|attr| {
            attr.get_value().as_i64()
        })
    }

    /// Extract float attribute from entity
    fn extract_float_attr(&self, entity: &NgsiEntity, attr_name: &str) -> Option<f64> {
        entity.attributes.get(attr_name).and_then(|attr| {
            attr.get_value().as_f64()
        })
    }

    /// Extract location from entity
    fn extract_location(&self, entity: &NgsiEntity) -> Option<FiwareGeoLocation> {
        entity.attributes.get("location").and_then(|attr| {
            let value = attr.get_value();
            if let Some(obj) = value.as_object() {
                let geo_type = obj.get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Point")
                    .to_string();

                let coordinates = obj.get("coordinates")
                    .or_else(|| obj.get("value").and_then(|v| v.as_object()).and_then(|o| o.get("coordinates")))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_f64())
                            .collect::<Vec<f64>>()
                    })
                    .unwrap_or_default();

                if coordinates.len() >= 2 {
                    return Some(FiwareGeoLocation { geo_type, coordinates });
                }
            }
            None
        })
    }

    /// Extract timestamp from entity
    fn extract_timestamp(&self, entity: &NgsiEntity) -> Option<DateTime<Utc>> {
        entity.attributes.get("timestamp")
            .or_else(|| entity.attributes.get("dateModified"))
            .or_else(|| entity.attributes.get("dateObserved"))
            .and_then(|attr| {
                attr.get_value().as_str().and_then(|s| {
                    DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                })
            })
    }

    /// Store RFID entity to Qdrant vector database
    pub async fn store_rfid_to_qdrant(&self, rfid_entity: &FiwareRfidEntity) -> Result<()> {
        let log = "store_rfid_to_qdrant".to_string();
        info!("{}",log.clone());

        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        // Generate embedding vector for the RFID entity
        let text_representation = self.rfid_to_text(rfid_entity);
        let embedding = self.generate_embedding(&text_representation).await?;

        // Create payload
        let (lat, lon) = rfid_entity.location.as_ref()
            .map(|l| (l.coordinates.get(1).copied(), l.coordinates.get(0).copied()))
            .unwrap_or((None, None));

        let payload = RfidVectorPayload {
            fiware_id: rfid_entity.id.clone(),
            rfid_uid: rfid_entity.rfid_uid.clone(),
            tag_type: rfid_entity.tag_type.clone(),
            location_lat: lat,
            location_lon: lon,
            zone: rfid_entity.zone.clone().unwrap_or_default(),
            reader_id: rfid_entity.reader_id.clone().unwrap_or_default(),
            status: rfid_entity.status.clone(),
            timestamp: rfid_entity.timestamp.to_rfc3339(),
            read_count: rfid_entity.read_count,
            signal_strength: rfid_entity.signal_strength.unwrap_or(0.0),
            raw_text: text_representation,
        };

        // Convert payload to Qdrant format
        let payload_json = serde_json::to_value(&payload)
            .context("Failed to serialize payload")?;

        let qdrant_payload: HashMap<String, QdrantValue> = payload_json
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), json_to_qdrant_value(v)))
                    .collect()
            })
            .unwrap_or_default();

        // Create point ID from RFID UID hash
        // let point_id = uuid::Uuid::new_v8(
        //     &uuid::Uuid::NAMESPACE_OID,
        //     rfid_entity.rfid_uid.as_bytes()
        // );
        //buat format di atas:
        let point_id = generate_api_key();

        let point = PointStruct {
            id: Some(PointId::from(point_id.to_string())),
            vectors: Some(embedding.clone().into()),
            payload: qdrant_payload,
        };

        // Upsert point to Qdrant: masih digabung di master qdrant data
        // client.upsert_points(FIWARE_RFID_COLLECTION, None, vec![point], None)
        //     .await
        //     .map_err(|e| anyhow!("Failed to upsert point: {}", e))?;

        let collection_name = env::var("ASIST_MASTER").context("ASIST_MASTER Not Set")?;
        let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL Not Set")?;
        // let collection_name = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL Not Set")?;

        let mut id : String = point_id.to_string();
        debug!("{:?}: id {:?}",log.clone(),id.clone());
        let payload_str : Value = to_value(&payload)?;
        let mut last_id = "".to_string();
        if let Err(e) = upsert_to_qdrant(payload_str, embedding.clone(), &*qdrant_url, &*collection_name).await {
            error!("Error Processing Row Upsert Vec To Qdrant (ID: {}): {:?}", id, e);
        } else {
            last_id = id.parse()?;
        }


        info!("[FIWARE] Stored RFID {} to Qdrant", rfid_entity.rfid_uid);
        Ok(())
    }

    /// Generate text representation of RFID entity for embedding
    fn rfid_to_text(&self, rfid: &FiwareRfidEntity) -> String {
        let location_str = rfid.location.as_ref()
            .map(|l| format!("coordinates [{}, {}]",
                l.coordinates.get(0).unwrap_or(&0.0),
                l.coordinates.get(1).unwrap_or(&0.0)))
            .unwrap_or_else(|| "unknown location".to_string());

        format!(
            "RFID Tag {} of type {} at {} in zone {} read by reader {} with status {} \
            signal strength {} read count {} timestamp {}",
            rfid.rfid_uid,
            rfid.tag_type,
            location_str,
            rfid.zone.as_deref().unwrap_or("unknown"),
            rfid.reader_id.as_deref().unwrap_or("unknown"),
            rfid.status,
            rfid.signal_strength.unwrap_or(0.0),
            rfid.read_count,
            rfid.timestamp.to_rfc3339()
        )
    }

    /// Generate embedding using local embedding service
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embed_url = env::var("EMBED_URL")
            .unwrap_or_else(|_| "http://localhost:11434/api/embeddings".to_string());

        let model = env::var("EMBED_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string());

        let request_body = json!({
            "model": model,
            "prompt": text
        });

        let response = self.http_client
            .post(&embed_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to call embedding service")?;

        if response.status().is_success() {
            let json: Value = response.json().await
                .context("Failed to parse embedding response")?;

            let embedding = json.get("embedding")
                .or_else(|| json.get("embeddings"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .ok_or_else(|| anyhow!("No embedding in response"))?;

            Ok(embedding)
        } else {
            // Fallback: generate simple hash-based embedding for testing
            warn!("[FIWARE] Embedding service unavailable, using fallback");
            Ok(generate_fallback_embedding(text))
        }
    }

    /// Search for similar RFID entities in Qdrant
    pub async fn search_rfid(&self, query: &str, limit: u64) -> Result<Vec<RfidVectorPayload>> {
        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        let query_embedding = self.generate_embedding(query).await?;

        // let search_result = client.search_points(
        //     SearchPointsBuilder::new(FIWARE_RFID_COLLECTION, query_embedding, limit)
        //         .with_payload(true)
        // ).await
        // .map_err(|e| anyhow!("Search failed: {}", e))?;

        let search_result = client
            .query(
                QueryPointsBuilder::new(FIWARE_RFID_COLLECTION)
                    .query(query_embedding)
                    .limit(limit)
            ).await.map_err(|e| anyhow!("Search failed: {}", e))?;


        let results: Vec<RfidVectorPayload> = search_result.result
            .iter()
            .filter_map(|point| {
                let payload_json: Value = point.payload.iter()
                    .map(|(k, v)| (k.clone(), qdrant_value_to_json(v)))
                    .collect::<serde_json::Map<String, Value>>()
                    .into();
                serde_json::from_value(payload_json).ok()
            })
            .collect();

        Ok(results)
    }

    /// Search RFID by zone
    pub async fn search_rfid_by_zone(&self, zone: &str, limit: u64) -> Result<Vec<RfidVectorPayload>> {
        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        // Use a generic query for zone-based search
        let query_text = format!("RFID tags in zone {}", zone);
        let query_embedding = self.generate_embedding(&query_text).await?;

        let filter = Filter::must([
            Condition::matches("zone", zone.to_string())
        ]);

        // let search_result = client.search_points(
        //     SearchPointsBuilder::new(FIWARE_RFID_COLLECTION, query_embedding, limit)
        //         .filter(filter)
        //         .with_payload(true)
        // ).await
        // .map_err(|e| anyhow!("Search failed: {}", e))?;

        let search_result = client
            .query(
                QueryPointsBuilder::new(FIWARE_RFID_COLLECTION)
                    .query(query_embedding)
                    .limit(limit)
            ).await.map_err(|e| anyhow!("Search failed: {}", e))?;


        let results: Vec<RfidVectorPayload> = search_result.result
            .iter()
            .filter_map(|point| {
                let payload_json: Value = point.payload.iter()
                    .map(|(k, v)| (k.clone(), qdrant_value_to_json(v)))
                    .collect::<serde_json::Map<String, Value>>()
                    .into();
                serde_json::from_value(payload_json).ok()
            })
            .collect();

        Ok(results)
    }

    /// Get cached entity count
    pub async fn get_cache_count(&self) -> usize {
        self.entity_cache.read().await.len()
    }

    /// Clear entity cache
    pub async fn clear_cache(&self) {
        self.entity_cache.write().await.clear();
        info!("[FIWARE] Entity cache cleared");
    }

    /// Poll FIWARE Context Broker for RFID updates
    pub async fn poll_and_sync(&self) -> Result<usize> {
        info!("[FIWARE] Polling Context Broker for RFID updates...");

        let mut total_synced = 0;

        // Query RFIDTag entities
        if let Ok(entities) = self.query_rfid_entities("RFIDTag").await {
            for entity in entities {
                if let Ok(rfid_entity) = self.entity_to_rfid(&entity) {
                    if let Ok(_) = self.store_rfid_to_qdrant(&rfid_entity).await {
                        total_synced += 1;
                    }
                }
            }
        }

        // Query Asset entities (may contain RFID data)
        if let Ok(entities) = self.query_rfid_entities("Asset").await {
            for entity in entities {
                if let Ok(rfid_entity) = self.entity_to_rfid(&entity) {
                    if let Ok(_) = self.store_rfid_to_qdrant(&rfid_entity).await {
                        total_synced += 1;
                    }
                }
            }
        }

        info!("[FIWARE] Synced {} entities to Qdrant", total_synced);
        Ok(total_synced)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Convert JSON value to Qdrant value
fn json_to_qdrant_value(v: &Value) -> QdrantValue {
    match v {
        Value::Null => QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::NullValue(0)) },
        Value::Bool(b) => QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(*b)) },
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) }
            } else if let Some(f) = n.as_f64() {
                QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) }
            } else {
                QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(n.to_string())) }
            }
        }
        Value::String(s) => QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s.clone())) },
        _ => QdrantValue { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(v.to_string())) },
    }
}

/// Convert Qdrant value to JSON value
fn qdrant_value_to_json(v: &QdrantValue) -> Value {
    match &v.kind {
        Some(qdrant_client::qdrant::value::Kind::NullValue(_)) => Value::Null,
        Some(qdrant_client::qdrant::value::Kind::BoolValue(b)) => Value::Bool(*b),
        Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) => json!(*i),
        Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) => json!(*f),
        Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Value::String(s.clone()),
        _ => Value::Null,
    }
}

/// Generate fallback embedding when embedding service is unavailable
fn generate_fallback_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut embedding = vec![0.0f32; VECTOR_DIMENSION as usize];

    for (i, chunk) in text.as_bytes().chunks(4).enumerate() {
        let mut hasher = DefaultHasher::new();
        chunk.hash(&mut hasher);
        let hash = hasher.finish();

        let idx = i % (VECTOR_DIMENSION as usize);
        embedding[idx] = ((hash as f32) / (u64::MAX as f32)) * 2.0 - 1.0;
    }

    // Normalize the vector
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for val in embedding.iter_mut() {
            *val /= magnitude;
        }
    }

    embedding
}

// ============================================================================
// WEB HANDLER ENDPOINTS
// ============================================================================

/// Initialize FIWARE agent and sync RFID data
pub async fn fiware_init(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[FIWARE] Init request from uid: {}", uid);

    let config = FiwareConfig::default();
    let handler = FiwareAgentHandler::new(config);

    // Initialize Qdrant
    if let Err(e) = handler.init_qdrant().await {
        error!("[FIWARE] Failed to init Qdrant: {}", e);
        return srv_response(
            json!({ "status": "error", "message": format!("Qdrant init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Poll and sync data
    let synced = handler.poll_and_sync().await.unwrap_or(0);

    let response = json!({
        "status": "success",
        "message": "FIWARE agent initialized",
        "synced_entities": synced,
        "timestamp": current_time()
    });

    srv_response(response.to_string(), StatusCode::OK)
}

/// Handle FIWARE notification webhook
pub async fn fiware_notify(uid: String, notification: FiwareNotification) -> WebResult<impl Reply> {
    info!("[FIWARE] Notification received from subscription: {}", notification.subscription_id);

    let config = FiwareConfig::default();
    let handler = FiwareAgentHandler::new(config);

    // Initialize Qdrant
    if let Err(e) = handler.init_qdrant().await {
        error!("[FIWARE] Failed to init Qdrant: {}", e);
        return srv_response(
            json!({ "status": "error", "message": format!("Qdrant init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Process notification
    match handler.process_notification(notification).await {
        Ok(count) => {
            let response = json!({
                "status": "success",
                "processed_count": count,
                "timestamp": current_time()
            });
            srv_response(response.to_string(), StatusCode::OK)
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": format!("Processing failed: {}", e)
            });
            srv_response(response.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Search RFID data in Qdrant
pub async fn fiware_search(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[FIWARE] Search request: {}", body.prompt);

    let config = FiwareConfig::default();
    let handler = FiwareAgentHandler::new(config);

    // Initialize Qdrant
    if let Err(e) = handler.init_qdrant().await {
        return srv_response(
            json!({ "status": "error", "message": format!("Qdrant init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Parse limit from options
    let limit = body.options.parse::<u64>().unwrap_or(10);

    // Search
    match handler.search_rfid(&body.prompt, limit).await {
        Ok(results) => {
            let response = json!({
                "status": "success",
                "query": body.prompt,
                "count": results.len(),
                "results": results,
                "timestamp": current_time()
            });
            srv_response(response.to_string(), StatusCode::OK)
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": format!("Search failed: {}", e)
            });
            srv_response(response.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create FIWARE subscription for RFID updates
pub async fn fiware_subscribe(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[FIWARE] Subscribe request from uid: {}", uid);

    let config = FiwareConfig::default();
    // let config_put_value = config.clone();
    let handler = FiwareAgentHandler::new(config.clone());

    match handler.create_rfid_subscription().await {
        Ok(subscription_id) => {
            let response = json!({
                "status": "success",
                "subscription_id": subscription_id,
                "endpoint": config.clone().subscription_endpoint,
                "timestamp": current_time()
            });
            srv_response(response.to_string(), StatusCode::OK)
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": format!("Subscription failed: {}", e)
            });
            srv_response(response.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiware_config_default() {
        let config = FiwareConfig::default();
        assert!(!config.context_broker_url.is_empty());
        assert!(!config.fiware_service.is_empty());
    }

    #[test]
    fn test_fallback_embedding() {
        let text = "Test RFID tag data";
        let embedding = generate_fallback_embedding(text);

        assert_eq!(embedding.len(), VECTOR_DIMENSION as usize);

        // Check normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_json_to_qdrant_conversion() {
        let json_str = json!("test");
        let qdrant_val = json_to_qdrant_value(&json_str);
        let back_to_json = qdrant_value_to_json(&qdrant_val);
        assert_eq!(json_str, back_to_json);
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let config = FiwareConfig::default();
        let handler = FiwareAgentHandler::new(config);
        assert_eq!(handler.get_cache_count().await, 0);
    }
}
