# sepcheck41

SEP-41 Soroban Token Behavioral Conformance and Interoperability Test Suite.

## Overview

SEP-0041 standardizes the token interface on Soroban, establishing common methods for balances, transfers, allowances, and minting/burning. However, behavioral differences in arithmetic overflow handling, authorization verification, allowance decay semantics, and event emissions can cause subtle integration issues across DeFi protocols.

`sepcheck41` is an automated conformance test runner that verifies custom token implementations against the canonical SEP-41 specification and Stellar Asset Contract (SAC) behavioral standards.

### Test Coverage

- **Interface Conformance**: Verifies function signatures, argument types, and return values (`balance`, `spendable_balance`, `authorized`, `transfer`, `transfer_from`, `approve`, `burn`, `burn_from`, `decimals`, `name`, `symbol`).
- **Allowance & Expiry Semantics**: Tests precise allowance consumption, expiration ledger thresholds, and edge-case zero allowances.
- **Arithmetic Safety**: Validates boundary conditions including zero-amount operations, balance exhaustion, and `u128::MAX` boundary behavior.
- **Authorization Enforcement**: Asserts strict invoker and signature authentication across all mutating calls.
- **Event Audit**: Validates event topics, data payloads, and emission sequences against standard schemas.

## Getting Started

### Prerequisites

- Rust 1.79+
- Soroban SDK

### Installation & Usage

```bash
# Test a token contract implementation
sepcheck41 test --wasm path/to/token.wasm

# Compare custom token against canonical reference implementation
sepcheck41 compare --target path/to/token.wasm --reference path/to/canonical_sac.wasm
```

## Roadmap

- [ ] SEP-41 core interface verification suite
- [ ] Allowance lifecycle and expiration test harness
- [ ] Boundary condition and arithmetic edge-case tests
- [ ] Structured conformance report generator

## License

Apache-2.0 or MIT
