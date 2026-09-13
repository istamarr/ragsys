// Embedded Agent Handler for IoT Devices to Qdrant Integration
// Supports: RFID, NFC, BLE Beacons, GPS Trackers, Environmental Sensors, Biometric Devices

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
use crate::domain::dbs_rag_LM;
use crate::domain::dbs_rag_LM::QdDetail;
use crate::domain::models::llm::CmdBody;
use crate::data::result_data::saveLogDetailKnowledge::upsert_to_qdrant;
use crate::shared::helperUtils::{current_time, srv_response};
use crate::shared::secureUtils::generate_api_key;
use crate::WebResult;

// ============================================================================
// CONSTANTS
// ============================================================================

const EMBEDDED_COLLECTION: &str = "embedded_devices";
const VECTOR_DIMENSION: u64 = 384;

// ============================================================================
// DEVICE TYPE ENUMERATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Rfid,
    Nfc,
    BleBeacon,
    GpsTracker,
    TemperatureSensor,
    HumiditySensor,
    PressureSensor,
    MotionSensor,
    ProximitySensor,
    LightSensor,
    Accelerometer,
    Gyroscope,
    Biometric,
    Camera,
    Barcode,
    QrCode,
    UltrasonicSensor,
    InfraredSensor,
    GasSensor,
    WaterSensor,
    VibrationSensor,
    Custom(String),
}

impl DeviceType {
    pub fn as_str(&self) -> &str {
        match self {
            DeviceType::Rfid => "rfid",
            DeviceType::Nfc => "nfc",
            DeviceType::BleBeacon => "ble_beacon",
            DeviceType::GpsTracker => "gps_tracker",
            DeviceType::TemperatureSensor => "temperature_sensor",
            DeviceType::HumiditySensor => "humidity_sensor",
            DeviceType::PressureSensor => "pressure_sensor",
            DeviceType::MotionSensor => "motion_sensor",
            DeviceType::ProximitySensor => "proximity_sensor",
            DeviceType::LightSensor => "light_sensor",
            DeviceType::Accelerometer => "accelerometer",
            DeviceType::Gyroscope => "gyroscope",
            DeviceType::Biometric => "biometric",
            DeviceType::Camera => "camera",
            DeviceType::Barcode => "barcode",
            DeviceType::QrCode => "qr_code",
            DeviceType::UltrasonicSensor => "ultrasonic_sensor",
            DeviceType::InfraredSensor => "infrared_sensor",
            DeviceType::GasSensor => "gas_sensor",
            DeviceType::WaterSensor => "water_sensor",
            DeviceType::VibrationSensor => "vibration_sensor",
            DeviceType::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rfid" => DeviceType::Rfid,
            "nfc" => DeviceType::Nfc,
            "ble_beacon" | "ble" | "beacon" => DeviceType::BleBeacon,
            "gps_tracker" | "gps" => DeviceType::GpsTracker,
            "temperature_sensor" | "temperature" | "temp" => DeviceType::TemperatureSensor,
            "humidity_sensor" | "humidity" => DeviceType::HumiditySensor,
            "pressure_sensor" | "pressure" => DeviceType::PressureSensor,
            "motion_sensor" | "motion" | "pir" => DeviceType::MotionSensor,
            "proximity_sensor" | "proximity" => DeviceType::ProximitySensor,
            "light_sensor" | "light" | "lux" => DeviceType::LightSensor,
            "accelerometer" | "accel" => DeviceType::Accelerometer,
            "gyroscope" | "gyro" => DeviceType::Gyroscope,
            "biometric" | "fingerprint" | "face" => DeviceType::Biometric,
            "camera" | "cam" => DeviceType::Camera,
            "barcode" => DeviceType::Barcode,
            "qr_code" | "qr" => DeviceType::QrCode,
            "ultrasonic_sensor" | "ultrasonic" => DeviceType::UltrasonicSensor,
            "infrared_sensor" | "infrared" | "ir" => DeviceType::InfraredSensor,
            "gas_sensor" | "gas" => DeviceType::GasSensor,
            "water_sensor" | "water" => DeviceType::WaterSensor,
            "vibration_sensor" | "vibration" => DeviceType::VibrationSensor,
            other => DeviceType::Custom(other.to_string()),
        }
    }
}

