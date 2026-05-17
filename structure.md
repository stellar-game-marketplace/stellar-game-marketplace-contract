# Project Structure

## Monorepo Overview

```
stellar-game-marketplace/
├── contracts/   ← this repo (Soroban / Rust)
├── frontend/    ← Next.js + Freighter wallet
└── backend/     ← NestJS indexer, auth, analytics
```

---

## Contracts (this repo)

```
contracts/
├── nft-contract/
│   └── src/
│       ├── lib.rs        # mint, transfer, approve, transfer_from, views
│       └── storage.rs    # NFTMetadata, DataKey
├── marketplace-contract/
│   └── src/
│       ├── lib.rs        # list, cancel_listing, buy, place_bid, views
│       └── storage.rs    # Listing, Bid, DataKey
├── atomic-swap-contract/
│   └── src/
│       └── lib.rs        # create_offer, execute, cancel, get_offer
└── royalty-contract/
    └── src/
        ├── lib.rs        # register, distribute, get_config
        └── storage.rs    # RoyaltyConfig, DataKey
Cargo.toml                # workspace
Makefile
```

### Call flow

```
marketplace-contract  ──┐
                        ├──▶  atomic-swap-contract  ──▶  nft-contract (transfer_from)
royalty-contract      ──┘                           ──▶  payment token (SAC/XLM)
```

---

## Implemented Contract API

### `nft-contract`

| Function | Auth | Description |
|---|---|---|
| `initialize(admin)` | — | Set admin, seed next token ID |
| `mint(to, name, description, image_uri, game_id, rarity) → u64` | admin | Mint NFT, return token_id |
| `transfer(from, to, token_id)` | from | Transfer ownership, clears approval |
| `approve(owner, spender, token_id)` | owner | Grant single-token approval |
| `transfer_from(spender, to, token_id)` | spender | Transfer via approval, clears approval |
| `owner_of(token_id) → Address` | — | Current owner |
| `metadata(token_id) → NFTMetadata` | — | name, description, image_uri, game_id, rarity |
| `total_supply() → u64` | — | Count of minted tokens |

Storage: `Owner(token_id)`, `Metadata(token_id)`, `Approved(token_id)` in persistent; `Admin`, `NextId` in instance.

---

### `marketplace-contract`

| Function | Auth | Description |
|---|---|---|
| `initialize(admin, fee_wallet, fee_rate_bps)` | — | Configure fee split |
| `list(seller, nft_contract, token_id, price, expires_at) → u64` | seller | Create listing, return listing_id |
| `cancel_listing(seller, listing_id)` | seller | Mark listing inactive |
| `buy(buyer, listing_id, payment_token)` | buyer | Split fee → fee_wallet, remainder → seller |
| `place_bid(bidder, listing_id, amount, expires_at) → u64` | bidder | Record bid, return bid_id |
| `get_listing(listing_id) → Listing` | — | View listing |
| `get_bid(bid_id) → Bid` | — | View bid |

Fee formula: `fee = price × fee_rate_bps / 10_000`. NFT transfer must be handled separately (via atomic-swap-contract or direct call).

Storage: `Listing(id)`, `Bid(id)` in persistent; `Admin`, `FeeWallet`, `FeeRate`, `NextListingId`, `NextBidId` in instance.

---

### `atomic-swap-contract`

| Function | Auth | Description |
|---|---|---|
| `initialize()` | — | Seed next offer ID |
| `create_offer(buyer, seller, nft_contract, token_id, payment_token, payment_amount, expires_at) → u64` | buyer | Lock buyer payment into contract, return offer_id |
| `execute(seller, offer_id)` | seller | Atomic: NFT → buyer (cross-contract `transfer_from`), payment → seller |
| `cancel(buyer, offer_id)` | buyer | Refund buyer after expiry |
| `get_offer(offer_id) → SwapOffer` | — | View offer |

Atomicity guarantee: payment is locked on `create_offer`; `execute` calls NFT `transfer_from` then releases funds in the same transaction. If either step fails, the whole transaction reverts.

Storage: `Offer(id)` in persistent; `NextOfferId` in instance.

---

### `royalty-contract`

| Function | Auth | Description |
|---|---|---|
| `initialize(admin)` | — | Set admin |
| `register(caller, nft_contract, token_id, creator, rate_bps)` | caller | Register royalty config (max 1000 bps = 10%) |
| `distribute(payer, nft_contract, token_id, payment_token, sale_price) → i128` | payer | Transfer `rate_bps/10_000 × sale_price` to creator, return amount |
| `get_config(nft_contract, token_id) → RoyaltyConfig` | — | View creator + rate |

