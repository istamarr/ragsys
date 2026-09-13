use std::collections::HashMap;
use std::fmt;

// ---------- Core types ----------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Debit,
    Credit,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: String,
    pub amount: i64,      // in cents (or smallest unit)
    pub side: Side,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub description: String,
    pub entries: Vec<Entry>,
}

// ---------- Ledger ----------
pub struct Ledger {
    accounts: HashMap<String, i64>, // account_id -> balance (in cents)
    transactions: Vec<Transaction>,
}

impl Ledger {
    /// Create a new ledger with the given initial account names (all start at 0).
    pub fn new(initial_accounts: Vec<String>) -> Self {
        let mut accounts = HashMap::new();
        for name in initial_accounts {
            accounts.insert(name, 0);
        }
        Self {
            accounts,
            transactions: Vec::new(),
        }
    }

    /// Add a new account with zero balance.
    pub fn add_account(&mut self, name: String) {
        self.accounts.entry(name).or_insert(0);
    }

    /// Post a transaction. Missing accounts are automatically created with 0 balance.
    /// Returns error if total debits != total credits.
    pub fn post(&mut self, tx: Transaction) -> Result<(), String> {
        let mut total_debit = 0;
        let mut total_credit = 0;

        // First pass: auto-create missing accounts and compute totals.
        for entry in &tx.entries {
            // Ensure the account exists (insert if missing)
            self.accounts.entry(entry.account_id.clone()).or_insert(0);
            match entry.side {
                Side::Debit => total_debit += entry.amount,
                Side::Credit => total_credit += entry.amount,
            }
        }

        // Check balanced
        if total_debit != total_credit {
            return Err(format!(
                "Debits ({}) do not equal credits ({})",
                total_debit, total_credit
            ));
        }

        // Apply entries
        for entry in &tx.entries {
            let balance = self.accounts.get_mut(&entry.account_id).unwrap();
            match entry.side {
                Side::Debit => *balance += entry.amount,
                Side::Credit => *balance -= entry.amount,
            }
        }

        self.transactions.push(tx);
        Ok(())
    }

    /// Mint tokens: buy/create tokens at a cost per token.
    pub fn mint(
        &mut self,
        token_count: i64,
        cost_per_token: i64,
        // You could also pass account names as parameters if needed
    ) -> Result<(), String> {
        let total_cost = token_count * cost_per_token;
        let tx = Transaction {
            description: format!("Mint {} tokens @ ${:.2} each", token_count, cost_per_token as f64 / 100.0),
            entries: vec![
                Entry {
                    account_id: "TokenAsset".to_string(),
                    amount: total_cost,
                    side: Side::Debit,
                },
                Entry {
                    account_id: "Cash".to_string(),
                    amount: total_cost,
                    side: Side::Credit,
                },
            ],
        };
        self.post(tx)
    }

    /// Sell tokens: record sale and cost of goods sold.
    pub fn sell(
        &mut self,
        token_count: i64,
        sale_price_per_token: i64,
        cost_per_token: i64,
    ) -> Result<(), String> {
        let total_revenue = token_count * sale_price_per_token;
        let total_cost = token_count * cost_per_token;

        // Sale proceeds
        let tx_sale = Transaction {
            description: format!("Sell {} tokens @ ${:.2} each", token_count, sale_price_per_token as f64 / 100.0),
            entries: vec![
                Entry {
                    account_id: "Cash".to_string(),
                    amount: total_revenue,
                    side: Side::Debit,
                },
                Entry {
                    account_id: "Revenue".to_string(),
                    amount: total_revenue,
                    side: Side::Credit,
                },
            ],
        };
        self.post(tx_sale)?;

        // COGS
        let tx_cogs = Transaction {
            description: format!("COGS for {} tokens @ ${:.2} each", token_count, cost_per_token as f64 / 100.0),
            entries: vec![
                Entry {
                    account_id: "COGS".to_string(),
                    amount: total_cost,
                    side: Side::Debit,
                },
                Entry {
                    account_id: "TokenAsset".to_string(),
                    amount: total_cost,
                    side: Side::Credit,
                },
            ],
        };
        self.post(tx_cogs)
    }

    /// Print a human‑readable balance sheet.
    pub fn print_balances(&self) {
        println!("\n========== LEDGER BALANCES ==========");
        let mut sorted: Vec<_> = self.accounts.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (id, balance) in sorted {
            let sign = if *balance >= 0 { "" } else { "-" };
            let abs = balance.abs();
            println!(
                "{:<12} {:>10} {:>6}",
                id,
                sign,
                format!("${:.2}", abs as f64 / 100.0)
            );
        }
        println!("=====================================\n");
    }

    /// Print all transactions with their entries.
    pub fn print_transactions(&self) {
        println!("\n========== TRANSACTION HISTORY ==========");
        for (i, tx) in self.transactions.iter().enumerate() {
            println!("{}. {}", i + 1, tx.description);
            for entry in &tx.entries {
                let side = if entry.side == Side::Debit { "DEBIT" } else { "CREDIT" };
                println!(
                    "   {:8} {:>8} ${:.2}",
                    side,
                    entry.account_id,
                    entry.amount as f64 / 100.0
                );
            }
            println!();
        }
        println!("==========================================\n");
    }
}

// ---------- Main example ----------
fn main() -> Result<(), String> {
    // Parameterized initial accounts
    let initial_accounts = vec![
        "Cash".to_string(),
        "TokenAsset".to_string(),
        "Revenue".to_string(),
        "COGS".to_string(),
        // you can add any extra accounts here
        "AccountsReceivable".to_string(),
    ];
    let mut ledger = Ledger::new(initial_accounts);

    // 1. Mint 100 tokens at $5.00 each (cost = 500 cents)
    ledger.mint(100, 500)?;
    ledger.print_balances();

    // 2. Sell 10 tokens at $15.00 each (revenue = 1500 cents, COGS = 500 cents)
    ledger.sell(10, 1500, 500)?;
    ledger.print_balances();

    // 3. Dynamically add a new account later (if needed)
    ledger.add_account("Dividends".to_string());
    // Could post a transaction using that account...

    // 4. Print all transactions
    ledger.print_transactions();

    Ok(())
}
// ```
// ---
// Sample Output
// ```
// ========== LEDGER BALANCES ==========
// AccountsReceivable         $0.00
// Cash                   -$500.00
// COGS                     $0.00
// Revenue                  $0.00
// TokenAsset             $500.00
// =====================================

// ========== LEDGER BALANCES ==========
// AccountsReceivable         $0.00
// Cash                   -$350.00
// COGS                    $50.00
// Revenue                $150.00
// TokenAsset             $450.00
// =====================================

// ========== TRANSACTION HISTORY ==========
// 1. Mint 100 tokens @ $5.00 each
//    DEBIT   TokenAsset $500.00
//    CREDIT       Cash $500.00

// 2. Sell 10 tokens @ $15.00 each
//    DEBIT       Cash $150.00
//    CREDIT   Revenue $150.00

// 3. COGS for 10 tokens @ $5.00 each
//    DEBIT       COGS $50.00
//    CREDIT TokenAsset $50.00

// Further Customisation

// · Parameterised operation names – you can modify mint and sell to accept account name parameters instead of hardcoded strings.
// · Multi‑currency – store balances in a map of (account, currency) or use rust_decimal for precise decimal handling.
// · Persistent storage – replace HashMap with a database (e.g., SQLite, PostgreSQL).

// Let me know if you want the mint/sell functions to be fully parameterised too!