// ============================================================================
// EMBEDDED DEVICE DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDevice {
    pub id: String,
    pub device_type: DeviceType,
    pub device_uid: String,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub location: Option<GeoLocation>,
    pub zone: Option<String>,
    pub status: DeviceStatus,
    pub battery_level: Option<f32>,
    pub signal_strength: Option<f32>,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Active,
    Inactive,
    Offline,
    Error,
    Maintenance,
    LowBattery,
    Unknown,
}

impl DeviceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            DeviceStatus::Active => "active",
            DeviceStatus::Inactive => "inactive",
            DeviceStatus::Offline => "offline",
            DeviceStatus::Error => "error",
            DeviceStatus::Maintenance => "maintenance",
            DeviceStatus::LowBattery => "low_battery",
            DeviceStatus::Unknown => "unknown",
        }
    }
}

// ============================================================================
// DEVICE-SPECIFIC DATA STRUCTURES
// ============================================================================

// RFID Tag Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidData {
    pub uid: String,
    pub tag_type: RfidTagType,
    pub memory_size: u32,
    pub read_count: u32,
    pub data_blocks: Option<Vec<Vec<u8>>>,
    pub antenna_id: Option<u8>,
    pub rssi: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RfidTagType {
    Mifare1K,
    Mifare4K,
    MifareUltralight,
    MifarePlus,
    MifareDesfire,
    Ntag213,
    Ntag215,
    Ntag216,
    Iso14443A,
    Iso14443B,
    Iso15693,
    Iso18000,
    EpcGen2,
    Unknown,
}

// NFC Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfcData {
    pub uid: String,
    pub nfc_type: NfcType,
    pub ndef_message: Option<String>,
    pub ndef_records: Vec<NdefRecord>,
    pub is_writable: bool,
    pub max_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NfcType {
    Type1,
    Type2,
    Type3,
    Type4,
    Type5,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdefRecord {
    pub record_type: String,
    pub payload: String,
    pub id: Option<String>,
}

// BLE Beacon Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleBeaconData {
    pub mac_address: String,
    pub uuid: Option<String>,
    pub major: Option<u16>,
    pub minor: Option<u16>,
    pub tx_power: Option<i8>,
    pub rssi: i32,
    pub distance: Option<f64>,
    pub beacon_type: BeaconType,
    pub manufacturer_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeaconType {
    IBeacon,
    Eddystone,
    AltBeacon,
    Unknown,
}

// GPS Tracker Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub accuracy: Option<f64>,
    pub satellites: Option<u8>,
    pub fix_type: GpsFixType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpsFixType {
    NoFix,
    Fix2D,
    Fix3D,
    DgpsFix,
    RtkFix,
}

// Environmental Sensor Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_type: DeviceType,
    pub value: f64,
    pub unit: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub accuracy: Option<f64>,
    pub calibration_date: Option<DateTime<Utc>>,
}

// Biometric Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricData {
    pub biometric_type: BiometricType,
    pub template_id: Option<String>,
    pub quality_score: Option<f32>,
    pub match_score: Option<f32>,
    pub is_verified: bool,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricType {
    Fingerprint,
    FaceRecognition,
    IrisRecognition,
    VoiceRecognition,
    PalmPrint,
    Retina,
    Unknown,
}

// ============================================================================
// UNIFIED DEVICE EVENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub event_id: String,
    pub device: EmbeddedDevice,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: DeviceEventData,
    pub raw_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Read,
    Write,
    Connect,
    Disconnect,
    Alert,
    Error,
    StatusChange,
    LocationUpdate,
    DataUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeviceEventData {
    Rfid(RfidData),
    Nfc(NfcData),
    BleBeacon(BleBeaconData),
    Gps(GpsData),
    Sensor(SensorReading),
    Biometric(BiometricData),
    Generic(HashMap<String, Value>),
}

// ============================================================================
// QDRANT VECTOR PAYLOAD
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedVectorPayload {
    pub event_id: String,
    pub device_id: String,
    pub device_uid: String,
    pub device_type: String,
    pub device_name: String,
    pub zone: String,
    pub status: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub battery_level: f32,
    pub signal_strength: f32,
    pub event_type: String,
    pub timestamp: String,
    pub raw_text: String,
    pub data_json: String,
}

