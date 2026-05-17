# stellar-game-marketplace — contracts

Soroban smart contracts powering the decentralized game asset marketplace on Stellar. Part of a 3-repo organization alongside [`frontend`](https://github.com/stellar-game-marketplace/frontend) and [`backend`](https://github.com/stellar-game-marketplace/backend).

---

## Contracts

| Contract | Description |
|---|---|
| `nft-contract` | Mint, transfer, and approve game asset NFTs |
| `marketplace-contract` | Create listings, place bids, execute purchases with fee split |
| `atomic-swap-contract` | Fraud-free peer-to-peer NFT ↔ payment settlement |
| `royalty-contract` | Register and distribute creator royalties on resale |

---

## Architecture

```
marketplace-contract  ──┐
                        ├──▶  atomic-swap-contract  ──▶  nft-contract
royalty-contract      ──┘                           ──▶  token (XLM/SAC)
```

- **NFT contract** owns token state (owner, metadata, approvals)
- **Atomic swap** locks buyer payment, calls `transfer_from` on the NFT contract, releases funds — or refunds both parties on failure
- **Marketplace** handles listings and bids; routes payments through fee wallet before settlement
- **Royalty** distributes a creator cut (in basis points) on every sale

---

## Project Structure

```
contracts/
├── nft-contract/
│   └── src/
│       ├── lib.rs        # mint, transfer, approve, transfer_from, views
│       └── storage.rs    # NFTMetadata, DataKey
├── marketplace-contract/
│   └── src/
│       ├── lib.rs        # list, cancel, buy, place_bid
│       └── storage.rs    # Listing, Bid, DataKey
├── atomic-swap-contract/
│   └── src/
│       └── lib.rs        # create_offer, execute, cancel
└── royalty-contract/
    └── src/
        ├── lib.rs        # register, distribute, get_config
        └── storage.rs    # RoyaltyConfig, DataKey
Cargo.toml                # workspace
Makefile
```

---

## Prerequisites

- [Rust](https://rustup.rs/) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli)

```bash
rustup target add wasm32-unknown-unknown
```

---

## Getting Started

```bash
git clone https://github.com/stellar-game-marketplace/contracts
cd contracts

# check all contracts
cargo check

# run tests
cargo test

# build all WASM binaries
make build

# build a single contract
make build-nft-contract
```

---

## Contract API Reference

### `nft-contract`

```rust
initialize(admin: Address)
mint(to, name, description, image_uri, game_id, rarity) -> u64   // returns token_id
transfer(from, to, token_id)
approve(owner, spender, token_id)
transfer_from(spender, to, token_id)
owner_of(token_id) -> Address
metadata(token_id) -> NFTMetadata
total_supply() -> u64
```

### `marketplace-contract`

```rust
initialize(admin, fee_wallet, fee_rate_bps)
list(seller, nft_contract, token_id, price, expires_at) -> u64    // returns listing_id
cancel_listing(seller, listing_id)
buy(buyer, listing_id, payment_token)                             // splits fee + seller amount
place_bid(bidder, listing_id, amount, expires_at) -> u64
get_listing(listing_id) -> Listing
get_bid(bid_id) -> Bid
```

### `atomic-swap-contract`

```rust
initialize()
create_offer(buyer, seller, nft_contract, token_id, payment_token, payment_amount, expires_at) -> u64
execute(seller, offer_id)    // atomic: NFT → buyer, payment → seller
cancel(buyer, offer_id)      // refund after expiry
get_offer(offer_id) -> SwapOffer
```

### `royalty-contract`

```rust
initialize(admin)
register(caller, nft_contract, token_id, creator, rate_bps)   // max 1000 bps (10%)
distribute(payer, nft_contract, token_id, payment_token, sale_price) -> i128
get_config(nft_contract, token_id) -> RoyaltyConfig
```

---

## Tests

```bash
cargo test
```

```
nft-contract         3 tests  ✓  mint/owner, transfer, approve+transfer_from
marketplace-contract 3 tests  ✓  list, cancel, place_bid
royalty-contract     1 test   ✓  register + get_config
atomic-swap-contract          —  integration tests (requires full mock env)
```

---

## Deployment

```bash
# deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/nft_contract.wasm \
  --source <ACCOUNT> \
  --network testnet

# initialize after deploy
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ACCOUNT> \
  --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>
```

Repeat for each contract. Pass the deployed `nft-contract` address into `atomic-swap-contract` calls at invocation time.

---

## Implementation Status

| Feature | Status |
|---|---|
| NFT mint / transfer / approve | ✅ Done |
| Marketplace listings & bids | ✅ Done |
| Atomic swap (lock → execute → refund) | ✅ Done |
| Royalty registration & distribution | ✅ Done |
| Auction engine | 🔜 Phase 2 |
| Soulbound / burn | 🔜 Phase 2 |
| Accept bid flow | 🔜 Phase 2 |
| DAO governance | 🔜 Phase 4 |
| Cross-chain bridge | 🔜 Phase 4 |

---

## Related Repos

- [`stellar-game-marketplace/frontend`](https://github.com/stellar-game-marketplace/frontend) — Next.js + Freighter wallet UI
- [`stellar-game-marketplace/backend`](https://github.com/stellar-game-marketplace/backend) — NestJS indexer, auth, analytics

---

## License

MIT
