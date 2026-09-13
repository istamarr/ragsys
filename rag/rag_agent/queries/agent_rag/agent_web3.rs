//! DB2 SELECT via odbc_api — Windows ODBC driver, no Linux WSL lib required
//! SELECT key, value FROM tbl_param
use std::collections::HashMap;
use anyhow::Context;
use odbc_api::Cursor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;

// ───────────────────────────── CONNECT ─────────────────────────────
// ── DB2 connection defaults (overridable via env) ─────────────────────────────
fn db2_env() -> (String, String, String, String, String, String, String) {
    (
        std::env::var("DB2_ODBC_DRIVER").unwrap_or_else(|_| "IBM DB2 ODBC DRIVER".to_string()),
        std::env::var("DB2_HOST")       .unwrap_or_else(|_| "10.253.13.17".to_string()),
        std::env::var("DB2_PORT")       .unwrap_or_else(|_| "60040".to_string()),
        std::env::var("DB2_DBNAME")     .unwrap_or_else(|_| "K1250202".to_string()),
        std::env::var("DB2_USER")       .unwrap_or_else(|_| "passion".to_string()),
        std::env::var("DB2_PASSWORD")   .unwrap_or_else(|_| "P4ss!ondev".to_string()),
        std::env::var("DB2_SCHEMA")     .unwrap_or_else(|_| "PEGADAIAN".to_string()),
    )
}

// ───────────────────────────── QUERY ─────────────────────────────
/// Sync DB2 SELECT via odbc_api.
/// Windows ODBC driver (IBM DB2 ODBC DRIVER) — no ibm_db / libdb2.so / WSL needed.
fn db2_select(sql: &str) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
    use odbc_api::{Environment, ConnectionOptions, ResultSetMetadata};
    use odbc_api::buffers::TextRowSet;

    let (driver, host, port, dbname, user, password, schema) = db2_env();
    let conn_str = format!(
        "DRIVER={{{driver}}};DATABASE={dbname};HOSTNAME={host};PORT={port};\
         PROTOCOL=TCPIP;UID={user};PWD={password};CurrentSchema={schema};"
    );
    println!("[db2_select] DRIVER={driver} {host}:{port}/{dbname} schema={schema}");

    let env = Environment::new()
        .context("ODBC Environment::new() failed — check DB2 ODBC driver is installed on Windows")?;

    let conn = env
        .connect_with_connection_string(&conn_str, ConnectionOptions::default())
        .context("DB2 ODBC connect failed — verify DB2_ODBC_DRIVER name via odbcad32.exe (Drivers tab)")?;

    let cursor = conn
        .execute(sql, ())
        .context("DB2 query execute failed")?;

    let mut results: Vec<HashMap<String, serde_json::Value>> = Vec::new();

    if let Some(mut cursor) = cursor {
        let col_names: Vec<String> = cursor
            .column_names()
            .context("Failed to get DB2 column names")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect DB2 column names")?;

        let mut buffers = TextRowSet::for_cursor(512, &mut cursor, Some(4096))
            .context("TextRowSet::for_cursor failed")?;
        let mut row_cursor = cursor
            .bind_buffer(&mut buffers)
            .context("cursor.bind_buffer failed")?;

        while let Some(batch) = row_cursor.fetch().context("row_cursor.fetch failed")? {
            for row_idx in 0..batch.num_rows() as usize {
                let mut record: HashMap<String, serde_json::Value> = HashMap::new();
                for (col_idx, col_name) in col_names.iter().enumerate() {
                    let val = batch
                        .at(col_idx, row_idx)
                        .map(|b| std::str::from_utf8(b).unwrap_or("").trim().to_owned())
                        .unwrap_or_default();
                    record.insert(col_name.clone(), serde_json::Value::String(val));
                }
                results.push(record);
            }
        }
    } else {
        println!("[db2_select] query returned no rows");
    }

    println!("[db2_select] {} rows loaded", results.len());
    Ok(results)
}

fn load_dotenv() {
    if dotenv::dotenv().is_err() {
        'dotenv: {
            if let Ok(exe_path) = std::env::current_exe() {
                for dir in exe_path.ancestors().skip(1) {
                    for candidate in [dir.join(".env"), dir.join("ap").join(".env")] {
                        if candidate.exists() {
                            if let Ok(iter) = dotenv::from_path_iter(&candidate) {
                                let mut n = 0usize;
                                for item in iter {
                                    if let Ok((k, v)) = item {
                                        unsafe { std::env::set_var(k, v); }
                                        n += 1;
                                    }
                                }
                                println!("[dotenv] loaded {} vars from {}", n, candidate.display());
                            }
                            break 'dotenv;
                        }
                    }
                }
            }
        }
    }
}

