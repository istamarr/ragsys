//! RFID AI Assistant - Prediction & Prevention Analysis
//!
//! Provides AI-powered analysis for RFID tracking data:
//! - Prediction: Demand forecasting and replenishment recommendations
//! - Prevention: Real-time loss prevention alerts during tracking transitions

use anyhow::{Context, Result};
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use chrono::{DateTime, Local};

// ============================================================================
// Data Structures
// ============================================================================

/// RFID analysis request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RfidAnalysisType {
    /// Demand Prediction & Replenishment (before tracking starts)
    Prediction,
    /// Real-Time Loss Prevention (at each tracking point update)
    Prevention,
}

/// RFID data for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidAnalysisData {
    /// RFID tag identifier
    pub tag_id: String,
    /// Collateral type (Emas, Elektronik, Kendaraan, Dokumen)
    pub collateral_type: String,
    /// Current status
    pub status: String,
    /// Current location
    pub location: String,
    /// Scan count (for prediction analysis)
    pub scan_count: Option<u32>,
    /// Last scan timestamp
    pub last_scan: Option<String>,
    /// Transition information (for prevention analysis)
    pub transition: Option<TransitionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionInfo {
    /// From location
    pub from: String,
    /// To location
    pub to: String,
    /// Departure timestamp
    pub departed_at: String,
    /// Arrival timestamp (if arrived)
    pub arrived_at: Option<String>,
    /// Courier/handler
    pub courier: String,
    /// Expected transit time in minutes
    pub expected_transit_minutes: Option<u32>,
    /// Actual elapsed time in minutes
    pub actual_elapsed_minutes: Option<u32>,
    /// Scanned at departure
    pub scanned_at_departure: bool,
    /// Scanned at arrival
    pub scanned_at_arrival: bool,
}

/// Changelog metadata response from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidChangelogMetadata {
    pub r#type: String,
    pub generated_at: String,
    pub analysis_type: String,
    pub tag_id: Option<String>,
    pub alert_level: Option<String>,
    pub alerts: Vec<Alert>,
    pub predictions: Vec<Prediction>,
    pub changelog: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub code: String,
    pub message: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub collateral_type: String,
    pub demand_trend: String,
    pub recommended_stock_level: u32,
    pub restock_by: Option<String>,
    pub overstock_risk: Option<bool>,
    pub confidence: f64,
}

// ============================================================================
// RFID AI Assistant
// ============================================================================

/// RFID AI Assistant for Prediction and Prevention analysis
pub struct RfidAiAssistant {
    /// Path to candle_eng binary (default: target/release/candle_eng.exe)
    candle_eng_path: PathBuf,
    /// Enable AI analysis (safe flag to disable if needed)
    ai_enabled: bool,
}

impl RfidAiAssistant {
    /// Create new RFID AI Assistant
    pub fn new() -> Self {
        let candle_eng_path = Self::get_candle_eng_path();
        Self {
            candle_eng_path,
            ai_enabled: true,
        }
    }

