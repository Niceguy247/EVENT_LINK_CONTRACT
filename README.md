# EventLink Soroban Contract

Rust/Soroban contract source for the EventLink ticketing prototype. This repository contains the contract crate, contract-specific CI, and the Testnet deployment helper. The web application and API live in separate repositories.

> **Network and maturity:** The deployment helper targets Stellar Testnet. The crate currently has no unit-test cases; the `cargo test` CI step verifies compilation only. Treat this code as a prototype until contract behavior, authorization, and upgrade policy have been independently reviewed.

## Repositories

| Project | Repository |
| --- | --- |
| Frontend | [EVENT_LINK](https://github.com/orbit-flow-labs/EVENT_LINK) |
| Backend API | [EVENT_LINK_BACKEND-](https://github.com/orbit-flow-labs/EVENT_LINK_BACKEND-) |
| Soroban contract | [EVENT_LINK_CONTRACT](https://github.com/orbit-flow-labs/EVENT_LINK_CONTRACT) |

## Contract interface

The current contract stores one event (`event_id` is initialized to `101`) and ticket records in Soroban storage.

| Method | Purpose |
| --- | --- |
| `initialize(organizer, name, total_supply, royalty_bps)` | Initialize the event once; rejects an empty name, zero supply, and royalties above 10,000 basis points. |
| `mint_ticket(buyer, tier_name, price, claim_secret_hash)` | Create a `Valid` ticket, or a `Claimable` ticket when a claim hash is provided; requires positive price, available supply, and a unique non-empty claim hash. |
| `claim_ticket(claim_secret_hash, new_owner)` | Require the new owner to authorize, consume the matching claim link, and transition its ticket from `Claimable` to `Valid`. |
| `check_in_ticket(organizer, ticket_id)` | Require the configured organizer and transition a `Valid` ticket to terminal `ProofNFT`; any resale listing is cleared. |
| `list_resale(seller, ticket_id, resale_price)` | Require the current owner and a `Valid`, unlisted ticket; enforce a positive price no greater than 150% of its original price. |
| `buy_resale(buyer, ticket_id)` | Require buyer authorization and a listed `Valid` ticket; update ownership, clear the listing, and emit royalty/seller payout amounts. |
| `get_ticket(ticket_id)` | Read a stored ticket record. |

The implemented ticket lifecycle is `Claimable -> Valid -> ProofNFT` for claimed tickets, or `Valid -> ProofNFT` for tickets without a claim link. `ProofNFT` is terminal: it cannot be listed or purchased. A claim link is removed when claimed; there is no contract method for manually expiring or revoking a link. Listing cancellation is only implicit when check-in clears the listing; there is no standalone cancellation method. These are per-ticket transitions; the contract does not model event-level lifecycle states.

Events are emitted for initialization, minting, claims, check-in, listings, and resale. Their payloads are method-specific and should be interpreted alongside the state transitions above, subject to ledger event retention and indexing policy.

### Current limitations

- Only one event is represented by the contract state; per-event isolation and multi-event storage are not implemented.
- The contract has no application-level claim-link expiry or revocation flow. Claim links and tickets are stored with ledger-managed persistence and can expire if not renewed according to network storage rules.
- A resale listing has no standalone cancellation operation; check-in clears it as part of making the ticket terminal.
- `buy_resale` updates contract ownership and calculates payout values, but does not transfer payment or distribute royalties.
- Unit tests cover large-value royalty calculations, claim-link uniqueness, claim-to-check-in lifecycle invariants, and listing cancellation on check-in. Authorization, inventory boundaries, resale caps, and event schemas need further coverage before production use.
- This repository does not define a contract upgrade or migration policy.

## Requirements

- Rust 1.86.0 with `rustfmt`
- The `wasm32-unknown-unknown` target
- Stellar CLI for Testnet deployment

Install the target with:

```sh
rustup toolchain install 1.86.0 --component rustfmt
rustup target add wasm32-unknown-unknown --toolchain 1.86.0
```

## Build and validate

Run from the repository root:

```sh
cargo fmt -- --check
cargo test
cargo build --target wasm32-unknown-unknown --release
```

The GitHub Actions workflow runs these same checks on pull requests and pushes to `main`.

### Contract errors

Expected contract failures use the public `Error` numeric codes (1-21) instead of string panics; `ContractError` is an alias for this enum. Clients should decode these errors by enum value; Soroban authorization failures from `require_auth` remain native authorization errors. Missing tickets and invalid claim links are reported as `TicketNotFound` and `InvalidClaimLink`, respectively.

## Deploy to Stellar Testnet

First install and configure Stellar CLI, then create or select a funded Testnet key named `eventlink_deployer`:

```sh
stellar keys generate eventlink_deployer --network testnet --fund
./scripts/deploy.sh
```

The script builds the WASM, deploys it, initializes event state, and prints `SOROBAN_CONTRACT_ID`. Supply optional environment values to customize initialization:

```sh
CONTRACT_NAME="EventLink Demo" TOTAL_SUPPLY=1000 ROYALTY_BPS=500 ./scripts/deploy.sh
```

Store the resulting contract ID in the backend's `SOROBAN_CONTRACT_ID` environment variable. Never commit deployer secrets or production credentials.

## Contributing

1. Open or select an issue and describe the invariant or behavior being changed.
2. Add Soroban tests for successful and rejected calls; avoid relying on compilation alone.
3. Run formatting, tests, and the WASM build locally.
4. Include the issue or Drips Wave task reference, validation output, and any deployment impact in the PR.
5. Coordinate interface changes with the backend and frontend maintainers.

Follow the current Drips Wave 10 contribution rules where applicable; this repository does not imply program sponsorship or acceptance.

## License

MIT. See [LICENSE](LICENSE).