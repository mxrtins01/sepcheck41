use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub decimal: u32,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowanceEntry {
    pub from: String,
    pub spender: String,
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MockTokenState {
    pub metadata: Option<TokenMetadata>,
    pub total_supply: i128,
    pub balances: std::collections::HashMap<String, i128>,
    pub allowances: std::collections::HashMap<(String, String), AllowanceEntry>,
    pub current_ledger: u32,
}

impl MockTokenState {
    pub fn new(name: impl Into<String>, symbol: impl Into<String>, decimal: u32) -> Self {
        Self {
            metadata: Some(TokenMetadata {
                name: name.into(),
                symbol: symbol.into(),
                decimal,
            }),
            total_supply: 0,
            balances: std::collections::HashMap::new(),
            allowances: std::collections::HashMap::new(),
            current_ledger: 1000,
        }
    }

    pub fn mint(&mut self, account: impl Into<String>, amount: i128) {
        if amount > 0 {
            *self.balances.entry(account.into()).or_insert(0) += amount;
            self.total_supply += amount;
        }
    }

    pub fn total_balance_sum(&self) -> i128 {
        self.balances.values().copied().sum()
    }
}
