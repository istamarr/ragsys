// Serial NFC/RFID Reader for RAG System
// Supports standard serial RFID/NFC readers over RS232/USB-CDC
//
// Compatible readers:
//   - Generic 125kHz RFID USB readers (HID/COM, 9600 baud, ASCII hex UID)
//   - MFRC522 + USB-serial adapter (115200 baud)
//   - PN532 UART mode (115200 baud)
//   - Wiegand readers via RS232 adapter (Wiegand-serial bridge)
//   - Any reader outputting UID as ASCII hex over serial
//
// Usage:
//   let config = SerialNfcRfidConfig::default();  // auto-detects COM port
//   connect_serial_reader_to_rag(config, qdrant_url).await?;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
// use qdrant_client::client::{QdrantClient, QdrantClientConfig};
use qdrant_client::Qdrant;
use qdrant_client;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::Reply;
use crate::domain::models::llm::CmdBody;
use crate::shared::helperUtils::{current_time, srv_response_json};
use crate::WebResult;

// ============================================================================
// SERIAL PROTOCOL TYPES
// ============================================================================

/// Output protocol of the connected serial RFID/NFC reader
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SerialProtocol {
    /// Most USB readers: plain hex UID per line e.g. `"A1B2C3D4\r\n"`
    AsciiHexLine,
    /// Prefixed output e.g. `"Card: A1B2C3D4"`, `"UID: AA BB CC DD"`
    PrefixedLine,
    /// Wiegand via serial bridge e.g. `"W26:01234567\n"`
    WiegandSerial,
    /// PN532 UART text mode e.g. `"UID Value: 0xAA 0xBB 0xCC 0xDD"`
    PN532Uart,
    /// Raw N bytes per tag read (no framing)
    RawBytes(u8),
}

impl Default for SerialProtocol {
    fn default() -> Self {
        SerialProtocol::AsciiHexLine
    }
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Configuration for a serial NFC/RFID reader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialNfcRfidConfig {
    /// Serial port path: `"COM3"` (Windows) or `"/dev/ttyUSB0"` (Linux)
    pub port: String,
    /// Baud rate. Most cheap readers: 9600. PN532/MFRC522: 115200
    pub baud_rate: u32,
    /// Protocol used to parse UIDs from the serial stream
    pub protocol: SerialProtocol,
    /// Serial read timeout in milliseconds
    pub timeout_ms: u64,
    /// Minimum milliseconds between two reads of the same UID (debounce)
    pub debounce_ms: u64,
    /// Human-readable name for this reader (used in logs / RAG context)
    pub reader_name: String,
    /// Optional RAG collection name to search/store against
    pub rag_collection: Option<String>,
}

impl Default for SerialNfcRfidConfig {
    fn default() -> Self {
        Self {
            #[cfg(windows)]
            port: "COM3".to_string(),
            #[cfg(not(windows))]
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 9600,
            protocol: SerialProtocol::AsciiHexLine,
            timeout_ms: 5000,
            debounce_ms: 1500,
            reader_name: "SerialReader-01".to_string(),
            rag_collection: None,
        }
    }
}

// ============================================================================
// TAG EVENT
// ============================================================================

/// Emitted each time an NFC/RFID tag is successfully read from the serial port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialTagEvent {
    /// Unique event ID
    pub read_id: String,
    /// Hex-encoded tag UID (e.g. `"A1B2C3D4"`)
    pub uid: String,
    /// Raw bytes received from serial port for this event
    pub raw_bytes: Vec<u8>,
    /// Protocol that produced this event
    pub protocol: SerialProtocol,
    /// Serial port this event came from
    pub port: String,
    /// Reader name from config
    pub reader_name: String,
    /// UTC timestamp of when the tag was scanned
    pub timestamp: DateTime<Utc>,
    /// Derived tag standard based on UID length
    pub tag_standard: TagStandard,
}

/// Inferred standard from UID byte count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagStandard {
    /// 4-byte UID — Mifare Classic 1K/4K, ISO14443A
    ISO14443A_4B,
    /// 7-byte UID — Mifare Ultralight, NTAG21x, ISO14443A
    ISO14443A_7B,
    /// 4-byte UID — EM4100 / EM4200, 125kHz
    EM4100,
    /// 10-byte UID — ISO15693 vicinity tags
    ISO15693,
    /// Unknown / other length
    Unknown(usize),
}