    /// Create with custom candle_eng path
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            candle_eng_path: path,
            ai_enabled: true,
        }
    }

    /// Disable AI analysis (safe mode - returns empty changelog)
    pub fn with_ai_enabled(mut self, enabled: bool) -> Self {
        self.ai_enabled = enabled;
        self
    }

    /// Get candle_eng binary path
    fn get_candle_eng_path() -> PathBuf {
        // Try target/release first, then fallback to target/debug
        let release_path = PathBuf::from("target/release/candle_eng.exe");
        let debug_path = PathBuf::from("target/debug/candle_eng.exe");

        if release_path.exists() {
            release_path
        } else if debug_path.exists() {
            debug_path
        } else {
            // Default to release path even if not exists (will error during run)
            release_path
        }
    }

    /// Run Prediction analysis (demand forecasting)
    pub async fn run_prediction(&self, data: Vec<RfidAnalysisData>) -> Result<RfidChangelogMetadata> {
        if !self.ai_enabled {
            warn!("AI analysis disabled - returning empty changelog");
            return Ok(self.empty_changelog("Prediction"));
        }

        info!("Running RFID Prediction analysis for {} records", data.len());

        let prompt = self.build_prediction_prompt(data)?;
        let changelog = self.call_candle_eng(&prompt).await?;

        Ok(changelog)
    }

    /// Run Prevention analysis (real-time loss prevention)
    pub async fn run_prevention(&self, data: RfidAnalysisData) -> Result<RfidChangelogMetadata> {
        if !self.ai_enabled {
            warn!("AI analysis disabled - returning empty changelog");
            return Ok(self.empty_changelog("Prevention"));
        }

        info!("Running RFID Prevention analysis for tag: {}", data.tag_id);

        let prompt = self.build_prevention_prompt(data)?;
        let changelog = self.call_candle_eng(&prompt).await?;

        Ok(changelog)
    }

    /// Build prediction prompt from RFID data
    fn build_prediction_prompt(&self, data: Vec<RfidAnalysisData>) -> Result<String> {
        let data_json = serde_json::to_string(&data)
            .context("Failed to serialize RFID data")?;

        Ok(format!(
            "[No write code, direct data result] 'data': {} create changelog metadata [rfid][Prediction]",
            data_json
        ))
    }

    /// Build prevention prompt from single RFID transition
    fn build_prevention_prompt(&self, data: RfidAnalysisData) -> Result<String> {
        let data_json = serde_json::to_string(&data)
            .context("Failed to serialize RFID data")?;

        Ok(format!(
            "[No write code, direct data result] 'data': {} create changelog metadata [rfid][Prevention]",
            data_json
        ))
    }

    /// Call candle_eng binary with prompt
    async fn call_candle_eng(&self, prompt: &str) -> Result<RfidChangelogMetadata> {
        info!("Calling candle_eng at: {:?}", self.candle_eng_path);

        if !self.candle_eng_path.exists() {
            error!("candle_eng binary not found at: {:?}", self.candle_eng_path);
            return Err(anyhow::anyhow!("candle_eng binary not found. Run: cargo build --release --bin candle_eng"));
        }

        let output = Command::new(&self.candle_eng_path)
            .arg("ask")
            .arg("-q")
            .arg(prompt)
            .output()
            .context("Failed to execute candle_eng")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("candle_eng failed: {}", stderr);
            return Err(anyhow::anyhow!("candle_eng execution failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let changelog: RfidChangelogMetadata = serde_json::from_str(&stdout)
            .context("Failed to parse candle_eng output as JSON")?;

        info!("Changelog generated: type={}, alerts={}, predictions={}",
              changelog.r#type, changelog.alerts.len(), changelog.predictions.len());

        Ok(changelog)
    }

    /// Generate empty changelog (when AI is disabled)
    fn empty_changelog(&self, analysis_type: &str) -> RfidChangelogMetadata {
        RfidChangelogMetadata {
            r#type: format!("rfid_{}", analysis_type.to_lowercase()),
            generated_at: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            analysis_type: analysis_type.to_string(),
            tag_id: None,
            alert_level: Some("INFO".to_string()),
            alerts: vec![],
            predictions: vec![],
            changelog: vec![format!("[{}] AI analysis disabled", analysis_type)],
        }
    }
}

impl Default for RfidAiAssistant {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Convenience Functions for Integration
// ============================================================================

/// Run prediction analysis on RFID tracking records
pub async fn analyze_rfid_prediction(tracking_records: Vec<RfidAnalysisData>) -> Result<RfidChangelogMetadata> {
    let assistant = RfidAiAssistant::new();
    assistant.run_prediction(tracking_records).await
}

/// Run prevention analysis on a single RFID transition
pub async fn analyze_rfid_prevention(tracking_record: RfidAnalysisData) -> Result<RfidChangelogMetadata> {
    let assistant = RfidAiAssistant::new();
    assistant.run_prevention(tracking_record).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prediction_prompt_generation() {
        let assistant = RfidAiAssistant::new().with_ai_enabled(false);
        let data = vec![
            RfidAnalysisData {
                tag_id: "RFID-001".to_string(),
                collateral_type: "Emas".to_string(),
                status: "Stored".to_string(),
                location: "Brankas Utama".to_string(),
                scan_count: Some(14),
                last_scan: Some("2026-05-10".to_string()),
                transition: None,
            }
        ];

        let prompt = assistant.build_prediction_prompt(data).unwrap();
        assert!(prompt.contains("[rfid][Prediction]"));
    }

    #[tokio::test]
    async fn test_prevention_prompt_generation() {
        let assistant = RfidAiAssistant::new().with_ai_enabled(false);
        let data = RfidAnalysisData {
            tag_id: "RFID-001".to_string(),
            collateral_type: "Emas".to_string(),
            status: "InTransit".to_string(),
            location: "Gudang Utama".to_string(),
            scan_count: None,
            last_scan: None,
            transition: Some(TransitionInfo {
                from: "Gudang Utama".to_string(),
                to: "Brankas Utama".to_string(),
                departed_at: "2026-05-15T08:00".to_string(),
                arrived_at: None,
                courier: "Staff Ahmad".to_string(),
                expected_transit_minutes: Some(30),
                actual_elapsed_minutes: Some(95),
                scanned_at_departure: true,
                scanned_at_arrival: false,
            }),
        };

        let prompt = assistant.build_prevention_prompt(data).unwrap();
        assert!(prompt.contains("[rfid][Prevention]"));
    }

    #[tokio::test]
    async fn test_ai_disabled_returns_empty_changelog() {
        let assistant = RfidAiAssistant::new().with_ai_enabled(false);
        let result = assistant.run_prediction(vec![]).await.unwrap();
        
        assert_eq!(result.analysis_type, "Prediction");
        assert!(result.alerts.is_empty());
        assert!(result.changelog.iter().any(|s| s.contains("AI analysis disabled")));
    }
}