// ============================================================================
// EMBEDDED AGENT CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedAgentConfig {
    pub qdrant_url: String,
    pub embed_url: String,
    pub embed_model: String,
    pub collection_name: String,
    pub timeout_seconds: u64,
    pub batch_size: usize,
    pub auto_create_collection: bool,
}

impl Default for EmbeddedAgentConfig {
    fn default() -> Self {
        Self {
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".to_string()),
            embed_url: env::var("ollamar/tamar:1b")
                .unwrap_or_else(|_| "http://localhost:11434/api/embeddings".to_string()),
            embed_model: env::var("EMBED_MODEL")
                .unwrap_or_else(|_| "ollamar/tamar:1b".to_string()),
            collection_name: EMBEDDED_COLLECTION.to_string(),
            timeout_seconds: 30,
            batch_size: 100,
            auto_create_collection: true,
        }
    }
}

// ============================================================================
// EMBEDDED AGENT HANDLER
// ============================================================================

pub struct EmbeddedAgentHandler {
    config: EmbeddedAgentConfig,
    http_client: HttpClient,
    qdrant_client: Arc<RwLock<Option<Qdrant>>>,
    device_cache: Arc<RwLock<HashMap<String, EmbeddedDevice>>>,
    event_buffer: Arc<RwLock<Vec<DeviceEvent>>>,
}

impl EmbeddedAgentHandler {
    pub fn new(config: EmbeddedAgentConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| HttpClient::new());

