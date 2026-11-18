use crate::types::{AllowanceEntry, MockTokenState};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    #[error("Missing authorization from account '{0}'")]
    Unauthorized(String),

    #[error("Insufficient balance for account '{account}': requested {requested}, available {available}")]
    InsufficientBalance {
        account: String,
        requested: i128,
        available: i128,
    },

    #[error("Insufficient allowance for spender '{spender}' from '{from}': requested {requested}, allowed {allowed}")]
    InsufficientAllowance {
        spender: String,
        from: String,
        requested: i128,
        allowed: i128,
    },

    #[error("Allowance expired at ledger {expiration}, current ledger {current}")]
    AllowanceExpired {
        expiration: u32,
        current: u32,
    },

    #[error("Negative transfer amount {0} disallowed")]
    NegativeAmount(i128),
}

pub struct TokenRunner;

impl TokenRunner {
    pub fn transfer(
        state: &mut MockTokenState,
        from: &str,
        to: &str,
        amount: i128,
        authorized: bool,
    ) -> Result<(), TokenError> {
        if amount < 0 {
            return Err(TokenError::NegativeAmount(amount));
        }
        if !authorized {
            return Err(TokenError::Unauthorized(from.to_string()));
        }

        let from_bal = state.balances.get(from).copied().unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance {
                account: from.to_string(),
                requested: amount,
                available: from_bal,
            });
        }

        *state.balances.entry(from.to_string()).or_insert(0) -= amount;
        *state.balances.entry(to.to_string()).or_insert(0) += amount;
        Ok(())
    }

    pub fn approve(
        state: &mut MockTokenState,
        from: &str,
        spender: &str,
        amount: i128,
        expiration_ledger: u32,
        authorized: bool,
    ) -> Result<(), TokenError> {
        if !authorized {
            return Err(TokenError::Unauthorized(from.to_string()));
        }

        state.allowances.insert(
            (from.to_string(), spender.to_string()),
            AllowanceEntry {
                from: from.to_string(),
                spender: spender.to_string(),
                amount,
                expiration_ledger,
            },
        );
        Ok(())
    }

    pub fn transfer_from(
        state: &mut MockTokenState,
        spender: &str,
        from: &str,
        to: &str,
        amount: i128,
        authorized: bool,
    ) -> Result<(), TokenError> {
        if amount < 0 {
            return Err(TokenError::NegativeAmount(amount));
        }
        if !authorized {
            return Err(TokenError::Unauthorized(spender.to_string()));
        }

        let key = (from.to_string(), spender.to_string());
        let allowance = state.allowances.get(&key).cloned().ok_or_else(|| {
            TokenError::InsufficientAllowance {
                spender: spender.to_string(),
                from: from.to_string(),
                requested: amount,
                allowed: 0,
            }
        })?;

        if allowance.expiration_ledger < state.current_ledger {
            return Err(TokenError::AllowanceExpired {
                expiration: allowance.expiration_ledger,
                current: state.current_ledger,
            });
        }

        if allowance.amount < amount {
            return Err(TokenError::InsufficientAllowance {
                spender: spender.to_string(),
                from: from.to_string(),
                requested: amount,
                allowed: allowance.amount,
            });
        }

        let from_bal = state.balances.get(from).copied().unwrap_or(0);
        if from_bal < amount {
            return Err(TokenError::InsufficientBalance {
                account: from.to_string(),
                requested: amount,
                available: from_bal,
            });
        }

        // Deduct balance and allowance
        *state.balances.entry(from.to_string()).or_insert(0) -= amount;
        *state.balances.entry(to.to_string()).or_insert(0) += amount;
        state.allowances.get_mut(&key).unwrap().amount -= amount;

        Ok(())
    }
}
