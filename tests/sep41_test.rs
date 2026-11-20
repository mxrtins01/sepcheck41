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

#[test]
fn test_burn_and_supply_invariant() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);
    assert_eq!(state.total_supply, 1_000_000);
    TokenRunner::verify_supply_invariant(&state).expect("Initial supply invariant valid");

    // Authorized burn
    TokenRunner::burn(&mut state, "ALICE", 400_000, true).expect("Burn must succeed");
    assert_eq!(*state.balances.get("ALICE").unwrap(), 600_000);
    assert_eq!(state.total_supply, 600_000);
    TokenRunner::verify_supply_invariant(&state).expect("Post-burn supply invariant valid");

    // Over-burn
    let err = TokenRunner::burn(&mut state, "ALICE", 700_000, true).expect_err("Over-burn must fail");
    assert!(matches!(err, TokenError::InsufficientBalance { .. }));

    // Unauthorized burn
    let unauth_err = TokenRunner::burn(&mut state, "ALICE", 100_000, false).expect_err("Unauth burn fails");
    assert!(matches!(unauth_err, TokenError::Unauthorized(..)));
}

#[test]
fn test_burn_from_with_allowance() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("ALICE", 1_000_000);

    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 500_000, 1500, true)
        .expect("Approve must succeed");

    // Spender burns 200,000 of Alice's tokens
    TokenRunner::burn_from(&mut state, "SPENDER", "ALICE", 200_000, true)
        .expect("Burn_from must succeed");

    assert_eq!(*state.balances.get("ALICE").unwrap(), 800_000);
    assert_eq!(state.total_supply, 800_000);
    let allowance = state.allowances.get(&("ALICE".to_string(), "SPENDER".to_string())).unwrap();
    assert_eq!(allowance.amount, 300_000);
    TokenRunner::verify_supply_invariant(&state).expect("Post burn_from invariant valid");

    // Over allowance burn_from
    let err = TokenRunner::burn_from(&mut state, "SPENDER", "ALICE", 400_000, true)
        .expect_err("Over allowance must fail");
    assert!(matches!(err, TokenError::InsufficientAllowance { .. }));
}

#[test]
fn test_allowance_invariants() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 500_000, 1500, true).unwrap();
    TokenRunner::verify_allowance_invariants(&state).expect("Valid allowance invariant");

    // Manually corrupt allowance to negative
    state.allowances.get_mut(&("ALICE".to_string(), "SPENDER".to_string())).unwrap().amount = -50;
    let err = TokenRunner::verify_allowance_invariants(&state).expect_err("Negative allowance must fail");
    assert!(matches!(err, TokenError::NegativeAllowance { .. }));
}

#[test]
fn test_clawback_operation_and_events() {
    let mut state = MockTokenState::new("Clawback Token", "CLAW", 7);
    state.mint("USER_A", 1_000_0000000);

    TokenRunner::clawback(&mut state, "ADMIN_A", "USER_A", 400_0000000, true)
        .expect("Clawback should succeed");

    assert_eq!(state.balances.get("USER_A"), Some(&600_0000000));
    assert_eq!(state.total_supply, 600_0000000);

    assert!(state.emitted_events.iter().any(|e| matches!(
        &e.event,
        sepcheck41::TokenEventType::Clawback { admin, from, amount }
            if admin == "ADMIN_A" && from == "USER_A" && *amount == 400_0000000
    )));
}

#[test]
fn test_zero_allowance_reset() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 500_000, 1500, true).unwrap();

    let entry = state.allowances.get(&("ALICE".to_string(), "SPENDER".to_string())).unwrap();
    assert_eq!(entry.amount, 500_000);

    TokenRunner::reset_allowance(&mut state, "ALICE", "SPENDER", true).unwrap();
    let reset_entry = state.allowances.get(&("ALICE".to_string(), "SPENDER".to_string())).unwrap();
    assert_eq!(reset_entry.amount, 0);

    let err = TokenRunner::reset_allowance(&mut state, "ALICE", "SPENDER", false).unwrap_err();
    assert!(matches!(err, TokenError::Unauthorized(..)));
}

#[test]
fn test_metadata_scaling_helpers() {
    let meta = sepcheck41::TokenMetadata {
        decimal: 7,
        name: "USD Coin".into(),
        symbol: "USDC".into(),
    };

    assert_eq!(meta.format_scaled(10_0000000), "10.0000000");
    assert_eq!(meta.format_scaled(1_5000000), "1.5000000");
    assert_eq!(meta.format_scaled(42), "0.0000042");
}

#[test]
fn test_clawback_unauthorized_rejection_and_supply_invariant() {
    let mut state = MockTokenState::new("Native Token", "NTK", 7);
    state.mint("USER_X", 500);

    let err = TokenRunner::clawback(&mut state, "FAKE_ADMIN", "USER_X", 100, false).unwrap_err();
    assert!(matches!(err, TokenError::Unauthorized(..)));

    TokenRunner::verify_supply_invariants(&state).expect("Supply invariant should hold");
}
