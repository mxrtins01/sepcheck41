use sepcheck41::{MockTokenState, TokenError, TokenRunner};

#[test]
fn test_direct_transfer_and_balance_conservation() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);
    state.mint("BOB", 500_000);

    let initial_total: i128 = state.balances.values().sum();

    // Alice transfers 200,000 to Bob
    TokenRunner::transfer(&mut state, "ALICE", "BOB", 200_000, true)
        .expect("Authorized transfer must succeed");

    assert_eq!(*state.balances.get("ALICE").unwrap(), 800_000);
    assert_eq!(*state.balances.get("BOB").unwrap(), 700_000);

    let final_total: i128 = state.balances.values().sum();
    assert_eq!(initial_total, final_total, "Total balance must be conserved");
}

#[test]
fn test_unauthorized_transfer_rejection() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);

    let err = TokenRunner::transfer(&mut state, "ALICE", "BOB", 200_000, false)
        .expect_err("Unauthorized transfer must fail");

    assert!(matches!(err, TokenError::Unauthorized(ref acc) if acc == "ALICE"));
}

#[test]
fn test_allowance_lifecycle_and_spend_down() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);

    // Alice approves Spender for 300,000 up to ledger 1500
    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 300_000, 1500, true)
        .expect("Approve must succeed");

    // Spender transfers 100,000 from Alice to Bob
    TokenRunner::transfer_from(&mut state, "SPENDER", "ALICE", "BOB", 100_000, true)
        .expect("Transfer_from must succeed");

    assert_eq!(*state.balances.get("ALICE").unwrap(), 900_000);
    assert_eq!(*state.balances.get("BOB").unwrap(), 100_000);

    let allowance = state
        .allowances
        .get(&("ALICE".to_string(), "SPENDER".to_string()))
        .unwrap();
    assert_eq!(allowance.amount, 200_000, "Allowance must decay by exact transfer amount");
}

#[test]
fn test_expired_allowance_rejection() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);
    state.current_ledger = 2000; // Past expiration!

    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 300_000, 1500, true)
        .expect("Approve must succeed");

    let err = TokenRunner::transfer_from(&mut state, "SPENDER", "ALICE", "BOB", 50_000, true)
        .expect_err("Expired allowance transfer must fail");

    assert!(matches!(err, TokenError::AllowanceExpired { .. }));
}
