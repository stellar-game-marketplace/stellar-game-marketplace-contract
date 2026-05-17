# Stellar Game Marketplace

> A fully decentralized gaming asset marketplace built on Stellar using Soroban smart contracts — enabling fraud-free, peer-to-peer trading of NFTs, in-game items, and currencies without centralized custody.

---

## Overview

Traditional game marketplaces (Steam, Epic, Roblox) are centralized, custodial, and prone to fake trades, chargebacks, item duplication, and high fees. This platform eliminates those problems by combining:

- **Stellar SDEX** — native decentralized orderbooks and path payments
- **Soroban Smart Contracts** — NFT logic, escrow, royalties, auctions, atomic swaps
- **Atomic Settlement** — either both assets exchange, or nothing happens

Players retain true ownership of their assets. No intermediaries. No fraud.

---

## Tradeable Asset Types

| Asset | Type |
|---|---|
| Rare Sword, Legendary Skin | NFT |
| Land Plot, Collectible Card | NFT |
| Clan Badge, Achievement | Soulbound NFT |
| In-game Currency | Fungible Token |
| Tournament Ticket | NFT |

---

## Architecture

```
┌─────────────────────────────┐
│        Frontend UI          │
│     React / Next.js         │
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│      Backend API Layer      │
│   Node.js / NestJS          │
│   Marketplace Indexer       │
└──────┬──────────────┬───────┘
       ▼              ▼
┌─────────────┐ ┌─────────────┐
│ Soroban NFT │ │ Atomic Swap │
│  Contracts  │ │  Contracts  │
└──────┬──────┘ └──────┬──────┘
       └──────┬─────────┘
              ▼
┌─────────────────────────────┐
│    Stellar Ledger + SDEX    │
└─────────────────────────────┘
```

---

## Core Features

### 1. NFT Game Asset System
Every in-game item is minted as an on-chain NFT with IPFS-hosted metadata.

```json
{
  "name": "Dragon Slayer Sword",
  "description": "Legendary weapon from the Abyss Realm",
  "image": "ipfs://Qm...",
  "attributes": [
    { "trait_type": "Damage", "value": 95 },
    { "trait_type": "Rarity", "value": "Legendary" }
  ]
}
```

### 2. Decentralized Orderbook
Listings are stored on Stellar SDEX or a custom Soroban orderbook. Players create buy/sell orders, bids, and auctions.

```json
{
  "seller": "GABC...",
  "asset_id": "dragon_sword_77",
  "price": "250 XLM",
  "expires": 1728820000
}
```

### 3. Atomic Swap Settlement
The core anti-scam mechanism. Both assets lock simultaneously — if any step fails, everything is refunded.

```
Buyer locks 500 XLM  →  Contract verifies ownership & signatures
Seller locks NFT     →  Assets exchange simultaneously
                     →  Failure? Full refund to both parties
```

```rust
pub fn swap(buyer: Address, seller: Address, nft: Address, payment: i128)
// verifies signatures → verifies NFT ownership → locks assets → executes transfers
```

### 4. Royalty Engine
Creators earn automatically on every resale.

```
NFT resold for 100 XLM  →  5 XLM to creator  →  95 XLM to seller
```

### 5. Escrow System
High-value trades are held in escrow until both parties confirm. Auto-refunds on expiration.

### 6. Cross-Game Asset Compatibility
A single wallet holds assets from multiple games via a universal gamer identity. Games integrate via the provided SDK.

### 7. Anti-Fraud System
- Ownership verification before listing
- Unique token IDs prevent duplication
- Wash trading and bot detection
- Wallet reputation scores

### 8. Liquidity Pools
Stellar AMMs and Soroswap pools enable instant game token conversion and liquidity incentives.

---

## Smart Contract Modules

```
contracts/
├── nft-contract/          # Minting, transfer, metadata, storage
├── marketplace-contract/  # Listings, bids, sales, fees
├── atomic-swap-contract/  # Swap logic, validation, escrow
├── royalty-contract/      # Royalty distribution
├── auctions/              # Auction engine
├── governance/            # DAO voting
└── rewards/               # Staking and incentives
```

---

## Project Structure