/// SELECT key, value FROM {schema}.tbl_param
/// If name_data is empty → returns all rows.
/// If name_data is set   → WHERE key = '<name_data>'
///
/// Uses odbc_api (Windows ODBC) — no ibm_db / Linux WSL lib needed.
pub async fn get_pgd_param_latest(
    name_data: &str,
) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
    load_dotenv();

    let schema   = std::env::var("DB2_SCHEMA").unwrap_or_else(|_| "PEGADAIAN".to_string());
    let sql = if name_data.is_empty() {
        format!("SELECT key, value FROM {schema}.tbl_param ORDER BY key ASC")
    } else {
        format!(
            "SELECT key, value FROM {schema}.tbl_param WHERE key = '{name_data}' ORDER BY key ASC"
        )
    };
    println!("[get_pgd_param_latest] SQL: {sql}");

    let sql_owned = sql.clone();
    let rows = tokio::task::spawn_blocking(move || db2_select(&sql_owned))
        .await
        .context("DB2 blocking task panicked")??;

    Ok(rows)
}

/// SELECT * FROM {schema}.tbl_param — all columns, all rows
pub async fn get_pgd_param_all() -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
    load_dotenv();

    let schema = std::env::var("DB2_SCHEMA").unwrap_or_else(|_| "PEGADAIAN".to_string());
    let sql    = format!("SELECT * FROM {schema}.tbl_param ORDER BY key ASC");
    println!("[get_pgd_param_all] SQL: {sql}");

    let sql_owned = sql.clone();
    let rows = tokio::task::spawn_blocking(move || db2_select(&sql_owned))
        .await
        .context("DB2 blocking task panicked")??;

    Ok(rows)
}


pub async fn get_pgd_fee_latest(
    account_no: &str,
    rubrik: &str,
) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
    load_dotenv();

    if account_no.is_empty() || rubrik.is_empty() {
        println!("Err # get_pgd_fee_latest # account_no or rubrik must be filled");
        return Ok(Vec::new());
    }

    let schema = std::env::var("DB2_SCHEMA").unwrap_or_else(|_| "PEGADAIAN".to_string());
    let sql = if rubrik == "SB" {
        format!(
            "SELECT * FROM {schema}.TBL_KREDIT tk JOIN {schema}.TBL_KREDIT_MASALAH tkm
            ON tk.ACCOUNT_NO = tkm.ACCOUNT_NO
            JOIN {schema}.TBL_JASA_SIMPAN_BJ tjsb ON tjsb.NO_APPLIKASI = tkm.ACCOUNT_NO
            JOIN {schema}.TBL_DET_JAM_SB tdjs ON tdjs.ACCOUNT_NO = tk.ACCOUNT_NO
            JOIN {schema}.TBL_BIAYA_KREDIT tbk ON tbk.ACCOUNT_NO = tk.ACCOUNT_NO
            WHERE tk.ACCOUNT_NO='{account_no}';"
        )
    } else {
        format!(
            "SELECT * FROM {schema}.TBL_KREDIT tk JOIN {schema}.TBL_KREDIT_MASALAH tkm
            ON tk.ACCOUNT_NO = tkm.ACCOUNT_NO
            LEFT JOIN {schema}.TBL_DET_JAM_KT tkr ON tkr.ACCOUNT_NO = tk.ACCOUNT_NO
            JOIN {schema}.TBL_DET_JAM_SB tdjs ON tdjs.ACCOUNT_NO = tk.ACCOUNT_NO
            JOIN {schema}.TBL_BIAYA_KREDIT tbk ON tbk.ACCOUNT_NO = tk.ACCOUNT_NO
            WHERE tk.ACCOUNT_NO='{account_no}';"
        )
    };

    println!("[get_pgd_fee_latest] SQL: {sql}");

    let rows = tokio::task::spawn_blocking(move || db2_select(&sql))
        .await
        .context("DB2 blocking task panicked")??;

    Ok(rows)
}


// ───────────────────────────── OBJECT NFTs ─────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicNFTsRAGConfig {
    pub minting: Vec<String>,
    pub floor_price: String,
    pub hype: bool,
    pub fee_royalty: f64,
    pub fee_marketplace: bool,
}

impl Default for DynamicNFTsRAGConfig {
    fn default() -> Self {
        // Dynamically detect available query files

        Self {
            minting:         Vec::new(),
            floor_price:     String::new(),
            hype:            true,
            fee_royalty:     0.8,
            fee_marketplace: false,
        }
    }
}


// ───────────────────────────── SETUP OBJECT NFTs ─────────────────────────────
// pub fn initialize_nft()->Result<String,dyn std::error::Error>{
    // DynamicNFTsETLConfig::default();

    // DynamicNFTsETLConfig::new();

    // Ok("initialize collaterl nft value done".to_string())
// }
