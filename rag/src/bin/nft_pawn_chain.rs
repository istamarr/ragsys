//! NFT Pawnbroking / Collateralized Lending Chain Binary
//!
//! Features:
//! 1. burn-lm ASIST model → Ollama tamar:1b fallback
//! 2. IPFS/Iroh Local storage for NFT metadata & documents
//! 3. Struk (receipt) generation in Bahasa Indonesia
//! 4. Collateral-ready & Marketplace-ready documents
//!
//! Usage: cargo run --bin nft_pawn_chain [OPTIONS] [MODE]
//!
//! Options:
//!   --router <ROUTER>  Router type: ipfs or iroh (default: ipfs)
//!
//! Examples:
//!   cargo run --bin nft_pawn_chain --router iroh dev
//!   cargo run --bin nft_pawn_chain --router ipfs pawn

use std::env;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;
use std::time::{UNIX_EPOCH};
use dotenv::dotenv;
use anyhow::{Result, Context};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use chrono::{DateTime, Local};
use uuid::Uuid;
use sha2::{Sha256, Digest};

// ============================================================================
// CLI ARGUMENTS
// ============================================================================

/// Router type for content storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum RouterType {
    /// Use IPFS for content routing and storage
    #[default]
    Ipfs,
    /// Use Iroh for content routing and storage
    Iroh,
}

impl std::fmt::Display for RouterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterType::Ipfs => write!(f, "ipfs"),
            RouterType::Iroh => write!(f, "iroh"),
        }
    }
}

/// NFT Pawnbroking / Collateralized Lending Chain CLI
#[derive(Parser, Debug)]
#[command(name = "nft_pawn_chain")]
#[command(author = "istamar")]
#[command(version = "1.0")]
#[command(about = "NFT Pawnbroking Chain with IPFS/Iroh storage", long_about = None)]
pub struct CliArgs {
    /// Router type for content storage (ipfs or iroh)
    #[arg(short, long, value_enum, default_value_t = RouterType::Ipfs)]
    pub router: RouterType,

    /// Mode to run: dev, interactive, pawn, rfid, help
    #[arg(default_value = "dev")]
    pub mode: String,
}

/// Global router configuration (thread-safe)
static ROUTER_TYPE: std::sync::OnceLock<RouterType> = std::sync::OnceLock::new();

/// Get current router type
pub fn get_router_type() -> RouterType {
    *ROUTER_TYPE.get().unwrap_or(&RouterType::Ipfs)
}

/// Check if using Iroh router
pub fn is_iroh_router() -> bool {
    get_router_type() == RouterType::Iroh
}

/// Check if using IPFS router
pub fn is_ipfs_router() -> bool {
    get_router_type() == RouterType::Ipfs
}

// Import from main crate
use rag::rag_chaining::base_chain::{
    nft_pawn_chain,
    query_nft_pawn,
    NftPawnRequest,
    NftPawnResult,
};

// Agent Protocol Integration for NFT indexing and search
use rag::rag_agent::agent_web3::agent_protocol_client::{
    AgentProtocolClient,
    NftFlowRequest,
    NftMetadataPayload,
    NftAttributePayload,
    struk_to_nft_request,
};

// ============================================================================
// RFID COLLATERAL TRACKING SYSTEM
// ============================================================================

/// RFID Tag data for physical collateral tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidTag {
    pub tag_id: String,
    pub collateral_id: String,
    pub struk_nomor: Option<String>,
    pub registered_at: String,
    pub last_scan: String,
    pub status: RfidStatus,
}

/// RFID scan status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RfidStatus {
    Active,
    InTransit,
    Stored,
    Released,
    Missing,
}

/// Collateral tracking record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralTracking {
    pub tracking_id: String,
    pub rfid_tag: RfidTag,
    pub collateral_type: String,
    pub description: String,
    pub current_location: LocationInfo,
    pub condition: ConditionInfo,
    pub delivery_history: Vec<DeliveryRecord>,
    pub placement_info: PlacementInfo,
    pub created_at: String,
    pub updated_at: String,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub location_code: String,
    pub location_name: String,
    pub warehouse: String,
    pub zone: String,
    pub shelf: Option<String>,
    pub gps_coordinates: Option<String>,
    pub updated_at: String,
}

/// Condition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionInfo {
    pub status: String,          // Baik, Rusak Ringan, Rusak Berat
    pub grade: String,           // A, B, C, D
    pub notes: String,
    pub inspected_by: String,
    pub inspected_at: String,
    pub photos: Vec<String>,
}

/// Delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub delivery_id: String,
    pub from_location: String,
    pub to_location: String,
    pub courier: String,
    pub departed_at: String,
    pub arrived_at: Option<String>,
    pub status: DeliveryStatus,
    pub notes: String,
}

/// Delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    InTransit,
    Delivered,
    Failed,
}

/// Placement information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementInfo {
    pub placement_id: String,
    pub storage_type: String,    // Brankas, Rak, Safe Deposit Box
    pub security_level: String,  // Standard, High, Maximum
    pub assigned_to: String,
    pub valid_until: String,
    pub special_instructions: String,
}

/// RFID Tracking Manager
pub struct RfidTrackingManager {
    storage_path: PathBuf,
    tracking_records: Vec<CollateralTracking>,
}

impl RfidTrackingManager {
    pub fn new() -> Result<Self> {
        let storage_path = PathBuf::from("./output/rfid_tracking");
        fs::create_dir_all(&storage_path)?;

        // Load existing records
        let records = Self::load_records(&storage_path)?;

        Ok(Self {
            storage_path,
            tracking_records: records,
        })
    }

    fn load_records(path: &PathBuf) -> Result<Vec<CollateralTracking>> {
        let index_file = path.join("tracking_index.json");
        if index_file.exists() {
            let content = fs::read_to_string(&index_file)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(Vec::new())
        }
    }

    fn save_records(&self) -> Result<()> {
        let index_file = self.storage_path.join("tracking_index.json");
        let content = serde_json::to_string_pretty(&self.tracking_records)?;
        fs::write(index_file, content)?;
        Ok(())
    }