```
stellar-game-marketplace/
├── contracts/
│   ├── nft-contract/src/         # lib.rs, mint.rs, transfer.rs, metadata.rs
│   ├── marketplace-contract/src/ # listings.rs, bids.rs, sales.rs, fees.rs
│   ├── atomic-swap-contract/src/ # swap.rs, validation.rs, escrow.rs
│   └── royalty-contract/
├── frontend/
│   ├── app/, components/, hooks/, services/
│   └── pages/                    # marketplace, inventory, auctions, profile
├── backend/src/
│   ├── indexer/, auth/, analytics/
│   └── fraud-detection/
├── sdk/
│   ├── js-sdk/
│   └── unity-sdk/
├── scripts/
├── docs/
└── docker/
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Blockchain | Stellar |
| Smart Contracts | Soroban (Rust/WASM) |
| Frontend | Next.js + Freighter Wallet |
| Backend | NestJS |
| Indexing | SubQuery |
| Database | PostgreSQL |
| Storage | IPFS |
| Realtime | WebSockets |

---

## Marketplace Flows

**Selling an Asset**
```
Connect wallet → Select NFT → Set price → Sign transaction → Listing stored on-chain
```

**Buying an Asset**
```
Select listing → Lock funds → Atomic swap executes → NFT transferred → Payment released
```

**User Journey**
```
Sign in → Connect Freighter → Mint NFT → List for 200 XLM
       → Buyer purchases → Atomic swap → Ownership transferred → Seller receives XLM
```

---

## Revenue Model

| Source | Detail |
|---|---|
| Marketplace Fees | 1–2% per trade |
| Premium Listings | Featured asset placement |
| Launchpad Fees | New game asset launches |
| Creator Royalties | Revenue sharing |
| Tournament Commissions | Entry fee percentage |

---

## Security Model

| Concern | Mitigation |
|---|---|
| Reentrancy | Rust memory safety + state guards |
| Unauthorized swaps | Signature verification |
| Stuck funds | Escrow expiration + auto-refund |
| Spam | Rate limiting |
| Treasury risk | Multi-sig |

---

## Storage Strategy

- **On-chain:** ownership records, listings, trade history, balances
- **Off-chain (IPFS):** images, metadata, videos, large assets

---

## Development Phases

| Phase | Milestone |
|---|---|
| **1 — MVP** | Wallet login, NFT minting, listings, atomic swap |
| **2 — Marketplace** | Auctions, royalties, game SDK, analytics |
| **3 — Liquidity** | AMMs, staking, rewards, yield farming |
| **4 — Ecosystem** | Cross-chain bridges, DAO governance, launchpad, tournaments |

**Estimated Timeline:** 6 months to audit-ready launch (1 month per phase + audit)

---

## Advanced Features (Post-MVP)

- **Rental Marketplace** — time-limited asset rentals
- **NFT Lending** — use NFTs as collateral
- **Guild Treasury** — clan-managed pooled assets
- **AI Fraud Detection** — suspicious pricing, wash trading, bot detection
- **Cross-Chain Bridges** — import Ethereum, Solana, and Polygon assets into Stellar

---

## Why Stellar

| Feature | Benefit |
|---|---|
| SDEX | Built-in decentralized orderbooks — less infrastructure |
| Soroban | Scalable Rust/WASM smart contracts |
| Atomic Swaps | Native fraud-free settlement |
| Path Payments | Multi-asset routing |
| Low Fees | Microtransactions viable |
| Fast Settlement | ~5 second finality |

---

## Key Challenges & Solutions

| Challenge | Solution |
|---|---|
| Fake assets | Verified collections |
| Wash trading | AI detection |
| Metadata hosting | IPFS |
| Liquidity | SDEX + AMMs |
| Scalability | Stellar low fees + Soroban parallelization |

---

## Resources

- [Stellar Developer Docs](https://developers.stellar.org)
- [Soroban Examples Repository](https://github.com/stellar/soroban-examples)
- [Soroban React Atomic Swap Demo](https://github.com/stellar/soroban-react-atomic-swap)
- [Soroswap Docs](https://docs.soroswap.finance)

---

## Vision

> **Steam Market + OpenSea + GameFi DEX** — fully decentralized, fraud-resistant, cross-game interoperable, and scalable on Stellar.