        Self {
            config,
            http_client,
            qdrant_client: Arc::new(RwLock::new(None)),
            device_cache: Arc::new(RwLock::new(HashMap::new())),
            event_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize Qdrant connection and collection
    pub async fn init_qdrant(&self) -> Result<()> {
        info!("[EMBEDDED] Connecting to Qdrant at {}", self.config.qdrant_url);

        let client = Qdrant::from_url(&self.config.qdrant_url)
            .build()
            .map_err(|e| anyhow!("Failed to create Qdrant client: {}", e))?;

        if self.config.auto_create_collection {
            let collections = client.list_collections().await
                .map_err(|e| anyhow!("Failed to list collections: {}", e))?;

            let exists = collections.collections
                .iter()
                .any(|c| c.name == self.config.collection_name);

            if !exists {
                info!("[EMBEDDED] Creating collection: {}", self.config.collection_name);

                client.create_collection(
                    qdrant_client::qdrant::CreateCollectionBuilder::new(&self.config.collection_name)
                        .vectors_config(VectorParams {
                            size: VECTOR_DIMENSION,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })
                ).await
                .map_err(|e| anyhow!("Failed to create collection: {}", e))?;

                info!("[EMBEDDED] Collection created successfully");
            }
        }

        let mut qdrant_lock = self.qdrant_client.write().await;
        *qdrant_lock = Some(client);

        info!("[EMBEDDED] Qdrant initialized successfully");
        Ok(())
    }

    // ========================================================================
    // RFID OPERATIONS
    // ========================================================================

    /// Process RFID tag read event
    pub async fn process_rfid_read(&self, rfid_data: RfidData, zone: Option<String>) -> Result<String> {
        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: DeviceType::Rfid,
            device_uid: rfid_data.uid.clone(),
            name: Some(format!("RFID-{}", &rfid_data.uid[..8.min(rfid_data.uid.len())])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location: None,
            zone: zone.clone(),
            status: DeviceStatus::Active,
            battery_level: None,
            signal_strength: rfid_data.rssi.map(|r| r as f32),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::Read,
            timestamp: Utc::now(),
            data: DeviceEventData::Rfid(rfid_data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    /// Batch process multiple RFID reads
    pub async fn process_rfid_batch(&self, readings: Vec<(RfidData, Option<String>)>) -> Result<Vec<String>> {
        let mut event_ids = Vec::new();

        for (rfid_data, zone) in readings {
            match self.process_rfid_read(rfid_data, zone).await {
                Ok(id) => event_ids.push(id),
                Err(e) => warn!("[EMBEDDED] Failed to process RFID: {}", e),
            }
        }

        Ok(event_ids)
    }

    // ========================================================================
    // NFC OPERATIONS
    // ========================================================================

    /// Process NFC tag read event
    pub async fn process_nfc_read(&self, nfc_data: NfcData, location: Option<GeoLocation>) -> Result<String> {
        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: DeviceType::Nfc,
            device_uid: nfc_data.uid.clone(),
            name: Some(format!("NFC-{}", &nfc_data.uid[..8.min(nfc_data.uid.len())])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location,
            zone: None,
            status: DeviceStatus::Active,
            battery_level: None,
            signal_strength: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::Read,
            timestamp: Utc::now(),
            data: DeviceEventData::Nfc(nfc_data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // BLE BEACON OPERATIONS
    // ========================================================================

    /// Process BLE beacon detection
    pub async fn process_ble_beacon(&self, beacon_data: BleBeaconData, zone: Option<String>) -> Result<String> {
        let distance_str = beacon_data.distance
            .map(|d| format!("{:.2}m", d))
            .unwrap_or_else(|| "unknown".to_string());

        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: DeviceType::BleBeacon,
            device_uid: beacon_data.mac_address.clone(),
            name: Some(format!("Beacon-{}", &beacon_data.mac_address.replace(":", "")[..6])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location: None,
            zone,
            status: DeviceStatus::Active,
            battery_level: None,
            signal_strength: Some(beacon_data.rssi as f32),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("distance".to_string(), distance_str);
                m
            },
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::Read,
            timestamp: Utc::now(),
            data: DeviceEventData::BleBeacon(beacon_data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // GPS TRACKER OPERATIONS
    // ========================================================================

    /// Process GPS location update
    pub async fn process_gps_update(&self, device_uid: String, gps_data: GpsData) -> Result<String> {
        let location = GeoLocation {
            latitude: gps_data.latitude,
            longitude: gps_data.longitude,
            altitude: gps_data.altitude,
            accuracy: gps_data.accuracy,
        };

        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: DeviceType::GpsTracker,
            device_uid: device_uid.clone(),
            name: Some(format!("GPS-{}", &device_uid[..8.min(device_uid.len())])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location: Some(location),
            zone: None,
            status: DeviceStatus::Active,
            battery_level: None,
            signal_strength: gps_data.satellites.map(|s| s as f32 * 10.0),
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::LocationUpdate,
            timestamp: Utc::now(),
            data: DeviceEventData::Gps(gps_data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // SENSOR OPERATIONS
    // ========================================================================

    /// Process sensor reading
    pub async fn process_sensor_reading(
        &self,
        device_uid: String,
        sensor_type: DeviceType,
        reading: SensorReading,
        location: Option<GeoLocation>,
    ) -> Result<String> {
        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: sensor_type.clone(),
            device_uid: device_uid.clone(),
            name: Some(format!("{}-{}", sensor_type.as_str(), &device_uid[..8.min(device_uid.len())])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location,
            zone: None,
            status: DeviceStatus::Active,
            battery_level: None,
            signal_strength: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("value".to_string(), reading.value.to_string());
                m.insert("unit".to_string(), reading.unit.clone());
                m
            },
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::DataUpdate,
            timestamp: Utc::now(),
            data: DeviceEventData::Sensor(reading),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // BIOMETRIC OPERATIONS
    // ========================================================================

    /// Process biometric verification
    pub async fn process_biometric(&self, device_uid: String, biometric_data: BiometricData) -> Result<String> {
        let status = if biometric_data.is_verified {
            DeviceStatus::Active
        } else {
            DeviceStatus::Error
        };

        let device = EmbeddedDevice {
            id: Uuid::new_v4().to_string(),
            device_type: DeviceType::Biometric,
            device_uid: device_uid.clone(),
            name: Some(format!("Bio-{}", &device_uid[..8.min(device_uid.len())])),
            manufacturer: None,
            model: None,
            firmware_version: None,
            location: None,
            zone: None,
            status,
            battery_level: None,
            signal_strength: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::Read,
            timestamp: Utc::now(),
            data: DeviceEventData::Biometric(biometric_data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // GENERIC EVENT PROCESSING
    // ========================================================================

    /// Process generic device event
    pub async fn process_generic_event(&self, device: EmbeddedDevice, data: HashMap<String, Value>) -> Result<String> {
        let event = DeviceEvent {
            event_id: Uuid::new_v4().to_string(),
            device: device.clone(),
            event_type: EventType::DataUpdate,
            timestamp: Utc::now(),
            data: DeviceEventData::Generic(data),
            raw_payload: None,
        };

        self.store_event(&event).await?;
        self.cache_device(device).await;

        Ok(event.event_id)
    }

    // ========================================================================
    // STORAGE OPERATIONS
    // ========================================================================

    /// Store event to Qdrant
    async fn store_event(&self, event: &DeviceEvent) -> Result<()> {
        let log = "store event".to_string();
        info!("{}", log.clone());

        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        let text_repr = self.event_to_text(event);
        let embedding = self.generate_embedding(&text_repr).await?;

        let payload = EmbeddedVectorPayload {
            event_id: event.event_id.clone(),
            device_id: event.device.id.clone(),
            device_uid: event.device.device_uid.clone(),
            device_type: event.device.device_type.as_str().to_string(),
            device_name: event.device.name.clone().unwrap_or_default(),
            zone: event.device.zone.clone().unwrap_or_default(),
            status: event.device.status.as_str().to_string(),
            latitude: event.device.location.as_ref().map(|l| l.latitude),
            longitude: event.device.location.as_ref().map(|l| l.longitude),
            battery_level: event.device.battery_level.unwrap_or(0.0),
            signal_strength: event.device.signal_strength.unwrap_or(0.0),
            event_type: format!("{:?}", event.event_type).to_lowercase(),
            timestamp: event.timestamp.to_rfc3339(),
            raw_text: text_repr,
            data_json: serde_json::to_string(&event.data).unwrap_or_default(),
        };

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

        //buat format 2  param atau formatting:
        // let point_id = Uuid::new_v8(&Uuid::NAMESPACE_OID, event.event_id.as_bytes());
        let point_id = generate_api_key();

        let point = PointStruct {
            id: Some(PointId::from(point_id.to_string())),
            vectors: Some(embedding.clone().into()),
            payload: qdrant_payload,
        };

        // client.upsert_points(&self.config.collection_name, None, vec![point], None)
        //     .await
        //     .map_err(|e| anyhow!("Failed to upsert point: {}", e))?;

        let collection_name = &self.config.collection_name;
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

        // let client = Qdrant::from_url(&*qdrant_url).build()?;
        // if let Some(existing_id) = crate::rag_agent::e::result_data::saveLogDetailKnowledge::check_existing_point(&client, &*collection_name, &payload).await? {
        //     info!("{:?}: embedding {:?} (ID: {:?}) ",
        //     log.clone(),
        //     "Point with payload exists",
        //     existing_id);
        // } else {
        //     if let Err(e) = upsert_to_qdrant(payload_str, embedding, &*qdrant_url, &*collection_name).await {
        //         error!("Error Processing Row Upsert Vec To Qdrant (ID: {}): {:?}", id, e);
        //     } else {
        //         last_id = id.parse()?;
        //     }
        //     info!("{:?}: Last ID {:?} Succeed {:?}",log.clone(), last_id.clone(), current_time());
        // }

        info!("[EMBEDDED] Stored event {} for device {}", event.event_id, event.device.device_uid);
        Ok(())
    }

    /// Generate text representation for embedding
    fn event_to_text(&self, event: &DeviceEvent) -> String {
        let location_str = event.device.location.as_ref()
            .map(|l| format!("at coordinates [{:.6}, {:.6}]", l.latitude, l.longitude))
            .unwrap_or_else(|| "location unknown".to_string());

        let data_summary = match &event.data {
            DeviceEventData::Rfid(r) => format!("RFID tag {} type {:?} read count {}", r.uid, r.tag_type, r.read_count),
            DeviceEventData::Nfc(n) => format!("NFC tag {} type {:?} records {}", n.uid, n.nfc_type, n.ndef_records.len()),
            DeviceEventData::BleBeacon(b) => format!("BLE beacon {} rssi {} distance {:?}m", b.mac_address, b.rssi, b.distance),
            DeviceEventData::Gps(g) => format!("GPS position [{:.6}, {:.6}] speed {:?} heading {:?}", g.latitude, g.longitude, g.speed, g.heading),
            DeviceEventData::Sensor(s) => format!("Sensor reading {:.2} {}", s.value, s.unit),
            DeviceEventData::Biometric(b) => format!("Biometric {:?} verified {} quality {:?}", b.biometric_type, b.is_verified, b.quality_score),
            DeviceEventData::Generic(g) => format!("Generic data with {} fields", g.len()),
        };

        format!(
            "{} device {} ({}) {} in zone {} status {} signal {} battery {} event {:?} {} timestamp {}",
            event.device.device_type.as_str(),
            event.device.device_uid,
            event.device.name.as_deref().unwrap_or("unnamed"),
            location_str,
            event.device.zone.as_deref().unwrap_or("unknown"),
            event.device.status.as_str(),
            event.device.signal_strength.unwrap_or(0.0),
            event.device.battery_level.unwrap_or(0.0),
            event.event_type,
            data_summary,
            event.timestamp.to_rfc3339()
        )
    }

    /// Generate embedding using local service
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request_body = json!({
            "model": self.config.embed_model,
            "prompt": text
        });

        let response = self.http_client
            .post(&self.config.embed_url)
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
            warn!("[EMBEDDED] Embedding service unavailable, using fallback");
            Ok(generate_fallback_embedding(text))
        }
    }

    /// Cache device in memory
    async fn cache_device(&self, device: EmbeddedDevice) {
        let mut cache = self.device_cache.write().await;
        cache.insert(device.device_uid.clone(), device);
    }

    // ========================================================================
    // SEARCH OPERATIONS
    // ========================================================================

    /// Search devices by query
    pub async fn search_devices(&self, query: &str, limit: u64) -> Result<Vec<EmbeddedVectorPayload>> {
        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        let query_embedding = self.generate_embedding(query).await?;

        // let search_result = client.search_points(
        //     SearchPointsBuilder::new(&self.config.collection_name, query_embedding, limit)
        //         .with_payload(true)
        // ).await
        // .map_err(|e| anyhow!("Search failed: {}", e))?;

        let search_result = client
            .query(
                QueryPointsBuilder::new(&self.config.collection_name)
                    .query(query_embedding)
                    .limit(limit)
            ).await.map_err(|e| anyhow!("Search failed: {}", e))?;

        let results: Vec<EmbeddedVectorPayload> = search_result.result
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

    /// Search by device type
    pub async fn search_by_device_type(&self, device_type: DeviceType, limit: u64) -> Result<Vec<EmbeddedVectorPayload>> {
        let query = format!("{} device events", device_type.as_str());

        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        let query_embedding = self.generate_embedding(&query).await?;

        let filter = Filter::must([
            Condition::matches("device_type", device_type.as_str().to_string())
        ]);

        // let search_result = client.search_points(
        //     SearchPointsBuilder::new(&self.config.collection_name, query_embedding, limit)
        //         .filter(filter)
        //         .with_payload(true)
        // ).await
        // .map_err(|e| anyhow!("Search failed: {}", e))?;

        let search_result = client
            .query(
                QueryPointsBuilder::new(&self.config.collection_name)
                    .query(query_embedding)
                    .limit(limit)
            ).await.map_err(|e| anyhow!("Search failed: {}", e))?;


        let results: Vec<EmbeddedVectorPayload> = search_result.result
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

    /// Search by zone
    pub async fn search_by_zone(&self, zone: &str, limit: u64) -> Result<Vec<EmbeddedVectorPayload>> {
        let query = format!("devices in zone {}", zone);

        let qdrant_lock = self.qdrant_client.read().await;
        let client = qdrant_lock.as_ref()
            .ok_or_else(|| anyhow!("Qdrant client not initialized"))?;

        let query_embedding = self.generate_embedding(&query).await?;

        let filter = Filter::must([
            Condition::matches("zone", zone.to_string())
        ]);

        // let search_result = client.search_points(
        //     SearchPointsBuilder::new(&self.config.collection_name, query_embedding, limit)
        //         .filter(filter)
        //         .with_payload(true)
        // ).await
        // .map_err(|e| anyhow!("Search failed: {}", e))?;

        let search_result = client
            .query(
                QueryPointsBuilder::new(&self.config.collection_name)
                    .query(query_embedding)
                    .limit(limit)
            ).await.map_err(|e| anyhow!("Search failed: {}", e))?;

        let results: Vec<EmbeddedVectorPayload> = search_result.result
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

    // ========================================================================
    // UTILITY METHODS
    // ========================================================================

    /// Get cached device count
    pub async fn get_cache_count(&self) -> usize {
        self.device_cache.read().await.len()
    }

    /// Clear device cache
    pub async fn clear_cache(&self) {
        self.device_cache.write().await.clear();
        info!("[EMBEDDED] Device cache cleared");
    }

    /// Get device from cache
    pub async fn get_cached_device(&self, device_uid: &str) -> Option<EmbeddedDevice> {
        self.device_cache.read().await.get(device_uid).cloned()
    }

    /// List all cached devices
    pub async fn list_cached_devices(&self) -> Vec<EmbeddedDevice> {
        self.device_cache.read().await.values().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> EmbeddedStats {
        let cache = self.device_cache.read().await;
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for device in cache.values() {
            *type_counts.entry(device.device_type.as_str().to_string()).or_insert(0) += 1;
        }

        EmbeddedStats {
            total_devices: cache.len(),
            device_type_counts: type_counts,
            collection_name: self.config.collection_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedStats {
    pub total_devices: usize,
    pub device_type_counts: HashMap<String, usize>,
    pub collection_name: String,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

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

/// Initialize embedded agent
pub async fn embedded_init(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[EMBEDDED] Init request from uid: {}", uid);

    let config = EmbeddedAgentConfig::default();
    let handler = EmbeddedAgentHandler::new(config);

    if let Err(e) = handler.init_qdrant().await {
        error!("[EMBEDDED] Failed to init Qdrant: {}", e);
        return srv_response(
            json!({ "status": "error", "message": format!("Init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    let response = json!({
        "status": "success",
        "message": "Embedded agent initialized",
        "supported_devices": [
            "rfid", "nfc", "ble_beacon", "gps_tracker",
            "temperature_sensor", "humidity_sensor", "pressure_sensor",
            "motion_sensor", "proximity_sensor", "light_sensor",
            "accelerometer", "gyroscope", "biometric",
            "camera", "barcode", "qr_code"
        ],
        "timestamp": current_time()
    });

    srv_response(response.to_string(), StatusCode::OK)
}

/// Process device event
pub async fn embedded_event(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[EMBEDDED] Event request: {}", body.cmd);

    let config = EmbeddedAgentConfig::default();
    let handler = EmbeddedAgentHandler::new(config);

    if let Err(e) = handler.init_qdrant().await {
        return srv_response(
            json!({ "status": "error", "message": format!("Init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Parse event data from body.prompt as JSON
    let event_data: Result<Value, _> = serde_json::from_str(&body.prompt);

    match event_data {
        Ok(data) => {
            let device_type = DeviceType::from_str(&body.cmd);
            let zone = if body.tags.is_empty() { None } else { Some(body.tags.clone()) };

            let result = match device_type {
                DeviceType::Rfid => {
                    if let Ok(rfid) = serde_json::from_value::<RfidData>(data) {
                        handler.process_rfid_read(rfid, zone).await
                    } else {
                        Err(anyhow!("Invalid RFID data format"))
                    }
                }
                DeviceType::Nfc => {
                    if let Ok(nfc) = serde_json::from_value::<NfcData>(data) {
                        handler.process_nfc_read(nfc, None).await
                    } else {
                        Err(anyhow!("Invalid NFC data format"))
                    }
                }
                DeviceType::BleBeacon => {
                    if let Ok(beacon) = serde_json::from_value::<BleBeaconData>(data) {
                        handler.process_ble_beacon(beacon, zone).await
                    } else {
                        Err(anyhow!("Invalid BLE beacon data format"))
                    }
                }
                DeviceType::GpsTracker => {
                    if let Ok(gps) = serde_json::from_value::<GpsData>(data) {
                        let device_uid = body.options.clone();
                        handler.process_gps_update(device_uid, gps).await
                    } else {
                        Err(anyhow!("Invalid GPS data format"))
                    }
                }
                _ => {
                    if let Ok(sensor) = serde_json::from_value::<SensorReading>(data.clone()) {
                        let device_uid = body.options.clone();
                        handler.process_sensor_reading(device_uid, device_type, sensor, None).await
                    } else if let Ok(generic) = serde_json::from_value::<HashMap<String, Value>>(data) {
                        let device = EmbeddedDevice {
                            id: Uuid::new_v4().to_string(),
                            device_type,
                            device_uid: body.options.clone(),
                            name: None,
                            manufacturer: None,
                            model: None,
                            firmware_version: None,
                            location: None,
                            zone,
                            status: DeviceStatus::Active,
                            battery_level: None,
                            signal_strength: None,
                            last_seen: Utc::now(),
                            created_at: Utc::now(),
                            metadata: HashMap::new(),
                        };
                        handler.process_generic_event(device, generic).await
                    } else {
                        Err(anyhow!("Invalid data format"))
                    }
                }
            };

            match result {
                Ok(event_id) => {
                    let response = json!({
                        "status": "success",
                        "event_id": event_id,
                        "timestamp": current_time()
                    });
                    srv_response(response.to_string(), StatusCode::OK)
                }
                Err(e) => {
                    let response = json!({
                        "status": "error",
                        "message": format!("Processing failed: {}", e)
                    });
                    srv_response(response.to_string(), StatusCode::BAD_REQUEST)
                }
            }
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": format!("Invalid JSON: {}", e)
            });
            srv_response(response.to_string(), StatusCode::BAD_REQUEST)
        }
    }
}

/// Search embedded devices
pub async fn embedded_search(uid: String, body: CmdBody) -> WebResult<impl Reply> {
    info!("[EMBEDDED] Search request: {}", body.prompt);

    let config = EmbeddedAgentConfig::default();
    let handler = EmbeddedAgentHandler::new(config);

    if let Err(e) = handler.init_qdrant().await {
        return srv_response(
            json!({ "status": "error", "message": format!("Init failed: {}", e) }).to_string(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    let limit = body.options.parse::<u64>().unwrap_or(10);

    let results = if !body.cmd.is_empty() {
        let device_type = DeviceType::from_str(&body.cmd);
        handler.search_by_device_type(device_type, limit).await
    } else if !body.tags.is_empty() {
        handler.search_by_zone(&body.tags, limit).await
    } else {
        handler.search_devices(&body.prompt, limit).await
    };

    match results {
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

/// Get embedded agent stats
pub async fn embedded_stats(uid: String) -> WebResult<impl Reply> {
    info!("[EMBEDDED] Stats request from uid: {}", uid);

    let config = EmbeddedAgentConfig::default();
    let handler = EmbeddedAgentHandler::new(config);

    let stats = handler.get_stats().await;

    let response = json!({
        "status": "success",
        "stats": stats,
        "timestamp": current_time()
    });

    srv_response(response.to_string(), StatusCode::OK)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_str() {
        assert_eq!(DeviceType::from_str("rfid"), DeviceType::Rfid);
        assert_eq!(DeviceType::from_str("nfc"), DeviceType::Nfc);
        assert_eq!(DeviceType::from_str("ble"), DeviceType::BleBeacon);
        assert_eq!(DeviceType::from_str("gps"), DeviceType::GpsTracker);
        assert_eq!(DeviceType::from_str("temperature"), DeviceType::TemperatureSensor);
    }

    #[test]
    fn test_config_default() {
        let config = EmbeddedAgentConfig::default();
        assert!(!config.qdrant_url.is_empty());
        assert!(!config.embed_url.is_empty());
    }

    #[test]
    fn test_fallback_embedding() {
        let text = "Test embedded device data";
        let embedding = generate_fallback_embedding(text);
        assert_eq!(embedding.len(), VECTOR_DIMENSION as usize);

        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let config = EmbeddedAgentConfig::default();
        let handler = EmbeddedAgentHandler::new(config);
        assert_eq!(handler.get_cache_count().await, 0);
    }

    #[test]
    fn test_device_status() {
        assert_eq!(DeviceStatus::Active.as_str(), "active");
        assert_eq!(DeviceStatus::Offline.as_str(), "offline");
        assert_eq!(DeviceStatus::LowBattery.as_str(), "low_battery");
    }
}