    /// Register new RFID tag for collateral
    pub fn register_rfid(&mut self,
                         rfid_tag_id: &str,
                         collateral_type: &str,
                         description: &str,
                         initial_location: &str,
                         warehouse: &str,
    ) -> Result<CollateralTracking> {
        let now = Local::now();
        let tracking_id = format!("TRK-{}-{}", now.format("%Y%m%d"), &Uuid::new_v4().to_string()[..8].to_uppercase());

        let tracking = CollateralTracking {
            tracking_id: tracking_id.clone(),
            rfid_tag: RfidTag {
                tag_id: rfid_tag_id.to_string(),
                collateral_id: format!("COL-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
                struk_nomor: None,
                registered_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_scan: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                status: RfidStatus::Active,
            },
            collateral_type: collateral_type.to_string(),
            description: description.to_string(),
            current_location: LocationInfo {
                location_code: format!("LOC-{}", &Uuid::new_v4().to_string()[..6].to_uppercase()),
                location_name: initial_location.to_string(),
                warehouse: warehouse.to_string(),
                zone: "A".to_string(),
                shelf: None,
                gps_coordinates: None,
                updated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            },
            condition: ConditionInfo {
                status: "Baik".to_string(),
                grade: "A".to_string(),
                notes: "Kondisi awal saat registrasi".to_string(),
                inspected_by: "System".to_string(),
                inspected_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                photos: vec![],
            },
            delivery_history: vec![],
            placement_info: PlacementInfo {
                placement_id: format!("PLC-{}", &Uuid::new_v4().to_string()[..6].to_uppercase()),
                storage_type: "Rak".to_string(),
                security_level: "Standard".to_string(),
                assigned_to: "Warehouse Staff".to_string(),
                valid_until: (now + chrono::Duration::days(365)).format("%Y-%m-%d").to_string(),
                special_instructions: "".to_string(),
            },
            created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.tracking_records.push(tracking.clone());
        self.save_records()?;

        Ok(tracking)
    }

    /// Scan RFID tag and get tracking info
    pub fn scan_rfid(&mut self, rfid_tag_id: &str) -> Option<&mut CollateralTracking> {
        let now = Local::now();
        for record in &mut self.tracking_records {
            if record.rfid_tag.tag_id == rfid_tag_id {
                record.rfid_tag.last_scan = now.format("%Y-%m-%d %H:%M:%S").to_string();
                return Some(record);
            }
        }
        None
    }

    /// Update collateral location
    pub fn update_location(&mut self,
                           rfid_tag_id: &str,
                           new_location: &str,
                           warehouse: &str,
                           zone: &str,
                           shelf: Option<&str>,
    ) -> Result<()> {
        let now = Local::now();

        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            record.current_location = LocationInfo {
                location_code: format!("LOC-{}", &Uuid::new_v4().to_string()[..6].to_uppercase()),
                location_name: new_location.to_string(),
                warehouse: warehouse.to_string(),
                zone: zone.to_string(),
                shelf: shelf.map(|s| s.to_string()),
                gps_coordinates: None,
                updated_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            record.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
            record.rfid_tag.last_scan = now.format("%Y-%m-%d %H:%M:%S").to_string();
            self.save_records()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Update collateral condition
    pub fn update_condition(&mut self,
                            rfid_tag_id: &str,
                            status: &str,
                            grade: &str,
                            notes: &str,
                            inspected_by: &str,
    ) -> Result<()> {
        let now = Local::now();

        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            record.condition = ConditionInfo {
                status: status.to_string(),
                grade: grade.to_string(),
                notes: notes.to_string(),
                inspected_by: inspected_by.to_string(),
                inspected_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                photos: record.condition.photos.clone(),
            };
            record.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
            self.save_records()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Create delivery record
    pub fn create_delivery(&mut self,
                           rfid_tag_id: &str,
                           from_location: &str,
                           to_location: &str,
                           courier: &str,
    ) -> Result<DeliveryRecord> {
        let now = Local::now();

        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            let delivery = DeliveryRecord {
                delivery_id: format!("DLV-{}-{}", now.format("%Y%m%d"), &Uuid::new_v4().to_string()[..6].to_uppercase()),
                from_location: from_location.to_string(),
                to_location: to_location.to_string(),
                courier: courier.to_string(),
                departed_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                arrived_at: None,
                status: DeliveryStatus::InTransit,
                notes: "".to_string(),
            };

            record.delivery_history.push(delivery.clone());
            record.rfid_tag.status = RfidStatus::InTransit;
            record.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
            self.save_records()?;

            Ok(delivery)
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Confirm delivery arrival
    pub fn confirm_delivery(&mut self, rfid_tag_id: &str, delivery_id: &str) -> Result<()> {
        let now = Local::now();

        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            if let Some(delivery) = record.delivery_history.iter_mut().find(|d| d.delivery_id == delivery_id) {
                delivery.arrived_at = Some(now.format("%Y-%m-%d %H:%M:%S").to_string());
                delivery.status = DeliveryStatus::Delivered;

                // Update location to destination
                record.current_location.location_name = delivery.to_location.clone();
                record.current_location.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
                record.rfid_tag.status = RfidStatus::Stored;
                record.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();

                self.save_records()?;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Delivery not found: {}", delivery_id))
            }
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Update placement info
    pub fn update_placement(&mut self,
                            rfid_tag_id: &str,
                            storage_type: &str,
                            security_level: &str,
                            assigned_to: &str,
                            special_instructions: &str,
    ) -> Result<()> {
        let now = Local::now();

        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            record.placement_info = PlacementInfo {
                placement_id: format!("PLC-{}", &Uuid::new_v4().to_string()[..6].to_uppercase()),
                storage_type: storage_type.to_string(),
                security_level: security_level.to_string(),
                assigned_to: assigned_to.to_string(),
                valid_until: (now + chrono::Duration::days(365)).format("%Y-%m-%d").to_string(),
                special_instructions: special_instructions.to_string(),
            };
            record.updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
            self.save_records()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Link RFID to Struk Gadai
    pub fn link_to_struk(&mut self, rfid_tag_id: &str, struk_nomor: &str) -> Result<()> {
        if let Some(record) = self.tracking_records.iter_mut().find(|r| r.rfid_tag.tag_id == rfid_tag_id) {
            record.rfid_tag.struk_nomor = Some(struk_nomor.to_string());
            record.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            self.save_records()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("RFID tag not found: {}", rfid_tag_id))
        }
    }

    /// Get all tracking records
    pub fn list_all(&self) -> &Vec<CollateralTracking> {
        &self.tracking_records
    }

    /// Print tracking info
    pub fn print_tracking_info(tracking: &CollateralTracking) {
        println!("\n{}", "=".repeat(70));
        println!("                    COLLATERAL TRACKING INFO                         ");
        println!("{}", "=".repeat(70));

        println!("\n[RFID TAG]");
        println!("   Tag ID         : {}", tracking.rfid_tag.tag_id);
        println!("   Collateral ID  : {}", tracking.rfid_tag.collateral_id);
        println!("   Struk Nomor    : {}", tracking.rfid_tag.struk_nomor.as_deref().unwrap_or("Not linked"));
        println!("   Status         : {:?}", tracking.rfid_tag.status);
        println!("   Last Scan      : {}", tracking.rfid_tag.last_scan);

        println!("\n[COLLATERAL]");
        println!("   Type           : {}", tracking.collateral_type);
        println!("   Description    : {}", tracking.description);

        println!("\n[CURRENT LOCATION]");
        println!("   Location       : {}", tracking.current_location.location_name);
        println!("   Warehouse      : {}", tracking.current_location.warehouse);
        println!("   Zone           : {}", tracking.current_location.zone);
        println!("   Shelf          : {}", tracking.current_location.shelf.as_deref().unwrap_or("-"));
        println!("   Updated        : {}", tracking.current_location.updated_at);

        println!("\n[CONDITION]");
        println!("   Status         : {}", tracking.condition.status);
        println!("   Grade          : {}", tracking.condition.grade);
        println!("   Notes          : {}", tracking.condition.notes);
        println!("   Inspected By   : {}", tracking.condition.inspected_by);
        println!("   Inspected At   : {}", tracking.condition.inspected_at);

        println!("\n[PLACEMENT]");
        println!("   Storage Type   : {}", tracking.placement_info.storage_type);
        println!("   Security Level : {}", tracking.placement_info.security_level);
        println!("   Assigned To    : {}", tracking.placement_info.assigned_to);
        println!("   Valid Until    : {}", tracking.placement_info.valid_until);

        if !tracking.delivery_history.is_empty() {
            println!("\n[DELIVERY HISTORY]");
            for (i, delivery) in tracking.delivery_history.iter().enumerate() {
                println!("   {}. {} -> {} ({:?})",
                         i + 1,
                         delivery.from_location,
                         delivery.to_location,
                         delivery.status
                );
                println!("      Courier: {} | Departed: {}",
                         delivery.courier,
                         delivery.departed_at
                );
                if let Some(ref arrived) = delivery.arrived_at {
                    println!("      Arrived: {}", arrived);
                }
            }
        }

        println!("\n{}", "=".repeat(70));
    }
}

// ============================================================================
// IPFS LOCAL CLIENT
// ============================================================================

// ============================================================================
// IROH LOCAL CLIENT
// ============================================================================

/// Iroh Local Client for storing NFT Pawn documents
pub struct IrohLocalClient {
    api_url: String,
    client: HttpClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrohAddResponse {
    pub hash: String,
    pub size: u64,
}

impl IrohLocalClient {
    pub fn new() -> Self {
        let api_url = env::var("IROH_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:4400".to_string());

        Self {
            api_url,
            client: HttpClient::new(),
        }
    }

    /// Check if Iroh daemon is running
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/health", self.api_url);
        match self.client.get(&url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => {
                // Try alternative endpoint
                let url2 = format!("{}/v0/node/status", self.api_url);
                match self.client.get(&url2).send().await {
                    Ok(r) => r.status().is_success(),
                    Err(_) => false,
                }
            }
        }
    }

    /// Add content to Iroh and return hash
    pub async fn add_content(&self, content: &str, _filename: &str) -> Result<IrohAddResponse> {
        let url = format!("{}/v0/blobs/add", self.api_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(content.to_string())
            .send()
            .await
            .context("Failed to connect to Iroh")?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Iroh add failed: {}", err));
        }

        let result: IrohAddResponse = response.json().await?;
        info!("Iroh: Stored content with hash: {}", result.hash);

        Ok(result)
    }

    /// Add JSON document to Iroh
    pub async fn add_json<T: Serialize>(&self, data: &T, filename: &str) -> Result<IrohAddResponse> {
        let json_content = serde_json::to_string_pretty(data)?;
        self.add_content(&json_content, filename).await
    }

    /// Get Iroh gateway URL for a hash
    pub fn get_gateway_url(&self, hash: &str) -> String {
        let gateway = env::var("IROH_GATEWAY")
            .unwrap_or_else(|_| "http://127.0.0.1:4401".to_string());
        format!("{}/blobs/{}", gateway, hash)
    }
}

// ============================================================================
// IPFS LOCAL CLIENT
// ============================================================================

/// IPFS Local Client for storing NFT Pawn documents
pub struct IpfsLocalClient {
    api_url: String,
    client: HttpClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsAddResponse {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Hash")]
    pub hash: String,
    #[serde(rename = "Size")]
    pub size: String,
}

impl IpfsLocalClient {
    pub fn new() -> Self {
        let api_url = env::var("IPFS_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());

        Self {
            api_url,
            client: HttpClient::new(),
        }
    }

    /// Check if IPFS daemon is running
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/v0/id", self.api_url);
        match self.client.post(&url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    /// Add content to IPFS and return CID (Content Identifier)
    pub async fn add_content(&self, content: &str, filename: &str) -> Result<IpfsAddResponse> {
        let url = format!("{}/api/v0/add?pin=true", self.api_url);

        let form = reqwest::multipart::Form::new()
            .text("file", content.to_string());

        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to connect to IPFS")?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("IPFS add failed: {}", err));
        }

        let result: IpfsAddResponse = response.json().await?;
        info!("IPFS: Stored {} with CID: {}", filename, result.hash);

        Ok(result)
    }

    /// Add JSON document to IPFS
    pub async fn add_json<T: Serialize>(&self, data: &T, filename: &str) -> Result<IpfsAddResponse> {
        let json_content = serde_json::to_string_pretty(data)?;
        self.add_content(&json_content, filename).await
    }

    /// Get IPFS gateway URL for a CID
    pub fn get_gateway_url(&self, cid: &str) -> String {
        let gateway = env::var("IPFS_GATEWAY")
            .unwrap_or_else(|_| "http://127.0.0.1:9393".to_string());
        format!("{}/ipfs/{}", gateway, cid)
    }

    /// Pin content to ensure persistence
    pub async fn pin(&self, cid: &str) -> Result<()> {
        let url = format!("{}/api/v0/pin/add?arg={}", self.api_url, cid);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            warn!("Failed to pin CID: {}", cid);
        }

        Ok(())
    }
}

// ============================================================================
// STRUK (RECEIPT) DOCUMENT - BAHASA INDONESIA
// ============================================================================

/// NFT Pawn Collateral Document (Struk Gadai NFT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrukGadaiNft {
    // Header
    pub nomor_struk: String,
    pub tanggal_terbit: String,
    pub waktu_terbit: String,

    // Pemilik / Owner
    pub nama_pemilik: String,
    pub alamat_wallet: String,

    // Detail NFT
    pub detail_nft: DetailNft,

    // Detail Pinjaman
    pub detail_pinjaman: DetailPinjaman,

    // Penilaian Risiko
    pub penilaian_risiko: PenilaianRisiko,

    // Status & Metadata
    pub status: StatusStruk,
    pub metadata: MetadataStruk,

    // Tanda Tangan Digital
    pub tanda_tangan_digital: TandaTanganDigital,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailNft {
    pub nama_koleksi: String,
    pub token_id: String,
    pub contract_address: String,
    pub blockchain: String,
    pub standard: String,  // ERC-721, ERC-1155
    pub deskripsi: String,
    pub gambar_url: Option<String>,
    pub atribut: Vec<AtributNft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtributNft {
    pub nama: String,
    pub nilai: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailPinjaman {
    pub nilai_taksiran: f64,
    pub mata_uang: String,  // ETH, USDC, IDR
    pub ltv_ratio: f64,     // Loan-to-Value ratio (0.0 - 1.0)
    pub jumlah_pinjaman: f64,
    pub suku_bunga_tahunan: f64,
    pub tenor_hari: u32,
    pub tanggal_jatuh_tempo: String,
    pub platform_lending: String,
    pub biaya_admin: f64,
    pub total_pelunasan: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenilaianRisiko {
    pub tingkat_risiko: String,  // Rendah, Sedang, Tinggi
    pub risiko_volatilitas: String,
    pub risiko_likuidasi: String,
    pub risiko_smart_contract: String,
    pub rekomendasi: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusStruk {
    pub status_dokumen: String,  // Draft, Aktif, Lunas, Default
    pub siap_jaminan: bool,
    pub siap_marketplace: bool,
    pub terverifikasi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataStruk {
    pub versi_dokumen: String,
    pub ipfs_cid: Option<String>,
    pub ipfs_url: Option<String>,
    pub hash_dokumen: String,
    pub dibuat_oleh: String,
    pub catatan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TandaTanganDigital {
    pub algoritma: String,
    pub hash_konten: String,
    pub timestamp: u64,
    pub valid: bool,
}

// ============================================================================
// DOCUMENT GENERATOR
// ============================================================================

/// Generate Struk Gadai NFT from input
pub fn generate_struk_gadai(
    nama_pemilik: &str,
    alamat_wallet: &str,
    nama_koleksi: &str,
    token_id: &str,
    contract_address: &str,
    nilai_taksiran: f64,
    ltv_ratio: f64,
    tenor_hari: u32,
    suku_bunga: f64,
    platform: &str,
) -> StrukGadaiNft {
    let now = Local::now();
    let nomor_struk = format!("SGN-{}-{}",
                              now.format("%Y%m%d"),
                              Uuid::new_v4().to_string()[..8].to_uppercase()
    );

    let jumlah_pinjaman = nilai_taksiran * ltv_ratio;
    let bunga_total = jumlah_pinjaman * (suku_bunga / 100.0) * (tenor_hari as f64 / 365.0);
    let biaya_admin = jumlah_pinjaman * 0.01; // 1% admin fee
    let total_pelunasan = jumlah_pinjaman + bunga_total + biaya_admin;

    let jatuh_tempo = now + chrono::Duration::days(tenor_hari as i64);

    // Generate document hash
    let hash_input = format!("{}{}{}{}{}",
                             nomor_struk, alamat_wallet, token_id, nilai_taksiran, now.timestamp()
    );
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let hash_dokumen = format!("{:x}", hasher.finalize());

    StrukGadaiNft {
        nomor_struk: nomor_struk.clone(),
        tanggal_terbit: now.format("%d-%m-%Y").to_string(),
        waktu_terbit: now.format("%H:%M:%S WIB").to_string(),

        nama_pemilik: nama_pemilik.to_string(),
        alamat_wallet: alamat_wallet.to_string(),

        detail_nft: DetailNft {
            nama_koleksi: nama_koleksi.to_string(),
            token_id: token_id.to_string(),
            contract_address: contract_address.to_string(),
            blockchain: "Ethereum".to_string(),
            standard: "ERC-721".to_string(),
            deskripsi: format!("{} #{}", nama_koleksi, token_id),
            gambar_url: None,
            atribut: vec![],
        },

        detail_pinjaman: DetailPinjaman {
            nilai_taksiran,
            mata_uang: "ETH".to_string(),
            ltv_ratio,
            jumlah_pinjaman,
            suku_bunga_tahunan: suku_bunga,
            tenor_hari,
            tanggal_jatuh_tempo: jatuh_tempo.format("%d-%m-%Y").to_string(),
            platform_lending: platform.to_string(),
            biaya_admin,
            total_pelunasan,
        },

        penilaian_risiko: PenilaianRisiko {
            tingkat_risiko: if ltv_ratio <= 0.3 { "Rendah" }
            else if ltv_ratio <= 0.5 { "Sedang" }
            else { "Tinggi" }.to_string(),
            risiko_volatilitas: "Sedang - NFT bersifat volatile".to_string(),
            risiko_likuidasi: format!("LTV {}% memberikan buffer {}%",
                                      (ltv_ratio * 100.0) as u32, ((1.0 - ltv_ratio) * 100.0) as u32),
            risiko_smart_contract: "Rendah - Platform terverifikasi".to_string(),
            rekomendasi: vec![
                "Monitor harga floor secara berkala".to_string(),
                "Siapkan dana pelunasan sebelum jatuh tempo".to_string(),
                "Pertimbangkan perpanjangan jika diperlukan".to_string(),
            ],
        },

        status: StatusStruk {
            status_dokumen: "Draft".to_string(),
            siap_jaminan: true,
            siap_marketplace: true,
            terverifikasi: false,
        },

        metadata: MetadataStruk {
            versi_dokumen: "1.0".to_string(),
            ipfs_cid: None,
            ipfs_url: None,
            hash_dokumen: hash_dokumen.clone(),
            dibuat_oleh: "NFT Pawn Chain RAG System".to_string(),
            catatan: None,
        },

        tanda_tangan_digital: TandaTanganDigital {
            algoritma: "SHA-256".to_string(),
            hash_konten: hash_dokumen,
            timestamp: now.timestamp() as u64,
            valid: true,
        },
    }
}

/// Generate printable Struk text in Bahasa Indonesia
pub fn generate_struk_text(struk: &StrukGadaiNft) -> String {
    format!(r#"
╔══════════════════════════════════════════════════════════════════╗
║                    STRUK GADAI NFT                               ║
║                 (NFT Pawn Receipt)                               ║
╠══════════════════════════════════════════════════════════════════╣
║  Nomor Struk    : {nomor_struk:<45} ║
║  Tanggal        : {tanggal} {waktu:<30} ║
╠══════════════════════════════════════════════════════════════════╣
║  INFORMASI PEMILIK                                               ║
╠══════════════════════════════════════════════════════════════════╣
║  Nama           : {nama_pemilik:<45} ║
║  Alamat Wallet  : {alamat_wallet:<45} ║
╠══════════════════════════════════════════════════════════════════╣
║  DETAIL NFT YANG DIGADAIKAN                                      ║
╠══════════════════════════════════════════════════════════════════╣
║  Koleksi        : {nama_koleksi:<45} ║
║  Token ID       : {token_id:<45} ║
║  Contract       : {contract:<45} ║
║  Blockchain     : {blockchain:<45} ║
║  Standard       : {standard:<45} ║
╠══════════════════════════════════════════════════════════════════╣
║  DETAIL PINJAMAN                                                 ║
╠══════════════════════════════════════════════════════════════════╣
║  Nilai Taksiran : {nilai_taksiran:>15.4} {mata_uang:<26} ║
║  LTV Ratio      : {ltv_ratio:>15.0}%{space:<26} ║
║  Jumlah Pinjaman: {jumlah_pinjaman:>15.4} {mata_uang2:<26} ║
║  Suku Bunga     : {suku_bunga:>15.1}% per tahun{space2:<15} ║
║  Tenor          : {tenor:>15} hari{space3:<22} ║
║  Jatuh Tempo    : {jatuh_tempo:<45} ║
║  Platform       : {platform:<45} ║
║  ──────────────────────────────────────────────────────────────  ║
║  Biaya Admin    : {biaya_admin:>15.4} {mata_uang3:<26} ║
║  TOTAL PELUNASAN: {total_pelunasan:>15.4} {mata_uang4:<26} ║
╠══════════════════════════════════════════════════════════════════╣
║  PENILAIAN RISIKO                                                ║
╠══════════════════════════════════════════════════════════════════╣
║  Tingkat Risiko : {tingkat_risiko:<45} ║
║  Volatilitas    : {risiko_volatilitas:<45} ║
║  Likuidasi      : {risiko_likuidasi:<45} ║
╠══════════════════════════════════════════════════════════════════╣
║  STATUS DOKUMEN                                                  ║
╠══════════════════════════════════════════════════════════════════╣
║  Status         : {status_dokumen:<45} ║
║  Siap Jaminan   : {siap_jaminan:<45} ║
║  Siap Jual      : {siap_marketplace:<45} ║
╠══════════════════════════════════════════════════════════════════╣
║  METADATA & VERIFIKASI                                           ║
╠══════════════════════════════════════════════════════════════════╣
║  Hash Dokumen   : {hash_short}...  ║
║  IPFS CID       : {ipfs_cid:<45} ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  Dokumen ini sah sebagai bukti gadai NFT dan dapat digunakan    ║
║  sebagai jaminan pinjaman atau dijual di marketplace.            ║
║                                                                  ║
║  Verifikasi: https://ipfs.io/ipfs/{ipfs_cid_short}              ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
"#,
            nomor_struk = struk.nomor_struk,
            tanggal = struk.tanggal_terbit,
            waktu = struk.waktu_terbit,
            nama_pemilik = truncate_str(&struk.nama_pemilik, 45),
            alamat_wallet = truncate_str(&struk.alamat_wallet, 45),
            nama_koleksi = truncate_str(&struk.detail_nft.nama_koleksi, 45),
            token_id = struk.detail_nft.token_id,
            contract = truncate_str(&struk.detail_nft.contract_address, 45),
            blockchain = struk.detail_nft.blockchain,
            standard = struk.detail_nft.standard,
            nilai_taksiran = struk.detail_pinjaman.nilai_taksiran,
            mata_uang = struk.detail_pinjaman.mata_uang,
            ltv_ratio = struk.detail_pinjaman.ltv_ratio * 100.0,
            space = "",
            jumlah_pinjaman = struk.detail_pinjaman.jumlah_pinjaman,
            mata_uang2 = struk.detail_pinjaman.mata_uang,
            suku_bunga = struk.detail_pinjaman.suku_bunga_tahunan,
            space2 = "",
            tenor = struk.detail_pinjaman.tenor_hari,
            space3 = "",
            jatuh_tempo = struk.detail_pinjaman.tanggal_jatuh_tempo,
            platform = struk.detail_pinjaman.platform_lending,
            biaya_admin = struk.detail_pinjaman.biaya_admin,
            mata_uang3 = struk.detail_pinjaman.mata_uang,
            total_pelunasan = struk.detail_pinjaman.total_pelunasan,
            mata_uang4 = struk.detail_pinjaman.mata_uang,
            tingkat_risiko = struk.penilaian_risiko.tingkat_risiko,
            risiko_volatilitas = truncate_str(&struk.penilaian_risiko.risiko_volatilitas, 45),
            risiko_likuidasi = truncate_str(&struk.penilaian_risiko.risiko_likuidasi, 45),
            status_dokumen = struk.status.status_dokumen,
            siap_jaminan = if struk.status.siap_jaminan { "[v] Ya" } else { "[x] Tidak" },
            siap_marketplace = if struk.status.siap_marketplace { "[v] Ya" } else { "[x] Tidak" },
            hash_short = &struk.metadata.hash_dokumen[..40],
            ipfs_cid = struk.metadata.ipfs_cid.as_deref().unwrap_or("Belum tersimpan di IPFS"),
            ipfs_cid_short = struk.metadata.ipfs_cid.as_deref().unwrap_or("N/A"),
    )
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}

// ============================================================================
// NFT PAWN FLOW WITH ROUTER STORAGE (IPFS/IROH)
// ============================================================================

/// Complete NFT Pawn Flow: Input → Analysis → Storage (IPFS/Iroh) → Struk Generation
pub async fn nft_pawn_flow_with_ipfs(
    nama_pemilik: &str,
    alamat_wallet: &str,
    nama_koleksi: &str,
    token_id: &str,
    contract_address: &str,
    nilai_taksiran: f64,
    ltv_ratio: f64,
    tenor_hari: u32,
    query: &str,
) -> Result<(StrukGadaiNft, NftPawnResult)> {
    let router_type = get_router_type();
    let router_name = match router_type {
        RouterType::Ipfs => "IPFS",
        RouterType::Iroh => "Iroh",
    };

    println!("\n[Flow] Starting NFT Pawn Flow with {} Storage...\n", router_name);

    // Step 1: Run NFT Pawn analysis
    println!("[Step 1] Running NFT Pawn Chain Analysis...");
    let request = NftPawnRequest {
        query: query.to_string(),
        nft_collection: Some(nama_koleksi.to_string()),
        loan_amount: Some(nilai_taksiran * ltv_ratio),
        collateral_type: Some("NFT PFP".to_string()),
        include_risk_analysis: true,
        include_market_data: true,
    };

    let analysis_result = nft_pawn_chain(request).await?;
    println!("[Step 1] [OK] Analysis complete via {}", analysis_result.source_model);

    // Step 2: Determine platform and interest rate from analysis
    let platform = if analysis_result.response.to_lowercase().contains("nftfi") {
        "NFTfi"
    } else if analysis_result.response.to_lowercase().contains("benddao") {
        "BendDAO"
    } else {
        "NFTfi"
    };

    let suku_bunga = if analysis_result.response.contains("50%") { 50.0 }
    else if analysis_result.response.contains("30%") { 30.0 }
    else { 40.0 }; // Default 40% APY

    // Step 3: Generate Struk
    println!("[Step 2] Generating Struk Gadai NFT...");
    let mut struk = generate_struk_gadai(
        nama_pemilik,
        alamat_wallet,
        nama_koleksi,
        token_id,
        contract_address,
        nilai_taksiran,
        ltv_ratio,
        tenor_hari,
        suku_bunga,
        platform,
    );

    // Update risk from analysis
    if let Some(ref risk) = analysis_result.risk_assessment {
        struk.penilaian_risiko.tingkat_risiko = risk.overall_risk_level.clone();
        struk.penilaian_risiko.risiko_volatilitas = risk.volatility_risk.clone();
        struk.penilaian_risiko.risiko_likuidasi = risk.liquidation_risk.clone();
    }

    // Add recommendations from analysis
    struk.penilaian_risiko.rekomendasi = analysis_result.recommendations.clone();

    println!("[Step 2] [OK] Struk generated: {}", struk.nomor_struk);

    // Step 3: Store based on router type (IPFS or Iroh)
    println!("[Step 3] Storing to {} Local...", router_name);

    match router_type {
        RouterType::Iroh => {
            // Use Iroh router
            let iroh = IrohLocalClient::new();

            if iroh.is_available().await {
                let filename = format!("{}.json", struk.nomor_struk);
                match iroh.add_json(&struk, &filename).await {
                    Ok(response) => {
                        struk.metadata.ipfs_cid = Some(response.hash.clone());
                        struk.metadata.ipfs_url = Some(iroh.get_gateway_url(&response.hash));
                        struk.status.status_dokumen = "Aktif".to_string();
                        struk.status.terverifikasi = true;

                        println!("[Step 3] [OK] Stored in Iroh: {}", response.hash);
                        println!("         Gateway URL: {}", iroh.get_gateway_url(&response.hash));
                    }
                    Err(e) => {
                        warn!("[Step 3] [!] Iroh storage failed: {}", e);
                        println!("[Step 3] [!] Iroh storage failed, saving locally...");
                        save_struk_locally(&struk)?;
                    }
                }
            } else {
                println!("[Step 3] [!] Iroh not available, saving locally...");
                save_struk_locally(&struk)?;
            }
        }
        RouterType::Ipfs => {
            // Use IPFS router (default)
            let ipfs = IpfsLocalClient::new();

            if ipfs.is_available().await {
                let filename = format!("{}.json", struk.nomor_struk);
                match ipfs.add_json(&struk, &filename).await {
                    Ok(response) => {
                        struk.metadata.ipfs_cid = Some(response.hash.clone());
                        struk.metadata.ipfs_url = Some(ipfs.get_gateway_url(&response.hash));
                        struk.status.status_dokumen = "Aktif".to_string();
                        struk.status.terverifikasi = true;

                        // Pin for persistence
                        let _ = ipfs.pin(&response.hash).await;

                        println!("[Step 3] [OK] Stored in IPFS: {}", response.hash);
                        println!("         Gateway URL: {}", ipfs.get_gateway_url(&response.hash));
                    }
                    Err(e) => {
                        warn!("[Step 3] [!] IPFS storage failed: {}", e);
                        println!("[Step 3] [!] IPFS storage failed, saving locally...");
                        save_struk_locally(&struk)?;
                    }
                }
            } else {
                println!("[Step 3] [!] IPFS not available, saving locally...");
                save_struk_locally(&struk)?;
            }
        }
    }

    // Step 4: Index in Agent Protocol (Qdrant for AI search)
    println!("[Step 4] Indexing in Agent Protocol for AI search...");
    let agent_client = AgentProtocolClient::new();

    if agent_client.is_available().await {
        // Build attributes from struk
        let attributes = vec![
            ("tingkat_risiko".to_string(), struk.penilaian_risiko.tingkat_risiko.clone()),
            ("ltv_ratio".to_string(), format!("{}%", (struk.detail_pinjaman.ltv_ratio * 100.0) as u32)),
            ("platform".to_string(), struk.detail_pinjaman.platform_lending.clone()),
            ("blockchain".to_string(), struk.detail_nft.blockchain.clone()),
            ("standard".to_string(), struk.detail_nft.standard.clone()),
        ];

        let ipfs_cid = struk.metadata.ipfs_cid.clone().unwrap_or_default();
        let nft_request = struk_to_nft_request(
            &struk.nomor_struk,
            &ipfs_cid,
            &struk.detail_nft.nama_koleksi,
            &struk.detail_nft.token_id,
            &struk.detail_nft.contract_address,
            &struk.alamat_wallet,
            &struk.detail_nft.deskripsi,
            &struk.detail_nft.blockchain,
            attributes,
        );

        match agent_client.index_nft(nft_request).await {
            Ok(response) => {
                println!("[Step 4] [OK] Indexed in Agent Protocol: {} NFT(s)", response.processed_count);
                println!("         Now searchable via: POST /scheme/nft/search");
            }
            Err(e) => {
                warn!("[Step 4] [!] Agent Protocol indexing failed: {}", e);
                println!("[Step 4] [!] Agent Protocol not available, skipping search indexing");
            }
        }
    } else {
        println!("[Step 4] [!] Agent Protocol not available, skipping search indexing");
        println!("         Set AGENT_PROTOCOL_URL env var to enable");
    }

    // Step 5: Generate printable struk
    println!("[Step 5] Generating printable Struk...\n");
    let struk_text = generate_struk_text(&struk);
    println!("{}", struk_text);

    println!("\n[Flow] [OK] NFT Pawn Flow Complete!");
    println!("       Document ready for: Collateral [v] | Marketplace [v] | AI Search [v]\n");

    Ok((struk, analysis_result))
}

/// Save struk locally if IPFS is not available
fn save_struk_locally(struk: &StrukGadaiNft) -> Result<()> {
    let output_dir = PathBuf::from("./output/struk");
    fs::create_dir_all(&output_dir)?;

    let json_path = output_dir.join(format!("{}.json", struk.nomor_struk));
    let txt_path = output_dir.join(format!("{}.txt", struk.nomor_struk));

    // Save JSON
    let json_content = serde_json::to_string_pretty(struk)?;
    fs::write(&json_path, &json_content)?;

    // Save printable text
    let txt_content = generate_struk_text(struk);
    fs::write(&txt_path, &txt_content)?;

    println!("         Saved to: {}", json_path.display());
    println!("         Saved to: {}", txt_path.display());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize
    dotenv().ok();
    env_logger::init();

    // Parse CLI arguments
    let cli = CliArgs::parse();

    // Set global router type
    let _ = ROUTER_TYPE.set(cli.router);

    let router_label = match cli.router {
        RouterType::Ipfs => "IPFS",
        RouterType::Iroh => "IROH",
    };

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║     NFT PAWNBROKING / COLLATERALIZED LENDING CHAIN               ║");
    println!("║     dengan {} Storage & Struk Gadai{}", router_label, " ".repeat(28 - router_label.len()));
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("  Router: {} | Mode: {}", cli.router, cli.mode);
    println!();

    let mode = cli.mode.as_str();

    match mode {
        "dev" | "d" => run_dev_mode().await?,
        "interactive" | "i" => interactive_mode().await?,
        "pawn" | "p" => run_pawn_flow().await?,
        "rfid" | "r" => rfid_mode().await?,
        "help" | "h" | "--help" => print_help(),
        _ => {
            println!("Mode tidak dikenal: {}", mode);
            print_help();
        }
    }

    Ok(())
}

fn print_help() {
    println!(r#"
PENGGUNAAN:
    cargo run --bin nft_pawn_chain [OPTIONS] [MODE]

OPTIONS:
    -r, --router <ROUTER>  Router type: ipfs atau iroh (default: ipfs)

MODE:
    dev, d          - Jalankan dev dengan contoh NFT (default)
    interactive, i  - Mode interaktif untuk query
    pawn, p         - Buat struk gadai NFT baru
    rfid, r         - RFID Collateral Tracking System
    help, h         - Tampilkan bantuan ini

CONTOH:
    cargo run --bin nft_pawn_chain dev
    cargo run --bin nft_pawn_chain --router iroh dev
    cargo run --bin nft_pawn_chain -r ipfs pawn
    cargo run --bin nft_pawn_chain --router iroh interactive
    cargo run --bin nft_pawn_chain rfid

RFID COMMANDS (dalam mode rfid):
    register        - Daftarkan RFID tag baru untuk jaminan
    scan            - Scan RFID tag dan lihat info tracking
    location        - Update lokasi jaminan
    condition       - Update kondisi jaminan
    delivery        - Buat pengiriman baru
    confirm         - Konfirmasi pengiriman tiba
    placement       - Update penempatan/penyimpanan
    link            - Hubungkan RFID ke Struk Gadai
    list            - Lihat semua jaminan terdaftar

INTERACTIVE COMMANDS (dalam mode interactive):
    search <query>  - Cari NFT serupa via Agent Protocol
    pawn            - Buat struk gadai NFT baru
    exit            - Keluar

ENVIRONMENT VARIABLES:
    IPFS_API_URL        - URL IPFS API (default: http://127.0.0.1:5001)
    IPFS_GATEWAY        - URL IPFS Gateway (default: http://127.0.0.1:9393)
    IROH_API_URL        - URL Iroh API (default: http://127.0.0.1:4400)
    IROH_GATEWAY        - URL Iroh Gateway (default: http://127.0.0.1:4401)
    BURN_LM_URL         - URL burn-lm ASIST (default: http://localhost:9393)
    OLLAMA_URL          - URL Ollama (default: http://localhost:11434)
    AGENT_PROTOCOL_URL  - URL Agent Protocol (default: http://localhost:3030)
    AGENT_PROTOCOL_TOKEN- Auth token untuk Agent Protocol

AGENT PROTOCOL INTEGRATION:
    NFT yang dibuat akan otomatis di-index ke Agent Protocol untuk:
    - AI-powered similarity search (POST /scheme/nft/search)
    - Vector embeddings di Qdrant
    - Integrasi dengan AI agents
"#);
}

/// dev mode with sample NFT
async fn run_dev_mode() -> Result<()> {
    println!("[MODE] dev - Contoh Gadai NFT Bored Ape\n");
    println!("-----------------------------------------------------------\n");

    // Sample NFT data
    let (struk, analysis) = nft_pawn_flow_with_ipfs(
        "Budi Santoso",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD87",
        "Bored Ape Yacht Club",
        "1234",
        "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D",
        50.0,   // 50 ETH nilai taksiran
        0.4,    // 40% LTV
        30,     // 30 hari tenor
        "Bagaimana cara menggunakan Bored Ape sebagai jaminan pinjaman?",
    ).await?;

    println!("\n-----------------------------------------------------------");
    println!("[RINGKASAN]");
    println!("-----------------------------------------------------------");
    println!("  Nomor Struk    : {}", struk.nomor_struk);
    println!("  NFT            : {} #{}", struk.detail_nft.nama_koleksi, struk.detail_nft.token_id);
    println!("  Nilai Taksiran : {} ETH", struk.detail_pinjaman.nilai_taksiran);
    println!("  Pinjaman       : {} ETH", struk.detail_pinjaman.jumlah_pinjaman);
    println!("  Total Pelunasan: {} ETH", struk.detail_pinjaman.total_pelunasan);
    println!("  Jatuh Tempo    : {}", struk.detail_pinjaman.tanggal_jatuh_tempo);
    println!("  IPFS CID       : {}", struk.metadata.ipfs_cid.unwrap_or("N/A".to_string()));
    println!("  Status         : {} | Siap Jaminan: {} | Siap Jual: {}",
             struk.status.status_dokumen,
             if struk.status.siap_jaminan { "[v]" } else { "[x]" },
             if struk.status.siap_marketplace { "[v]" } else { "[x]" }
    );
    println!("-----------------------------------------------------------\n");

    Ok(())
}

/// Interactive pawn flow - create new struk
async fn run_pawn_flow() -> Result<()> {
    use std::io::{self, Write};

    println!("[MODE] Buat Struk Gadai NFT Baru\n");
    println!("Masukkan data NFT dan pinjaman:\n");

    // Get input from user
    fn prompt(label: &str) -> String {
        print!("{}: ", label);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    fn prompt_f64(label: &str, default: f64) -> f64 {
        let input = prompt(&format!("{} [{}]", label, default));
        if input.is_empty() {
            default
        } else {
            input.parse().unwrap_or(default)
        }
    }

    fn prompt_u32(label: &str, default: u32) -> u32 {
        let input = prompt(&format!("{} [{}]", label, default));
        if input.is_empty() {
            default
        } else {
            input.parse().unwrap_or(default)
        }
    }

    let nama_pemilik = prompt("Nama Pemilik");
    let alamat_wallet = prompt("Alamat Wallet (0x...)");
    let nama_koleksi = prompt("Nama Koleksi NFT");
    let token_id = prompt("Token ID");
    let contract_address = prompt("Contract Address (0x...)");
    let nilai_taksiran = prompt_f64("Nilai Taksiran (ETH)", 10.0);
    let ltv_ratio = prompt_f64("LTV Ratio (0.0-1.0)", 0.4);
    let tenor_hari = prompt_u32("Tenor (hari)", 30);

    println!("\n[Processing] Membuat struk gadai...\n");

    let (struk, _analysis) = nft_pawn_flow_with_ipfs(
        &nama_pemilik,
        &alamat_wallet,
        &nama_koleksi,
        &token_id,
        &contract_address,
        nilai_taksiran,
        ltv_ratio,
        tenor_hari,
        &format!("Analisis gadai NFT {} dengan nilai {} ETH", nama_koleksi, nilai_taksiran),
    ).await?;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  [OK] STRUK GADAI NFT BERHASIL DIBUAT                            ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Nomor: {:<54} ║", struk.nomor_struk);
    if let Some(ref cid) = struk.metadata.ipfs_cid {
        println!("║  IPFS : {:<54} ║", cid);
    }
    println!("║                                                                  ║");
    println!("║  Dokumen siap digunakan untuk:                                   ║");
    println!("║  - Jaminan pinjaman di platform DeFi                             ║");
    println!("║  - Dijual di NFT marketplace                                     ║");
    println!("║  - Verifikasi kepemilikan                                        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// Interactive query mode
async fn interactive_mode() -> Result<()> {
    use std::io::{self, Write};

    println!("[MODE] Interaktif - Tanya tentang NFT Pawnbroking\n");
    println!("Commands: 'pawn' buat struk | 'search <query>' cari NFT | 'exit' keluar\n");

    loop {
        print!("NFT Pawn > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let query = input.trim();

        if query.eq_ignore_ascii_case("exit") || query.eq_ignore_ascii_case("quit") {
            println!("Terima kasih! Sampai jumpa.");
            break;
        }

        if query.eq_ignore_ascii_case("pawn") {
            run_pawn_flow().await?;
            continue;
        }

        // Search NFTs via Agent Protocol
        if query.to_lowercase().starts_with("search ") {
            let search_query = &query[7..].trim();
            if search_query.is_empty() {
                println!("Usage: search <query>");
                continue;
            }

            println!("\n[Searching NFTs via Agent Protocol...]\n");
            let agent_client = AgentProtocolClient::new();

            if !agent_client.is_available().await {
                println!("[!] Agent Protocol not available. Set AGENT_PROTOCOL_URL env var.\n");
                continue;
            }

            match agent_client.search_nft(search_query, 5).await {
                Ok(response) => {
                    if response.results.is_empty() {
                        println!("Tidak ada NFT yang cocok dengan: {}\n", search_query);
                    } else {
                        println!("Found {} NFT(s) matching '{}':\n", response.count, search_query);
                        for (i, result) in response.results.iter().enumerate() {
                            println!("{}. [Score: {:.2}] CID: {}", i + 1, result.score, result.ipfs_cid);
                            println!("   Type: {} | Content: {}", result.chunk_type,
                                     if result.content.len() > 80 {
                                         format!("{}...", &result.content[..80])
                                     } else {
                                         result.content.clone()
                                     });
                            println!();
                        }
                    }
                }
                Err(e) => println!("Search error: {}\n", e),
            }
            continue;
        }

        if query.is_empty() {
            continue;
        }

        match query_nft_pawn(query).await {
            Ok(response) => println!("\n{}\n", response),
            Err(e) => println!("\nError: {}\n", e),
        }
    }

    Ok(())
}

/// Pretty print NFT Pawn result
fn print_result(result: &NftPawnResult) {
    println!("=== NFT PAWN ANALYSIS RESULT ===\n");

    println!("[Use Case] {}", result.use_case);
    println!("[Source Model] {}\n", result.source_model);

    println!("[Response]");
    println!("{}\n", result.response);

    if let Some(ref risk) = result.risk_assessment {
        println!("[Risk Assessment]");
        println!("  - Volatility Risk: {}", risk.volatility_risk);
        println!("  - Liquidation Risk: {}", risk.liquidation_risk);
        println!("  - Smart Contract Risk: {}", risk.smart_contract_risk);
        println!("  - Counterparty Risk: {}", risk.counterparty_risk);
        println!("  - Overall Level: {}\n", risk.overall_risk_level);
    }

    if let Some(ref market) = result.market_insights {
        println!("[Market Insights]");
        println!("  - Typical LTV Ratio: {}", market.typical_ltv_ratio);
        println!("  - Interest Rates: {}", market.interest_rates);
        println!("  - Popular Platforms: {}", market.popular_platforms.join(", "));
        println!("  - Market Trend: {}\n", market.market_trend);
    }

    println!("[Recommendations]");
    for (i, rec) in result.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec);
    }
    println!();
}

// ============================================================================
// RFID MODE - COLLATERAL TRACKING
// ============================================================================

/// RFID Collateral Tracking Mode
async fn rfid_mode() -> Result<()> {
    use std::io::{self, Write};

    println!("\n{}", "=".repeat(70));
    println!("     RFID COLLATERAL TRACKING SYSTEM                                ");
    println!("     Sistem Pelacakan Jaminan dengan RFID                           ");
    println!("{}\n", "=".repeat(70));

    let mut manager = RfidTrackingManager::new()?;

    println!("Commands: register | scan | location | condition | delivery | confirm");
    println!("          placement | link | list | dev | help | exit\n");

    loop {
        print!("RFID > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        let cmd = parts.get(0).map(|s| *s).unwrap_or("");

        match cmd.to_lowercase().as_str() {
            "exit" | "quit" | "q" => {
                println!("Terima kasih! Sampai jumpa.");
                break;
            }

            "help" | "h" => {
                print_rfid_help();
            }

            "dev" => {
                run_rfid_dev(&mut manager)?;
            }

            "register" | "reg" => {
                rfid_register(&mut manager)?;
            }

            "scan" | "s" => {
                rfid_scan(&mut manager)?;
            }

            "location" | "loc" => {
                rfid_update_location(&mut manager)?;
            }

            "condition" | "cond" => {
                rfid_update_condition(&mut manager)?;
            }

            "delivery" | "del" => {
                rfid_create_delivery(&mut manager)?;
            }

            "confirm" | "conf" => {
                rfid_confirm_delivery(&mut manager)?;
            }

            "placement" | "place" => {
                rfid_update_placement(&mut manager)?;
            }

            "link" => {
                rfid_link_struk(&mut manager)?;
            }

            "list" | "ls" => {
                rfid_list_all(&manager);
            }

            "" => continue,

            _ => {
                println!("Command tidak dikenal: {}. Ketik 'help' untuk bantuan.", cmd);
            }
        }
    }

    Ok(())
}

fn print_rfid_help() {
    println!(r#"
RFID COLLATERAL TRACKING COMMANDS:
-----------------------------------
  register    - Daftarkan RFID tag baru untuk jaminan
  scan        - Scan RFID tag dan lihat info tracking
  location    - Update lokasi jaminan
  condition   - Update kondisi jaminan
  delivery    - Buat pengiriman baru
  confirm     - Konfirmasi pengiriman tiba
  placement   - Update penempatan/penyimpanan
  link        - Hubungkan RFID ke Struk Gadai
  list        - Lihat semua jaminan terdaftar
  dev         - Jalankan dev dengan contoh data
  help        - Tampilkan bantuan ini
  exit        - Keluar dari mode RFID
"#);
}

fn prompt(label: &str) -> String {
    use std::io::{self, Write};
    print!("{}: ", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn rfid_register(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[REGISTER] Daftarkan RFID Tag Baru\n");

    let rfid_tag = prompt("RFID Tag ID");
    let collateral_type = prompt("Jenis Jaminan (Emas/Elektronik/Kendaraan/Dokumen)");
    let description = prompt("Deskripsi");
    let location = prompt("Lokasi Awal");
    let warehouse = prompt("Gudang");

    match manager.register_rfid(&rfid_tag, &collateral_type, &description, &location, &warehouse) {
        Ok(tracking) => {
            println!("\n[OK] RFID Tag berhasil didaftarkan!");
            RfidTrackingManager::print_tracking_info(&tracking);
        }
        Err(e) => println!("\n[ERROR] Gagal mendaftarkan: {}", e),
    }

    Ok(())
}

fn rfid_scan(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[SCAN] Scan RFID Tag\n");

    let rfid_tag = prompt("RFID Tag ID");

    match manager.scan_rfid(&rfid_tag) {
        Some(tracking) => {
            println!("\n[OK] Tag ditemukan!");
            RfidTrackingManager::print_tracking_info(tracking);
        }
        None => println!("\n[NOT FOUND] RFID Tag tidak ditemukan: {}", rfid_tag),
    }

    Ok(())
}

fn rfid_update_location(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[LOCATION] Update Lokasi Jaminan\n");

    let rfid_tag = prompt("RFID Tag ID");
    let new_location = prompt("Lokasi Baru");
    let warehouse = prompt("Gudang");
    let zone = prompt("Zone");
    let shelf_input = prompt("Rak (kosongkan jika tidak ada)");
    let shelf = if shelf_input.is_empty() { None } else { Some(shelf_input.as_str()) };

    match manager.update_location(&rfid_tag, &new_location, &warehouse, &zone, shelf) {
        Ok(_) => println!("\n[OK] Lokasi berhasil diupdate!"),
        Err(e) => println!("\n[ERROR] Gagal update lokasi: {}", e),
    }

    Ok(())
}

fn rfid_update_condition(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[CONDITION] Update Kondisi Jaminan\n");

    let rfid_tag = prompt("RFID Tag ID");
    let status = prompt("Status (Baik/Rusak Ringan/Rusak Berat)");
    let grade = prompt("Grade (A/B/C/D)");
    let notes = prompt("Catatan");
    let inspected_by = prompt("Diperiksa Oleh");

    match manager.update_condition(&rfid_tag, &status, &grade, &notes, &inspected_by) {
        Ok(_) => println!("\n[OK] Kondisi berhasil diupdate!"),
        Err(e) => println!("\n[ERROR] Gagal update kondisi: {}", e),
    }

    Ok(())
}

fn rfid_create_delivery(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[DELIVERY] Buat Pengiriman Baru\n");

    let rfid_tag = prompt("RFID Tag ID");
    let from_location = prompt("Dari Lokasi");
    let to_location = prompt("Ke Lokasi");
    let courier = prompt("Kurir/Pengantar");

    match manager.create_delivery(&rfid_tag, &from_location, &to_location, &courier) {
        Ok(delivery) => {
            println!("\n[OK] Pengiriman berhasil dibuat!");
            println!("   Delivery ID : {}", delivery.delivery_id);
            println!("   From        : {}", delivery.from_location);
            println!("   To          : {}", delivery.to_location);
            println!("   Courier     : {}", delivery.courier);
            println!("   Status      : {:?}", delivery.status);
        }
        Err(e) => println!("\n[ERROR] Gagal buat pengiriman: {}", e),
    }

    Ok(())
}

fn rfid_confirm_delivery(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[CONFIRM] Konfirmasi Pengiriman Tiba\n");

    let rfid_tag = prompt("RFID Tag ID");
    let delivery_id = prompt("Delivery ID");

    match manager.confirm_delivery(&rfid_tag, &delivery_id) {
        Ok(_) => println!("\n[OK] Pengiriman berhasil dikonfirmasi!"),
        Err(e) => println!("\n[ERROR] Gagal konfirmasi: {}", e),
    }

    Ok(())
}

fn rfid_update_placement(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[PLACEMENT] Update Penempatan/Penyimpanan\n");

    let rfid_tag = prompt("RFID Tag ID");
    let storage_type = prompt("Tipe Penyimpanan (Brankas/Rak/Safe Deposit Box)");
    let security_level = prompt("Level Keamanan (Standard/High/Maximum)");
    let assigned_to = prompt("Ditugaskan Kepada");
    let special_instructions = prompt("Instruksi Khusus");

    match manager.update_placement(&rfid_tag, &storage_type, &security_level, &assigned_to, &special_instructions) {
        Ok(_) => println!("\n[OK] Penempatan berhasil diupdate!"),
        Err(e) => println!("\n[ERROR] Gagal update penempatan: {}", e),
    }

    Ok(())
}

fn rfid_link_struk(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[LINK] Hubungkan RFID ke Struk Gadai\n");

    let rfid_tag = prompt("RFID Tag ID");
    let struk_nomor = prompt("Nomor Struk Gadai (SGN-...)");

    match manager.link_to_struk(&rfid_tag, &struk_nomor) {
        Ok(_) => println!("\n[OK] RFID berhasil dihubungkan ke Struk {}!", struk_nomor),
        Err(e) => println!("\n[ERROR] Gagal menghubungkan: {}", e),
    }

    Ok(())
}

fn rfid_list_all(manager: &RfidTrackingManager) {
    let records = manager.list_all();

    if records.is_empty() {
        println!("\n[INFO] Belum ada jaminan terdaftar.\n");
        return;
    }

    println!("\n{}", "=".repeat(70));
    println!("                    DAFTAR JAMINAN TERDAFTAR                         ");
    println!("{}", "=".repeat(70));
    println!("\n{:<15} {:<20} {:<15} {:<10} {:?}",
             "RFID TAG", "JENIS", "LOKASI", "KONDISI", "STATUS");
    println!("{}", "-".repeat(70));

    for record in records {
        println!("{:<15} {:<20} {:<15} {:<10} {:?}",
                 truncate_str(&record.rfid_tag.tag_id, 15),
                 truncate_str(&record.collateral_type, 20),
                 truncate_str(&record.current_location.location_name, 15),
                 &record.condition.grade,
                 record.rfid_tag.status
        );
    }

    println!("\nTotal: {} jaminan terdaftar\n", records.len());
}

fn run_rfid_dev(manager: &mut RfidTrackingManager) -> Result<()> {
    println!("\n[DEV] Menjalankan Dev RFID Tracking...\n");

    // Register sample collateral
    println!("[1/5] Mendaftarkan jaminan contoh...");
    let tracking = manager.register_rfid(
        "RFID-001-EMAS",
        "Emas",
        "Emas Batangan 10 gram - Antam",
        "Gudang Utama",
        "Warehouse Jakarta",
    )?;
    println!("      OK - Tag {} terdaftar", tracking.rfid_tag.tag_id);

    // Update location
    println!("[2/5] Update lokasi ke brankas...");
    manager.update_location(
        "RFID-001-EMAS",
        "Brankas Utama",
        "Warehouse Jakarta",
        "B",
        Some("Safe-01"),
    )?;
    println!("      OK - Lokasi diupdate");

    // Update condition
    println!("[3/5] Inspeksi kondisi...");
    manager.update_condition(
        "RFID-001-EMAS",
        "Baik",
        "A",
        "Kondisi sempurna, segel utuh",
        "Inspector Ahmad",
    )?;
    println!("      OK - Kondisi diupdate");

    // Update placement
    println!("[4/5] Update penempatan ke high security...");
    manager.update_placement(
        "RFID-001-EMAS",
        "Safe Deposit Box",
        "Maximum",
        "Security Team",
        "Akses hanya dengan 2 kunci",
    )?;
    println!("      OK - Penempatan diupdate");

    // Link to struk
    println!("[5/5] Hubungkan ke Struk Gadai...");
    manager.link_to_struk(
        "RFID-001-EMAS",
        "SGN-20241211-DEV001",
    )?;
    println!("      OK - Terhubung ke struk\n");

    // Show final result
    if let Some(record) = manager.scan_rfid("RFID-001-EMAS") {
        println!("[DEV COMPLETE] Hasil tracking:");
        RfidTrackingManager::print_tracking_info(record);
    }

    Ok(())
}