Storage: `Royalty(nft_contract, token_id)` in persistent; `Admin` in instance.

---

## Test Coverage

| Contract | Tests | Scenarios |
|---|---|---|
| `nft-contract` | 3 | mint + owner_of, transfer, approve + transfer_from |
| `marketplace-contract` | 3 | list + get, cancel_listing, place_bid |
| `royalty-contract` | 1 | register + get_config |
| `atomic-swap-contract` | — | integration tests (requires full mock env) |

---

## Implementation Status

| Feature | Status |
|---|---|
| NFT mint / transfer / approve | ✅ Done |
| Marketplace listings & bids | ✅ Done |
| Atomic swap (lock → execute → refund) | ✅ Done |
| Royalty registration & distribution | ✅ Done |
| Accept bid flow | 🔜 Phase 2 |
| Auction engine | 🔜 Phase 2 |
| Soulbound / burn | 🔜 Phase 2 |
| DAO governance | 🔜 Phase 4 |
| Cross-chain bridge | 🔜 Phase 4 |

---

## Frontend Integration (`stellar-game-marketplace/frontend`)

Stack: Next.js + Freighter wallet (Stellar browser extension).

### How to connect to contracts

1. Install `@stellar/stellar-sdk` and `@stellar/freighter-api`.
2. Use `SorobanRpc.Server` to submit transactions to testnet/mainnet.
3. Build contract calls with `Contract.call(method, ...args)` and sign via Freighter.

### Key integration points

| Contract | Frontend action | SDK call |
|---|---|---|
| `nft-contract` | Display NFT gallery | `metadata(token_id)`, `owner_of(token_id)` |
| `nft-contract` | Approve marketplace/swap | `approve(owner, spender, token_id)` |
| `marketplace-contract` | Create listing | `list(seller, nft_contract, token_id, price, expires_at)` |
| `marketplace-contract` | Buy NFT | `buy(buyer, listing_id, payment_token)` |
| `marketplace-contract` | Place bid | `place_bid(bidder, listing_id, amount, expires_at)` |
| `atomic-swap-contract` | Initiate P2P swap | `create_offer(...)` then seller calls `execute(seller, offer_id)` |
| `royalty-contract` | Show royalty info | `get_config(nft_contract, token_id)` |

### Typical buy flow (frontend)

```
1. User clicks "Buy"
2. Frontend calls nft-contract.approve(buyer, atomic_swap_id, token_id)   ← seller side, done at listing time
3. Frontend calls marketplace-contract.buy(buyer, listing_id, XLM_token)
   → fee split happens on-chain
4. For atomic safety: use atomic-swap-contract.create_offer + execute instead of step 3
5. Optionally call royalty-contract.distribute after settlement
```

---

## Backend Integration (`stellar-game-marketplace/backend`)

Stack: NestJS + Stellar Horizon / Soroban RPC.

### Responsibilities

- Index contract events (listings created, bids placed, swaps executed, royalties distributed)
- Serve REST/GraphQL API to frontend (listings, bids, NFT metadata cache)
- Handle auth (JWT + Stellar account verification)
- Analytics (volume, fees collected, top assets)

### Event indexing

Soroban contracts emit events via `env.events().publish(...)`. The backend should subscribe to:

| Event topic | Contract | Trigger |
|---|---|---|
| `list` | marketplace-contract | New listing created |
| `cancel` | marketplace-contract | Listing cancelled |
| `buy` | marketplace-contract | Purchase executed |
| `bid` | marketplace-contract | Bid placed |
| `execute` | atomic-swap-contract | Swap completed |
| `cancel` | atomic-swap-contract | Swap refunded |
| `distribute` | royalty-contract | Royalty paid |

> Note: event publishing is not yet wired in the current contract code — add `env.events().publish((symbol,), payload)` calls to each mutating function before backend indexing.

### Indexer setup (NestJS)

```ts
// Poll Soroban RPC for contract events
const server = new SorobanRpc.Server(RPC_URL);
const events = await server.getEvents({
  startLedger,
  filters: [{ type: 'contract', contractIds: [MARKETPLACE_ID, SWAP_ID, ROYALTY_ID] }],
});
```

Store indexed data in PostgreSQL; expose via REST endpoints consumed by the Next.js frontend.