impl TagStandard {
    fn from_uid_hex(uid: &str) -> Self {
        let byte_count = uid.len() / 2;
        match byte_count {
            4 => TagStandard::EM4100,
            5 => TagStandard::ISO14443A_4B,
            7 => TagStandard::ISO14443A_7B,
            10 => TagStandard::ISO15693,
            _ => TagStandard::Unknown(byte_count),
        }
    }
}

// ============================================================================
// READER — SYNC CORE (runs in spawn_blocking)
// ============================================================================

/// Serial NFC/RFID reader — synchronous core, designed for `spawn_blocking`
pub struct SerialNfcRfidReader {
    pub config: SerialNfcRfidConfig,
    last_uid: Option<(String, std::time::Instant)>,
}

impl SerialNfcRfidReader {
    pub fn new(config: SerialNfcRfidConfig) -> Self {
        Self { config, last_uid: None }
    }

    // -----------------------------------------------------------------------
    // Public helpers
    // -----------------------------------------------------------------------

    /// Return a list of serial port names available on this system
    pub fn list_ports() -> Vec<String> {
        match serialport::available_ports() {
            Ok(ports) => ports.into_iter().map(|p| p.port_name).collect(),
            Err(e) => {
                warn!("[SerialRFID] Cannot enumerate serial ports: {}", e);
                vec![]
            }
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn is_debounced(&self, uid: &str) -> bool {
        if let Some((ref last_uid, last_time)) = self.last_uid {
            if last_uid == uid {
                return (last_time.elapsed().as_millis() as u64) < self.config.debounce_ms;
            }
        }
        false
    }

    fn parse_line(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        match &self.config.protocol {
            SerialProtocol::AsciiHexLine => {
                let hex: String = trimmed.chars()
                    .filter(|c| c.is_ascii_hexdigit())
                    .collect();
                if hex.len() >= 8 && hex.len() <= 28 {
                    Some(hex.to_uppercase())
                } else {
                    None
                }
            }

            SerialProtocol::PrefixedLine => {
                let lower = trimmed.to_lowercase();
                for prefix in &["card:", "uid:", "tag:", "id:", "no:", "rfid:", "nfc:", "serial:"] {
                    if let Some(rest) = lower.strip_prefix(prefix) {
                        let hex: String = rest.trim().chars()
                            .filter(|c| c.is_ascii_hexdigit())
                            .collect();
                        if hex.len() >= 8 {
                            return Some(hex.to_uppercase());
                        }
                    }
                }
                None
            }

            SerialProtocol::WiegandSerial => {
                // Format: "W26:01234567" or "W34:0123456789"
                let upper = trimmed.to_uppercase();
                if let Some(rest) = upper.strip_prefix('W') {
                    if let Some(colon) = rest.find(':') {
                        let hex: String = rest[colon + 1..].chars()
                            .filter(|c| c.is_ascii_hexdigit())
                            .collect();
                        if !hex.is_empty() {
                            return Some(hex);
                        }
                    }
                }
                None
            }

            SerialProtocol::PN532Uart => {
                // "UID Length: 4 bytes, UID Value: 0xAA 0xBB 0xCC 0xDD"
                let lower = trimmed.to_lowercase();
                if lower.contains("uid value:") || lower.contains("uid:") {
                    let bytes: Vec<String> = trimmed.split_whitespace()
                        .filter_map(|w| {
                            let w = w.trim_matches(',')
                                .trim_matches(':')
                                .trim_start_matches("0x")
                                .trim_start_matches("0X");
                            if w.len() == 2 && w.chars().all(|c| c.is_ascii_hexdigit()) {
                                Some(w.to_uppercase())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if bytes.len() >= 4 {
                        return Some(bytes.join(""));
                    }
                }
                None
            }

            SerialProtocol::RawBytes(_) => None,
        }
    }

    fn make_event(&self, uid: String, raw: Vec<u8>) -> SerialTagEvent {
        let tag_standard = TagStandard::from_uid_hex(&uid);
        SerialTagEvent {
            read_id: Uuid::new_v4().to_string(),
            uid,
            raw_bytes: raw,
            protocol: self.config.protocol.clone(),
            port: self.config.port.clone(),
            reader_name: self.config.reader_name.clone(),
            timestamp: Utc::now(),
            tag_standard,
        }
    }

    // -----------------------------------------------------------------------
    // Main blocking loop
    // -----------------------------------------------------------------------

    /// Open the serial port and emit `SerialTagEvent` for every tag read.
    /// Blocks until the channel is closed or a fatal I/O error occurs.
    pub fn open_and_stream(
        &mut self,
        tx: std::sync::mpsc::SyncSender<SerialTagEvent>,
    ) -> Result<()> {
        info!(
            "[SerialRFID] Opening {} at {} baud (protocol={:?})",
            self.config.port, self.config.baud_rate, self.config.protocol
        );

        let port_builder = serialport::new(&self.config.port, self.config.baud_rate)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None);

        let port = port_builder
            .open()
            .with_context(|| format!("Failed to open serial port '{}'", self.config.port))?;

        info!("[SerialRFID] Port opened — listening for NFC/RFID tags...");

        if let SerialProtocol::RawBytes(n) = self.config.protocol {
            use std::io::Read;
            let mut port = port;
            let n = n as usize;
            let mut buf = vec![0u8; n];
            loop {
                match port.read_exact(&mut buf) {
                    Ok(()) => {
                        let uid = hex::encode(&buf).to_uppercase();
                        if !self.is_debounced(&uid) {
                            self.last_uid = Some((uid.clone(), std::time::Instant::now()));
                            info!("[SerialRFID] Tag read (raw {}B): UID={}", n, uid);
                            if tx.send(self.make_event(uid, buf.clone())).is_err() {
                                warn!("[SerialRFID] Receiver dropped — stopping reader");
                                break;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        error!("[SerialRFID] Read error: {}", e);
                        break;
                    }
                }
            }
        } else {
            let reader = BufReader::new(port);
            for line_res in reader.lines() {
                match line_res {
                    Ok(line) => {
                        debug!("[SerialRFID] raw <- {:?}", line);
                        if let Some(uid) = self.parse_line(&line) {
                            if !self.is_debounced(&uid) {
                                self.last_uid = Some((uid.clone(), std::time::Instant::now()));
                                info!("[SerialRFID] Tag read: UID={}", uid);
                                let raw = line.into_bytes();
                                if tx.send(self.make_event(uid, raw)).is_err() {
                                    warn!("[SerialRFID] Receiver dropped — stopping reader");
                                    break;
                                }
                            } else {
                                debug!("[SerialRFID] Debounced: {}", uid);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        error!("[SerialRFID] Line error: {}", e);
                        break;
                    }
                }
            }
        }

        info!("[SerialRFID] Stream ended for port {}", self.config.port);
        Ok(())
    }
}

// ============================================================================
// ASYNC RAG INTEGRATION
// ============================================================================

/// RAG response enriched with the originating tag event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidRagResult {
    pub tag_event: SerialTagEvent,
    pub rag_response: String,
    pub collection: String,
    pub timestamp: String,
}

/// Start a background task that reads from a serial NFC/RFID reader and
/// routes each tag UID through the RAG pipeline.
///
/// Returns a channel receiver — poll it to get `RfidRagResult` per tag scan.
pub async fn connect_serial_reader_to_rag(
    config: SerialNfcRfidConfig,
    qdrant_url: String,
) -> Result<mpsc::Receiver<RfidRagResult>> {
    use crate::rag_chaining::rag_pipeline::rag_pipeline_rfid;

    // Serial events: sync channel (blocking reader → async consumer)
    let (serial_tx, serial_rx) =
        std::sync::mpsc::sync_channel::<SerialTagEvent>(64);
    let serial_rx = Arc::new(Mutex::new(serial_rx));

    // RAG results: async channel
    let (rag_tx, rag_rx) = mpsc::channel::<RfidRagResult>(64);

    // ── Spawn blocking serial reader ────────────────────────────────────────
    let cfg_clone = config.clone();
    tokio::task::spawn_blocking(move || {
        let mut reader = SerialNfcRfidReader::new(cfg_clone);
        if let Err(e) = reader.open_and_stream(serial_tx) {
            error!("[SerialRFID] Reader exited with error: {}", e);
        }
    });

    // ── Spawn async RAG routing task ────────────────────────────────────────
    let collection = config
        .rag_collection
        .clone()
        .unwrap_or_else(|| "rfid_knowledge".to_string());

    tokio::spawn(async move {
        // let qdrant_cfg = QdrantClient::from(&qdrant_url);
        let qdrant_client = match Qdrant::from_url(&*qdrant_url).build() {
            Ok(c) => c,
            Err(e) => {
                error!("[SerialRFID] Qdrant init failed: {}", e);
                return;
            }
        };

        loop {
            let rx = Arc::clone(&serial_rx);
            let event = tokio::task::spawn_blocking(move || rx.lock().unwrap().recv())
                .await;

            match event {
                Ok(Ok(tag)) => {
                    info!(
                        "[SerialRFID→RAG] Routing UID={} reader={}",
                        tag.uid, tag.reader_name
                    );

                    let prompt = format!(
                        "RFID/NFC tag scanned — UID: {} | Reader: {} | Port: {} | Standard: {:?} | Time: {}",
                        tag.uid, tag.reader_name, tag.port, tag.tag_standard, tag.timestamp
                    );

                    let body = CmdBody {
                        model: String::new(),
                        prompt: prompt.clone(),
                        cmd: String::new(),
                        tags: "rfid serial".to_string(),
                        options: String::new(),
                        keepAlive: String::new(),
                    };

                    match rag_pipeline_rfid(&qdrant_client, &tag.uid, body).await {
                        Ok(response) => {
                            let result = RfidRagResult {
                                tag_event: tag,
                                rag_response: response,
                                collection: collection.clone(),
                                timestamp: current_time(),
                            };
                            if rag_tx.send(result).await.is_err() {
                                info!("[SerialRFID→RAG] Result receiver dropped — stopping");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("[SerialRFID→RAG] RAG pipeline error for UID={}: {}", tag.uid, e);
                        }
                    }
                }
                Ok(Err(_)) => {
                    info!("[SerialRFID→RAG] Serial reader disconnected");
                    break;
                }
                Err(e) => {
                    error!("[SerialRFID→RAG] spawn_blocking panic: {}", e);
                    break;
                }
            }
        }
    });

    Ok(rag_rx)
}

// ============================================================================
// WARP HTTP HANDLERS
// ============================================================================

/// GET /serial/rfid/ports
/// Returns all serial ports available on this machine
pub async fn serial_list_ports_handler(uid: String) -> WebResult<impl Reply> {
    info!("[SerialRFID] list_ports request from uid={}", uid);
    let ports = SerialNfcRfidReader::list_ports();
    let response = serde_json::json!({
        "status": "ok",
        "uid": uid,
        "ports": ports,
        "count": ports.len(),
        "timestamp": current_time()
    });
    srv_response_json(response, actix_web::http::StatusCode::OK)
}

/// POST /serial/rfid/scan-once
/// Body: `SerialNfcRfidConfig`
/// Opens the port, waits for exactly one tag read, then closes.
pub async fn serial_scan_once_handler(
    uid: String,
    config: SerialNfcRfidConfig,
) -> WebResult<impl Reply> {
    info!(
        "[SerialRFID] scan_once uid={} port={} baud={}",
        uid, config.port, config.baud_rate
    );

    let config_clone = config.clone();
    let port_name = config.port.clone();

    let spawn_result = tokio::task::spawn_blocking(move || -> std::result::Result<SerialTagEvent, String> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<SerialTagEvent>(1);
        let mut reader = SerialNfcRfidReader::new(config_clone);

        std::thread::spawn(move || {
            if let Err(e) = reader.open_and_stream(tx) {
                error!("[SerialRFID] scan_once reader error: {}", e);
            }
        });

        rx.recv_timeout(Duration::from_millis(15_000))
            .map_err(|_| format!("Timeout: no tag detected within 15 seconds on {}", port_name))
    })
    .await;

    match spawn_result {
        Ok(Ok(event)) => {
            let response = serde_json::json!({
                "status": "scanned",
                "uid": event.uid,
                "tag_standard": format!("{:?}", event.tag_standard),
                "port": event.port,
                "reader_name": event.reader_name,
                "protocol": format!("{:?}", event.protocol),
                "timestamp": event.timestamp,
                "read_id": event.read_id
            });
            srv_response_json(response, actix_web::http::StatusCode::OK)
        }
        Ok(Err(msg)) => {
            let response = serde_json::json!({
                "status": "error",
                "message": msg,
                "port": config.port,
                "timestamp": current_time()
            });
            srv_response_json(response, actix_web::http::StatusCode::BAD_REQUEST)
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "message": format!("Internal task error: {}", e),
                "port": config.port,
                "timestamp": current_time()
            });
            srv_response_json(response, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /serial/rfid/connect-rag
/// Body: `SerialConnectRagRequest`
/// Starts a background reader that pipes every tag scan into the RAG pipeline.
/// Returns immediately with a session ID.
pub async fn serial_connect_rag_handler(
    uid: String,
    req: SerialConnectRagRequest,
) -> WebResult<impl Reply> {
    info!(
        "[SerialRFID] connect_rag uid={} port={} qdrant={}",
        uid, req.config.port, req.qdrant_url
    );

    let session_id = Uuid::new_v4().to_string();
    let config = req.config.clone();
    let qdrant_url = req.qdrant_url.clone();
    let session_id_clone = session_id.clone();

    tokio::spawn(async move {
        match connect_serial_reader_to_rag(config, qdrant_url).await {
            Ok(mut rx) => {
                info!("[SerialRFID] Session {} started", session_id_clone);
                while let Some(result) = rx.recv().await {
                    info!(
                        "[SerialRFID] Session {} — UID={} rag_len={}",
                        session_id_clone,
                        result.tag_event.uid,
                        result.rag_response.len()
                    );
                }
                info!("[SerialRFID] Session {} ended", session_id_clone);
            }
            Err(e) => {
                error!("[SerialRFID] Session {} failed to start: {}", session_id_clone, e);
            }
        }
    });

    let response = serde_json::json!({
        "status": "started",
        "session_id": session_id,
        "port": req.config.port,
        "baud_rate": req.config.baud_rate,
        "protocol": format!("{:?}", req.config.protocol),
        "reader_name": req.config.reader_name,
        "qdrant_url": req.qdrant_url,
        "timestamp": current_time()
    });
    srv_response_json(response, actix_web::http::StatusCode::OK)
}

/// Request body for `serial_connect_rag_handler`
#[derive(Debug, Clone, Deserialize)]
pub struct SerialConnectRagRequest {
    pub config: SerialNfcRfidConfig,
    pub qdrant_url: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reader(proto: SerialProtocol) -> SerialNfcRfidReader {
        SerialNfcRfidReader::new(SerialNfcRfidConfig {
            protocol: proto,
            ..Default::default()
        })
    }

    #[test]
    fn parse_ascii_hex_line() {
        let r = make_reader(SerialProtocol::AsciiHexLine);
        assert_eq!(r.parse_line("A1B2C3D4\r\n"), Some("A1B2C3D4".to_string()));
        assert_eq!(r.parse_line("  a1b2c3d4  "), Some("A1B2C3D4".to_string()));
        assert_eq!(r.parse_line("AABB"), None); // too short (4 hex = 2 bytes)
    }

    #[test]
    fn parse_prefixed_line() {
        let r = make_reader(SerialProtocol::PrefixedLine);
        assert_eq!(r.parse_line("UID: A1B2C3D4"), Some("A1B2C3D4".to_string()));
        assert_eq!(r.parse_line("Card: DEADBEEF"), Some("DEADBEEF".to_string()));
        assert_eq!(r.parse_line("RFID: 01234567"), Some("01234567".to_string()));
        assert_eq!(r.parse_line("unknown: 1234"), None); // unknown prefix
    }

    #[test]
    fn parse_wiegand_serial() {
        let r = make_reader(SerialProtocol::WiegandSerial);
        assert_eq!(r.parse_line("W26:01234567"), Some("01234567".to_string()));
        assert_eq!(r.parse_line("W34:AABBCCDD"), Some("AABBCCDD".to_string()));
        assert_eq!(r.parse_line("randomline"), None);
    }

    #[test]
    fn parse_pn532_uart() {
        let r = make_reader(SerialProtocol::PN532Uart);
        let line = "UID Value: 0xAA 0xBB 0xCC 0xDD";
        assert_eq!(r.parse_line(line), Some("AABBCCDD".to_string()));
    }

    #[test]
    fn tag_standard_from_uid() {
        assert!(matches!(TagStandard::from_uid_hex("A1B2C3D4"), TagStandard::EM4100));
        assert!(matches!(TagStandard::from_uid_hex("A1B2C3D4E5F6AA"), TagStandard::ISO14443A_7B));
    }

    #[test]
    fn debounce_same_uid() {
        let mut r = SerialNfcRfidReader::new(SerialNfcRfidConfig {
            debounce_ms: 2000,
            ..Default::default()
        });
        assert!(!r.is_debounced("AABBCCDD"));
        r.last_uid = Some(("AABBCCDD".to_string(), std::time::Instant::now()));
        assert!(r.is_debounced("AABBCCDD"));
        assert!(!r.is_debounced("11223344")); // different UID passes immediately
    }

    #[test]
    fn list_ports_does_not_panic() {
        let _ = SerialNfcRfidReader::list_ports();
    }
}
