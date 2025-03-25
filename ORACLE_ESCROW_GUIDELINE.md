# Trustless Work Smart Escrow with Oracle Integration

## Overview

This project extends the **Trustless Work Smart Escrow** contract to support conditional fund release based on an external, off-chain event. The contract integrates an oracle that verifies an off-chain condition before releasing the escrowed funds. This implementation is particularly useful for decentralized betting, conditional payments, and event-driven smart contract automation.

## Features

- **Oracle Integration**: The contract fetches off-chain data to validate conditions for fund release.
- **Trustless Escrow Mechanism**: Funds remain locked until the oracle verifies the condition as `true`.
- **Automated Fund Release**: The contract automatically transfers funds based on verified oracle responses.
- **Security Measures**: Ensures only authorized oracles can update conditions.

## Implementation Details

### 1️⃣ Oracle Integration

- Implemented a **Mock Oracle Contract (********`mock_oracle.rs`********\*\*\*\*)** to simulate an off-chain data source.
- Functions:
  - `initialize(e: Env, result: Option<bool>)`: Initializes the oracle with a condition result.
  - `get_result(e: Env) -> Option<bool>`: Fetches the stored oracle result.
  - `set_result(e: Env, result: Option<bool>)`: Updates the oracle condition.

### 2️⃣ Escrow Contract Modification

- Modified the **escrow contract** to:
  - Hold funds until the oracle confirms an event.
  - Include `release_funds_based_on_oracle()` function that:
    - Calls the oracle to verify the condition.
    - Transfers the locked funds accordingly.
    - Ensures only authorized oracles can update the escrow condition.

### 3️⃣ Testing

- Used **mock oracles** to simulate real-world conditions.
- Created **unit tests** to verify:
  - Funds remain locked until the oracle confirms the condition.
  - Only verified oracle responses trigger fund release.
  - Malicious updates are rejected.
- Successfully deployed and tested using Soroban’s smart contract framework.

## How to Run the Project

### Prerequisites

- Rust installed (`cargo` and `rustc`)
- Soroban SDK installed

### Steps to Run

1. **Clone the repository**:
   ```sh
   git clone https://github.com/your-repo/Trustless-Work-Smart-Escrow.git
   cd Trustless-Work-Smart-Escrow
   ```
2. **Compile the contracts**:
   ```sh
   cargo build
   ```
3. **Run tests**:
   ```sh
   cargo test
   ```
4. **Deploy the contract**:
   ```sh
   stellar contract deploy --wasm ./target/wasm32-unknown-unknown/release/escrow.wasm
   ```
5. **Interact with the Oracle and Escrow**:
   ```sh
   soroban contract invoke --id <CONTRACT_ID> --fn set_result --arg true
   soroban contract invoke --id <CONTRACT_ID> --fn release_funds_based_on_oracle
   ```



