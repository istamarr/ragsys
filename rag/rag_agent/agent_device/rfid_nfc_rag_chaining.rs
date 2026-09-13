use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{SearchPointsBuilder, VectorParams, Distance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Arc;
use uuid::Uuid;

use crate::rag_module::handler::rfid_nfc_handler::{
    RfidTag, RfidReader, RfidReaderConfig, MetalType, ValidationResult,
    UhfTagData, MetalDetectionData, Position3D, ItemCount, ItemDetail
};
use crate::shared::helperUtils::current_time;
use crate::WebResult;

// ============================================================================
// RAG CHAINING FOR RFID/NFC SYSTEM
// ============================================================================

/// Enhanced use cases for valuable items tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RfidUseCase {
    AccessControl { location: String },
    InventoryManagement { warehouse: String },
    AssetTracking { facility: String },
    AttendanceSystem { organization: String },
    PaymentSystem { merchant: String },
    SecurityAudit { system_name: String },

    // New use cases for valuable items
    ValuablesTracking {
        facility: String,
        metal_type: MetalType,
        security_level: SecurityLevel
    },
    JewelryInventory {
        store_id: String,
        display_case: Option<String>,
    },
    DocumentTracking {
        repository: String,
        classification: DocumentClassification,
    },
    ElectronicsInventory {
        datacenter: String,
        rack_id: Option<String>,
    },
    SupplyChainTracking {
        route: String,
        checkpoint: String,
        vehicle_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentClassification {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Position data in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub zone: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Item count by type and category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemCount {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub by_metal: HashMap<MetalType, usize>,
    pub high_value_items: usize,
    pub on_metal_items: usize,
}

/// Detailed item information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetail {
    pub uid: String,
    pub tag_type: String,
    pub metal_type: Option<MetalType>,
    pub signal_strength: i8,
    pub estimated_distance: f32,
    pub position: Option<Position3D>,
    pub last_seen: DateTime<Utc>,
    pub validation: ValidationResult,
    pub metadata: HashMap<String, String>,
}

/// Scan response with comprehensive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub scan_id: String,
    pub position: Position3D,
    pub count: ItemCount,
    pub items: Vec<ItemDetail>,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub zone: String,
    pub anomalies: Vec<AnomalyDetection>,
    pub insights: Vec<String>,
}

/// Anomaly detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub affected_items: Vec<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    MissingItem,
    DuplicateRead,
    WeakSignal,
    UnexpectedLocation,
    TamperingDetected,
    SignalInterference,
    MetalInterference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// RAG-enhanced RFID processor
pub struct RfidRagChaining {
    reader: RfidReader,
    qdrant_client: Arc<Qdrant>,
    use_case: RfidUseCase,
    cache: Arc<tokio::sync::RwLock<HashMap<String, ScanResponse>>>,
}

impl RfidRagChaining {
    pub fn new(
        reader_config: RfidReaderConfig,
        qdrant_client: Arc<Qdrant>,
        use_case: RfidUseCase,
    ) -> Self {
        Self {
            reader: RfidReader::new(reader_config),
            qdrant_client,
            use_case,
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the system
    pub async fn initialize(&mut self) -> Result<()> {
        println!("Initializing RFID RAG Chaining System...");
        println!("Use Case: {:?}", self.use_case);

        // Connect to RFID reader
        self.reader.connect().await?;

        // Ensure Qdrant collection exists
        self.ensure_qdrant_collection().await?;

        println!("RFID RAG System initialized successfully");
        Ok(())
    }

    /// Perform comprehensive scan with RAG processing
    pub async fn comprehensive_scan(&self) -> Result<ScanResponse> {
        let scan_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        println!("Starting comprehensive scan: {}", scan_id);

        // Scan for tags
        let raw_tags = self.reader.scan_for_tags().await?;
        println!("  Found {} raw tags", raw_tags.len());

        // Process tags with metal compensation and validation
        let mut processed_items = Vec::new();
        let mut anomalies = Vec::new();

        for tag in raw_tags {
            // Apply metal compensation if needed
            let mut processed_tag = tag;
            if let Err(e) = self.reader.apply_metal_compensation(&mut processed_tag) {
                anomalies.push(AnomalyDetection {
                    anomaly_type: AnomalyType::MetalInterference,
                    severity: AnomalySeverity::Medium,
                    description: format!("Failed to apply metal compensation: {}", e),
                    affected_items: vec![processed_tag.uid.clone()],
                    recommended_action: "Check tag placement and reader positioning".to_string(),
                });
            }

            // Validate tag
            let validation = self.reader.validate_valuable_tag(&processed_tag)?;

            // Create item detail
            let item_detail = ItemDetail {
                uid: processed_tag.uid.clone(),
                tag_type: format!("{:?}", processed_tag.tag_type),
                metal_type: if processed_tag.tag_type.is_on_metal() {
                    let metal_detection = self.reader.detect_metal_type(&processed_tag);
                    Some(metal_detection.metal_type)
                } else {
                    None
                },
                signal_strength: processed_tag.signal_strength.unwrap_or(-100),
                estimated_distance: self.reader.get_tag_read_range(&processed_tag),
                position: self.estimate_position(&processed_tag).await?,
                last_seen: processed_tag.last_seen,
                validation: validation.clone(),
                metadata: processed_tag.metadata,
            };

            // Check for anomalies
            if !validation.is_valid {
                anomalies.push(AnomalyDetection {
                    anomaly_type: AnomalyType::WeakSignal,
                    severity: AnomalySeverity::Medium,
                    description: "Tag validation failed".to_string(),
                    affected_items: vec![processed_tag.uid.clone()],
                    recommended_action: "Check tag condition and reader alignment".to_string(),
                });
            }

            processed_items.push(item_detail);
        }

        // Calculate position and count
        let position = self.calculate_aggregate_position(&processed_items).await?;
        let count = self.calculate_item_counts(&processed_items);

        // Generate insights using RAG
        let insights = self.generate_insights(&processed_items, &anomalies).await?;

        // Store in Qdrant for future reference
        self.store_scan_results(&scan_id, &processed_items, &position).await?;

        let response = ScanResponse {
            scan_id,
            position,
            count,
            items: processed_items,
            confidence: self.calculate_overall_confidence(&processed_items),
            timestamp,
            zone: self.get_zone_from_position(&position),
            anomalies,
            insights,
        };

        // Cache the response
        {
            let mut cache = self.cache.write().await;
            cache.insert(scan_id.clone(), response.clone());
        }

        println!(" Comprehensive scan completed: {} items, {} anomalies",
                response.items.len(), response.anomalies.len());

        Ok(response)
    }

    /// Track specific valuable item
    pub async fn track_valuable_item(&self, tag_uid: &str) -> Result<ItemDetail> {
        println!(" Tracking valuable item: {}", tag_uid);

        // Try to read specific tag
        let tag = self.reader.read_tag(tag_uid).await?;

        // Apply processing
        let mut processed_tag = tag;
        self.reader.apply_metal_compensation(&mut processed_tag)?;
        let validation = self.reader.validate_valuable_tag(&processed_tag)?;

        let item_detail = ItemDetail {
            uid: processed_tag.uid.clone(),
            tag_type: format!("{:?}", processed_tag.tag_type),
            metal_type: if processed_tag.tag_type.is_on_metal() {
                let metal_detection = self.reader.detect_metal_type(&processed_tag);
                Some(metal_detection.metal_type)
            } else {
                None
            },
            signal_strength: processed_tag.signal_strength.unwrap_or(-100),
            estimated_distance: self.reader.get_tag_read_range(&processed_tag),
            position: self.estimate_position(&processed_tag).await?,
            last_seen: processed_tag.last_seen,
            validation,
            metadata: processed_tag.metadata,
        };

        Ok(item_detail)
    }

    /// Get historical data for an item
    pub async fn get_item_history(&self, tag_uid: &str, limit: usize) -> Result<Vec<ScanResponse>> {
        println!(" Getting history for item: {} (limit: {})", tag_uid, limit);

        // Search Qdrant for historical scans containing this item
        let search_result = self.qdrant_client
            .search(SearchPointsBuilder::new(
                &format!("rfid_scans_{}", self.get_use_case_string()),
                vec![0.0; 384], // Dummy embedding for now
            )
            .limit(limit as u64)
            .with_filter(
                qdrant_client::qdrant::FilterBuilder::new()
                    .must_one_of(
                        qdrant_client::qdrant::ConditionBuilder::matches(
                            qdrant_client::qdrant::FieldConditionBuilder::new("tag_uid")
                                .with_match(qdrant_client::qdrant::MatchBuilder::new_text(tag_uid))
                                .build()
                        )
                    )
                    .build()
            )
            .build())
            .await
            .map_err(|e| anyhow!("Failed to search Qdrant: {}", e))?;

        // Convert search results to ScanResponse (simplified)
        let mut history = Vec::new();
        for _point in search_result.result {
            // TODO: Parse point data and reconstruct ScanResponse
            // For now, return empty history
        }

        Ok(history)
    }

    /// Generate analytics report
    pub async fn generate_analytics_report(&self, time_range: TimeRange) -> Result<AnalyticsReport> {
        println!(" Generating analytics report for {:?}", time_range);

        // Query historical data
        let historical_data = self.query_historical_data(&time_range).await?;

        // Generate insights
        let insights = self.generate_analytics_insights(&historical_data).await?;

        let report = AnalyticsReport {
            time_range,
            total_scans: historical_data.len(),
            unique_items: self.count_unique_items(&historical_data),
            item_types: self.analyze_item_types(&historical_data),
            metal_distribution: self.analyze_metal_distribution(&historical_data),
            zone_activity: self.analyze_zone_activity(&historical_data),
            anomaly_trends: self.analyze_anomaly_trends(&historical_data),
            insights,
            generated_at: Utc::now(),
        };

        Ok(report)
    }

    // Private helper methods

    async fn ensure_qdrant_collection(&self) -> Result<()> {
        let collection_name = format!("rfid_scans_{}", self.get_use_case_string());

        // Check if collection exists
        let collections = self.qdrant_client.list_collections().await?;
        let exists = collections.collections.iter()
            .any(|c| c.name == collection_name);

        if !exists {
            println!(" Creating Qdrant collection: {}", collection_name);

            self.qdrant_client.create_collection(
                qdrant_client::qdrant::CreateCollectionBuilder::new(&collection_name)
                    .vectors_config(VectorParams {
                        size: 384, // Embedding dimension
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })
            ).await?;
        }

        Ok(())
    }

    fn get_use_case_string(&self) -> String {
        match &self.use_case {
            RfidUseCase::ValuablesTracking { facility, .. } => format!("valuables_{}", facility),
            RfidUseCase::JewelryInventory { store_id, .. } => format!("jewelry_{}", store_id),
            RfidUseCase::DocumentTracking { repository, .. } => format!("docs_{}", repository),
            RfidUseCase::ElectronicsInventory { datacenter, .. } => format!("electronics_{}", datacenter),
            RfidUseCase::SupplyChainTracking { route, .. } => format!("supplychain_{}", route),
            RfidUseCase::AccessControl { location } => format!("access_{}", location),
            RfidUseCase::InventoryManagement { warehouse } => format!("inventory_{}", warehouse),
            RfidUseCase::AssetTracking { facility } => format!("assets_{}", facility),
            RfidUseCase::AttendanceSystem { organization } => format!("attendance_{}", organization),
            RfidUseCase::PaymentSystem { merchant } => format!("payment_{}", merchant),
            RfidUseCase::SecurityAudit { system_name } => format!("security_{}", system_name),
        }
    }

    async fn estimate_position(&self, tag: &RfidTag) -> Result<Option<Position3D>> {
        // Simplified position estimation based on signal strength
        if let Some(rssi) = tag.signal_strength {
            if let Some(ref uhf_data) = tag.uhf_data {
                // Use RSSI and phase angle for triangulation
                let distance = self.rssi_to_distance(rssi);
                let angle = uhf_data.phase_angle.unwrap_or(0.0);

                let x = distance * angle.cos();
                let y = distance * angle.sin();
                let z = 1.0; // Default height

                Ok(Some(Position3D {
                    x,
                    y,
                    z,
                    zone: self.get_zone_from_coordinates(x, y),
                    confidence: self.calculate_position_confidence(rssi),
                    timestamp: Utc::now(),
                }))
            } else {
                // NFC or other tag types - simpler positioning
                let distance = self.rssi_to_distance(rssi);
                Ok(Some(Position3D {
                    x: distance,
                    y: 0.0,
                    z: 1.0,
                    zone: "Unknown".to_string(),
                    confidence: 0.5,
                    timestamp: Utc::now(),
                }))
            }
        } else {
            Ok(None)
        }
    }

    fn rssi_to_distance(&self, rssi: i8) -> f64 {
        // Simplified RSSI to distance conversion
        // Real implementation would use calibration data
        let normalized_rssi = (rssi + 100) as f64 / 50.0; // Normalize -100 to -50 -> 0.0 to 1.0
        (1.0 - normalized_rssi) * 15.0 // Max 15 meters for UHF
    }

    fn get_zone_from_coordinates(&self, x: f64, y: f64) -> String {
        // Simple zone determination based on coordinates
        match (x as i32, y as i32) {
            (0..=5, 0..=5) => "Zone_A".to_string(),
            (6..=10, 0..=5) => "Zone_B".to_string(),
            (0..=5, 6..=10) => "Zone_C".to_string(),
            (6..=10, 6..=10) => "Zone_D".to_string(),
            _ => "Zone_Unknown".to_string(),
        }
    }

    fn calculate_position_confidence(&self, rssi: i8) -> f32 {
        match rssi {
            rssi if rssi > -50 => 0.95,
            rssi if rssi > -60 => 0.85,
            rssi if rssi > -70 => 0.75,
            rssi if rssi > -80 => 0.60,
            _ => 0.40,
        }
    }

    async fn calculate_aggregate_position(&self, items: &[ItemDetail]) -> Result<Position3D> {
        if items.is_empty() {
            return Err(anyhow!("No items to calculate position"));
        }

        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        let mut z_sum = 0.0;
        let mut confidence_sum = 0.0;
        let mut valid_positions = 0;

        for item in items {
            if let Some(pos) = &item.position {
                x_sum += pos.x;
                y_sum += pos.y;
                z_sum += pos.z;
                confidence_sum += pos.confidence;
                valid_positions += 1;
            }
        }

        if valid_positions == 0 {
            return Err(anyhow!("No valid positions found"));
        }

        let count = valid_positions as f64;
        Ok(Position3D {
            x: x_sum / count,
            y: y_sum / count,
            z: z_sum / count,
            zone: self.get_zone_from_coordinates(x_sum / count, y_sum / count),
            confidence: (confidence_sum / valid_positions as f32) * 0.9, // Slightly reduce for aggregate
            timestamp: Utc::now(),
        })
    }

    fn calculate_item_counts(&self, items: &[ItemDetail]) -> ItemCount {
        let mut by_type = HashMap::new();
        let mut by_metal = HashMap::new();
        let mut high_value_items = 0;
        let mut on_metal_items = 0;

        for item in items {
            // Count by type
            *by_type.entry(item.tag_type.clone()).or_insert(0) += 1;

            // Count by metal type
            if let Some(metal_type) = &item.metal_type {
                *by_metal.entry(metal_type.clone()).or_insert(0) += 1;
                on_metal_items += 1;

                // High value metals
                match metal_type {
                    MetalType::Gold | MetalType::Platinum => high_value_items += 1,
                    _ => {}
                }
            }

            // High value items based on validation
            if item.validation.confidence > 0.8 {
                high_value_items += 1;
            }
        }

        ItemCount {
            total: items.len(),
            by_type,
            by_metal,
            high_value_items,
            on_metal_items,
        }
    }

    fn calculate_overall_confidence(&self, items: &[ItemDetail]) -> f32 {
        if items.is_empty() {
            return 0.0;
        }

        let confidence_sum: f32 = items.iter()
            .map(|item| item.validation.confidence)
            .sum();

        confidence_sum / items.len() as f32
    }

    fn get_zone_from_position(&self, position: &Position3D) -> String {
        position.zone.clone()
    }

    async fn generate_insights(&self, items: &[ItemDetail], anomalies: &[AnomalyDetection]) -> Result<Vec<String>> {
        let mut insights = Vec::new();

        // Generate insights based on use case
        match &self.use_case {
            RfidUseCase::ValuablesTracking { facility, metal_type, .. } => {
                insights.push(format!("Scanning {} facility for {:?} items", facility, metal_type));
                insights.push(format!("Found {} items with {} anomalies", items.len(), anomalies.len()));

                let metal_count = items.iter().filter(|i| i.metal_type.as_ref() == Some(metal_type)).count();
                insights.push(format!("{} {:?} items detected", metal_count, metal_type));
            }
            RfidUseCase::JewelryInventory { store_id, display_case } => {
                insights.push(format!("Jewelry inventory for store {}", store_id));
                if let Some(case) = display_case {
                    insights.push(format!("Display case: {}", case));
                }

                let gold_items = items.iter().filter(|i| i.metal_type == Some(MetalType::Gold)).count();
                let silver_items = items.iter().filter(|i| i.metal_type == Some(MetalType::Silver)).count();
                insights.push(format!("Gold items: {}, Silver items: {}", gold_items, silver_items));
            }
            _ => {
                insights.push(format!("Scan completed with {} items", items.len()));
                insights.push(format!("Anomalies detected: {}", anomalies.len()));
            }
        }

        // Add anomaly insights
        for anomaly in anomalies.iter().take(5) {
            insights.push(format!(" {}: {}", anomaly.anomaly_type, anomaly.description));
        }

        Ok(insights)
    }

    async fn store_scan_results(&self, scan_id: &str, items: &[ItemDetail], position: &Position3D) -> Result<()> {
        // TODO: Implement actual storage in Qdrant
        // For now, just log the storage
        println!(" Storing scan results: {} items", items.len());
        Ok(())
    }

    async fn query_historical_data(&self, time_range: &TimeRange) -> Result<Vec<ScanResponse>> {
        // TODO: Implement historical data query
        println!(" Querying historical data for {:?}", time_range);
        Ok(Vec::new())
    }

    fn count_unique_items(&self, historical_data: &[ScanResponse]) -> usize {
        let mut unique_uids = std::collections::HashSet::new();
        for scan in historical_data {
            for item in &scan.items {
                unique_uids.insert(&item.uid);
            }
        }
        unique_uids.len()
    }

    fn analyze_item_types(&self, historical_data: &[ScanResponse]) -> HashMap<String, usize> {
        let mut type_counts = HashMap::new();
        for scan in historical_data {
            for item in &scan.items {
                *type_counts.entry(item.tag_type.clone()).or_insert(0) += 1;
            }
        }
        type_counts
    }

    fn analyze_metal_distribution(&self, historical_data: &[ScanResponse]) -> HashMap<MetalType, usize> {
        let mut metal_counts = HashMap::new();
        for scan in historical_data {
            for item in &scan.items {
                if let Some(metal_type) = &item.metal_type {
                    *metal_counts.entry(metal_type.clone()).or_insert(0) += 1;
                }
            }
        }
        metal_counts
    }

    fn analyze_zone_activity(&self, historical_data: &[ScanResponse]) -> HashMap<String, usize> {
        let mut zone_counts = HashMap::new();
        for scan in historical_data {
            *zone_counts.entry(scan.zone.clone()).or_insert(0) += 1;
        }
        zone_counts
    }

    fn analyze_anomaly_trends(&self, historical_data: &[ScanResponse]) -> HashMap<AnomalyType, usize> {
        let mut anomaly_counts = HashMap::new();
        for scan in historical_data {
            for anomaly in &scan.anomalies {
                *anomaly_counts.entry(anomaly.anomaly_type.clone()).or_insert(0) += 1;
            }
        }
        anomaly_counts
    }

    async fn generate_analytics_insights(&self, historical_data: &[ScanResponse]) -> Result<Vec<String>> {
        let mut insights = Vec::new();

        insights.push(format!("Analyzed {} historical scans", historical_data.len()));

        let total_items: usize = historical_data.iter().map(|s| s.items.len()).sum();
        insights.push(format!("Total items processed: {}", total_items));

        let total_anomalies: usize = historical_data.iter().map(|s| s.anomalies.len()).sum();
        insights.push(format!("Total anomalies detected: {}", total_anomalies));

        Ok(insights)
    }
}

// Additional data structures for analytics

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub time_range: TimeRange,
    pub total_scans: usize,
    pub unique_items: usize,
    pub item_types: HashMap<String, usize>,
    pub metal_distribution: HashMap<MetalType, usize>,
    pub zone_activity: HashMap<String, usize>,
    pub anomaly_trends: HashMap<AnomalyType, usize>,
    pub insights: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

// API endpoints for integration

/// Scan API handler
pub async fn scan_api_handler(
    rag_chaining: Arc<RfidRagChaining>,
) -> Result<ScanResponse> {
    rag_chaining.comprehensive_scan().await
}

/// Track specific item API handler
pub async fn track_item_api_handler(
    rag_chaining: Arc<RfidRagChaining>,
    tag_uid: String,
) -> Result<ItemDetail> {
    rag_chaining.track_valuable_item(&tag_uid).await
}

/// Get item history API handler
pub async fn item_history_api_handler(
    rag_chaining: Arc<RfidRagChaining>,
    tag_uid: String,
    limit: Option<usize>,
) -> Result<Vec<ScanResponse>> {
    let limit = limit.unwrap_or(100);
    rag_chaining.get_item_history(&tag_uid, limit).await
}

/// Analytics API handler
pub async fn analytics_api_handler(
    rag_chaining: Arc<RfidRagChaining>,
    time_range: TimeRange,
) -> Result<AnalyticsReport> {
    rag_chaining.generate_analytics_report(time_range).await
}
