use sepcheck41::{MockTokenState, TokenRunner};

#[test]
fn test_batch_transfer_atomicity() {
    let mut state = MockTokenState::new("Native", "NTK", 7);
    state.mint("ALICE", 1_000_000);

    // 1st transfer valid, 2nd transfer fails (Bob only has 0, asks 500k)
    let batch = vec![
        ("ALICE", "BOB", 100_000),
        ("BOB", "CHARLIE", 500_000), // FAILS!
    ];

    let res = TokenRunner::batch_transfer(&mut state, &batch, true);
    assert!(res.is_err());

    // Verify state was completely rolled back
    assert_eq!(*state.balances.get("ALICE").unwrap(), 1_000_000);
    assert_eq!(state.balances.get("BOB").copied().unwrap_or(0), 0);
}
