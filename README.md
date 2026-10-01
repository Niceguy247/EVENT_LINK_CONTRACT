# EventLink Contract

Soroban smart contract for EventLink ticket issuance and lifecycle operations.

## Requirements

- Rust 1.86.0 with the `wasm32-unknown-unknown` target
- Stellar CLI

## Test and build

```sh
cargo fmt -- --check
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## Deploy to Stellar Testnet

```sh
stellar keys generate eventlink_deployer --network testnet --fund
./scripts/deploy.sh
```

The script deploys the contract, initializes its event state, and prints `SOROBAN_CONTRACT_ID`. Configure that value in the backend's environment. Set `CONTRACT_NAME`, `TOTAL_SUPPLY`, and `ROYALTY_BPS` to override deployment defaults.

The backend lives in [EVENT_LINK_BACKEND-](https://github.com/orbit-flow-labs/EVENT_LINK_BACKEND-); the frontend lives in [EVENT_LINK](https://github.com/orbit-flow-labs/EVENT_LINK).