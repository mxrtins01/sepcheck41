use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenEventType {
    Transfer {
        from: String,
        to: String,
        amount: i128,
    },
    Approve {
        from: String,
        spender: String,
        amount: i128,
        expiration_ledger: u32,
    },
    Burn {
        from: String,
        amount: i128,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmittedTokenEvent {
    pub ledger: u32,
    pub event: TokenEventType,
}
