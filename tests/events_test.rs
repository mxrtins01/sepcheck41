use sepcheck41::{MockTokenState, TokenEventType, TokenRunner};

#[test]
fn test_event_emission_lifecycle() {
    let mut state = MockTokenState::new("Native", "NTK", 7);
    state.mint("ALICE", 1_000_000);

    TokenRunner::transfer(&mut state, "ALICE", "BOB", 100_000, true).unwrap();
    assert_eq!(state.emitted_events.len(), 1);
    assert!(matches!(
        state.emitted_events[0].event,
        TokenEventType::Transfer { amount: 100_000, .. }
    ));

    TokenRunner::approve(&mut state, "ALICE", "SPENDER", 50_000, 2000, true).unwrap();
    assert_eq!(state.emitted_events.len(), 2);
    assert!(matches!(
        state.emitted_events[1].event,
        TokenEventType::Approve { amount: 50_000, .. }
    ));

    TokenRunner::burn(&mut state, "ALICE", 20_000, true).unwrap();
    assert_eq!(state.emitted_events.len(), 3);
    assert!(matches!(
        state.emitted_events[2].event,
        TokenEventType::Burn { amount: 20_000, .. }
    ));
}
