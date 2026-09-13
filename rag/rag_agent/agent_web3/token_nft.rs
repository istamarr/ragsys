//erc20
//format: token erc1155/erc721 ----> metadata cid ----> metadata log hist minting ----> metadata log hist ledger
//node1 dan 2 open , node 3 unttuk access superuser
//ipfs tambahkan role fetch and mapping
use ethers::{
    abi::{Abi, Function, Param, ParamType, StateMutability},
    contract::Contract,
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::sync::Arc;
use anyhow::Result;

/// Build a minimal ABI from a list of function signatures.
fn build_abi(functions: &[&str]) -> Abi {
    let mut abi = Abi::default();
    for sig in functions {
        let func: Function = sig.parse().unwrap();
        abi.functions
            .entry(func.name.clone())
            .or_insert_with(Vec::new)
            .push(func);
    }
    abi
}

/// ERC-20: print total supply, balance, name, symbol, decimals.
async fn print_erc20(
    provider: &Provider<Http>,
    token_addr: Address,
    owner: Address,
) -> Result<()> {
    let abi = build_abi(&[
        "totalSupply()(uint256)",
        "balanceOf(address)(uint256)",
        "name()(string)",
        "symbol()(string)",
        "decimals()(uint8)",
    ]);
    let contract = Contract::new(token_addr, abi, Arc::new(provider.clone()));

    let total: U256 = contract.method("totalSupply", ())?.call().await?;
    let balance: U256 = contract.method("balanceOf", owner)?.call().await?;
    let name: String = contract.method("name", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    let decimals: u8 = contract.method("decimals", ())?.call().await?;

    let total_formatted = ethers::utils::format_units(total, decimals)?;
    let balance_formatted = ethers::utils::format_units(balance, decimals)?;

    println!("=== ERC-20 ({}) ===", symbol);
    println!("Name        : {}", name);
    println!("Symbol      : {}", symbol);
    println!("Decimals    : {}", decimals);
    println!("Total Supply: {} {}", total_formatted, symbol);
    println!("Balance of {}: {} {}", owner, balance_formatted, symbol);
    println!();
    Ok(())
}

/// ERC-721: print balance, name, symbol, totalSupply (if enumerable), and sample tokenURI.
async fn print_erc721(
    provider: &Provider<Http>,
    token_addr: Address,
    owner: Address,
) -> Result<()> {
    let abi = build_abi(&[
        "balanceOf(address)(uint256)",
        "name()(string)",
        "symbol()(string)",
        "tokenURI(uint256)(string)",
        "totalSupply()(uint256)", // optional; we'll handle error
    ]);
    let contract = Contract::new(token_addr, abi, Arc::new(provider.clone()));

    let balance: U256 = contract.method("balanceOf", owner)?.call().await?;
    let name: String = contract.method("name", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;

    println!("=== ERC-721 ({}) ===", symbol);
    println!("Name        : {}", name);
    println!("Symbol      : {}", symbol);
    println!("NFTs owned by {}: {}", owner, balance);

    // Try totalSupply (may revert if not ERC721Enumerable)
    let total_supply = match contract.method::<_, U256>("totalSupply", ())?.call().await {
        Ok(s) => format!("{}", s),
        Err(_) => "Not supported (missing Enumerable)".to_string(),
    };
    println!("Total minted: {}", total_supply);

    // If owner has NFTs, fetch tokenURI for first few (if we know token IDs).
    // Without enumerable we can't list owned IDs easily; we'll just show an example.
    if balance > U256::zero() {
        // Attempt to get URI for token #1 (may not belong to owner)
        let token_id = U256::from(1);
        match contract.method::<_, String>("tokenURI", token_id)?.call().await {
            Ok(uri) => println!("Example tokenURI (id=1): {}", uri),
            Err(_) => println!("Could not fetch tokenURI (maybe id doesn't exist or not supported)."),
        }
    }
    println!();
    Ok(())
}

/// ERC-1155: print balance and URI for given token IDs.
async fn print_erc1155(
    provider: &Provider<Http>,
    token_addr: Address,
    owner: Address,
    token_ids: &[U256],
) -> Result<()> {
    let abi = build_abi(&[
        "balanceOf(address,uint256)(uint256)",
        "uri(uint256)(string)",
        "totalSupply(uint256)(uint256)", // optional
    ]);
    let contract = Contract::new(token_addr, abi, Arc::new(provider.clone()));

    println!("=== ERC-1155 at {} ===", token_addr);
    for &id in token_ids {
        let balance: U256 = contract
            .method("balanceOf", (owner, id))?
            .call()
            .await?;
        let uri: String = contract
            .method("uri", id)?
            .call()
            .await?;

        // Try totalSupply for this ID
        let total_supply = match contract
            .method::<_, U256>("totalSupply", id)?
            .call()
            .await
        {
            Ok(s) => format!("{}", s),
            Err(_) => "Not supported".to_string(),
        };

        println!("\nToken ID     : {}", id);
        println!("Balance of {}: {}", owner, balance);
        println!("Total Supply : {}", total_supply);
        println!("URI template : {}", uri);

        // Expand URI if it contains {id} (common in 1155)
        let expanded = uri.replace("{id}", &hex::encode(id.to_lower_be_bytes()));
        println!("Resolved URL : {}", expanded);
    }
    println!();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Ethereum (e.g., mainnet via Infura)
    let rpc_url = "https://mainnet.infura.io/v3/YOUR_PROJECT_ID";
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Example addresses (replace with real ones)
    let erc20 = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse::<Address>()?; // DAI
    let erc721 = "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D".parse::<Address>()?; // BAYC
    let erc1155 = "0x495f947276749Ce646f68AC8c248420045cb7b5e".parse::<Address>()?; // OpenSea shared

    let owner = "0xYourWalletAddress".parse::<Address>()?;

    print_erc20(&provider, erc20, owner).await?;
    print_erc721(&provider, erc721, owner).await?;
    // For ERC-1155, query some token IDs (e.g., 1, 2, 3)
    let ids = vec![U256::from(1), U256::from(2), U256::from(3)];
    print_erc1155(&provider, erc1155, owner, &ids).await?;

    Ok(())
}
// ```

// ---

// 3. Sample Output

// ```
// === ERC-20 (DAI) ===
// Name        : Dai Stablecoin
// Symbol      : DAI
// Decimals    : 18
// Total Supply: 5234567890.123456789 DAI
// Balance of 0xYourWalletAddress: 123.450000000000000000 DAI

// === ERC-721 (BAYC) ===
// Name        : Bored Ape Yacht Club
// Symbol      : BAYC
// NFTs owned by 0xYourWalletAddress: 2
// Total minted: 10000
// Example tokenURI (id=1): ipfs://QmZcH4LvTZbZqQeRrWxYyZ...

// === ERC-1155 at 0x495f947276749Ce646f68AC8c248420045cb7b5e ===

// Token ID     : 1
// Balance of 0xYourWalletAddress: 5
// Total Supply : Not supported
// URI template : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/{id}
// Resolved URL : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/0000000000000000000000000000000000000000000000000000000000000001

// Token ID     : 2
// Balance of 0xYourWalletAddress: 0
// Total Supply : Not supported
// URI template : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/{id}
// Resolved URL : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/0000000000000000000000000000000000000000000000000000000000000002

// Token ID     : 3
// Balance of 0xYourWalletAddress: 0
// Total Supply : Not supported
// URI template : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/{id}
// Resolved URL : https://api.opensea.io/api/v1/metadata/0x495f947276749Ce646f68AC8c248420045cb7b5e/0000000000000000000000000000000000000000000000000000000000000003
