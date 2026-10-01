#!/usr/bin/env bash
set -euo pipefail

CONTRACT_NAME="${CONTRACT_NAME:-EventLink Ticketing}"
TOTAL_SUPPLY="${TOTAL_SUPPLY:-1000}"
ROYALTY_BPS="${ROYALTY_BPS:-500}"

cargo build --target wasm32-unknown-unknown --release
WASM_PATH="target/wasm32-unknown-unknown/release/event_ticket.wasm"
if [[ ! -f "$WASM_PATH" ]]; then
  printf 'Contract WASM not found at %s\n' "$WASM_PATH" >&2
  exit 1
fi

stellar keys generate eventlink_deployer --network testnet --fund || true
DEPLOYER_ADDRESS="$(stellar keys address eventlink_deployer)"
CONTRACT_ID="$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source eventlink_deployer \
  --network testnet)"

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source eventlink_deployer \
  --network testnet \
  -- \
  initialize \
  --organizer "$DEPLOYER_ADDRESS" \
  --name "$CONTRACT_NAME" \
  --total_supply "$TOTAL_SUPPLY" \
  --royalty_bps "$ROYALTY_BPS"

printf 'SOROBAN_CONTRACT_ID=%s\n' "$CONTRACT_ID"