#[derive(Debug, Clone)]
pub struct NftLetterParams {
    pub token_id: String,
    pub token_name: String,
    pub owner_name: String,
    pub owner_wallet: String,
    pub collection_name: String,
    pub contract_address: String,
    pub blockchain: String,
    pub ipfs_hash: String,
    pub edition: String,
    pub royalty_percent: f32,
    pub issue_date: String,
    pub issuer_name: String,
    pub issuer_contact: String,
    pub metadata_uri: String,
}

impl Default for NftLetterParams {
    fn default() -> Self {
        use std::env;
        Self {
            token_id:         env::var("NFT_TOKEN_ID").unwrap_or_else(|_| "0001".to_string()),
            token_name:       env::var("NFT_TOKEN_NAME").unwrap_or_else(|_| "Untitled NFT".to_string()),
            owner_name:       env::var("NFT_OWNER_NAME").unwrap_or_else(|_| "Anonymous".to_string()),
            owner_wallet:     env::var("NFT_OWNER_WALLET").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),
            collection_name:  env::var("NFT_COLLECTION").unwrap_or_else(|_| "Genesis Collection".to_string()),
            contract_address: env::var("NFT_CONTRACT").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),
            blockchain:       env::var("NFT_BLOCKCHAIN").unwrap_or_else(|_| "Ethereum".to_string()),
            ipfs_hash:        env::var("NFT_IPFS_HASH").unwrap_or_else(|_| "QmXxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string()),
            edition:          env::var("NFT_EDITION").unwrap_or_else(|_| "1 of 1".to_string()),
            royalty_percent:  env::var("NFT_ROYALTY").ok().and_then(|v| v.parse().ok()).unwrap_or(5.0),
            issue_date:       env::var("NFT_ISSUE_DATE").unwrap_or_else(|_| chrono::Local::now().format("%B %d, %Y").to_string()),
            issuer_name:      env::var("NFT_ISSUER_NAME").unwrap_or_else(|_| "DIFSR Platform".to_string()),
            issuer_contact:   env::var("NFT_ISSUER_CONTACT").unwrap_or_else(|_| "contact@difsr.io".to_string()),
            metadata_uri:     env::var("NFT_METADATA_URI").unwrap_or_else(|_| "ipfs://".to_string()),
        }
    }
}

pub fn print(params: &NftLetterParams) -> String {
    format!(
        r#"
================================================================================
                        NFT CERTIFICATE OF OWNERSHIP
================================================================================

  This certificate is issued to confirm the authenticated ownership of a
  Non-Fungible Token (NFT) asset registered on the blockchain.

--------------------------------------------------------------------------------
  ASSET INFORMATION
--------------------------------------------------------------------------------
  Token Name       : {token_name}
  Token ID         : #{token_id}
  Collection       : {collection_name}
  Edition          : {edition}

--------------------------------------------------------------------------------
  BLOCKCHAIN RECORD
--------------------------------------------------------------------------------
  Blockchain       : {blockchain}
  Contract Address : {contract_address}
  IPFS Hash        : {ipfs_hash}
  Metadata URI     : {metadata_uri}

--------------------------------------------------------------------------------
  OWNERSHIP
--------------------------------------------------------------------------------
  Owner Name       : {owner_name}
  Owner Wallet     : {owner_wallet}
  Royalty          : {royalty_percent:.1}%

--------------------------------------------------------------------------------
  ISSUANCE
--------------------------------------------------------------------------------
  Issue Date       : {issue_date}
  Issued By        : {issuer_name}
  Contact          : {issuer_contact}

================================================================================
  This document serves as an official record of NFT ownership. Verify on-chain
  at any time using the contract address and token ID listed above.
================================================================================
"#,
        token_name       = params.token_name,
        token_id         = params.token_id,
        collection_name  = params.collection_name,
        edition          = params.edition,
        blockchain       = params.blockchain,
        contract_address = params.contract_address,
        ipfs_hash        = params.ipfs_hash,
        metadata_uri     = params.metadata_uri,
        owner_name       = params.owner_name,
        owner_wallet     = params.owner_wallet,
        royalty_percent  = params.royalty_percent,
        issue_date       = params.issue_date,
        issuer_name      = params.issuer_name,
        issuer_contact   = params.issuer_contact,
    )
}

pub fn print_to_stdout(params: &NftLetterParams) {
    println!("{}", print(params));
}

pub fn print_to_file(params: &NftLetterParams, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, print(params))?;
    println!("NFT letter saved: {}", path);
    Ok(())
}

pub fn print_minimal(params: &NftLetterParams) -> String {
    format!(
        "NFT #{} | {} | Owner: {} | Wallet: {} | Chain: {}",
        params.token_id,
        params.token_name,
        params.owner_name,
        params.owner_wallet,
        params.blockchain,
    )
}
