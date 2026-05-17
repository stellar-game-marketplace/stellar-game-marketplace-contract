use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct Listing {
    pub seller: Address,
    pub nft_contract: Address,
    pub token_id: u64,
    pub price: i128,       // in stroops
    pub expires_at: u64,   // ledger timestamp
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Bid {
    pub bidder: Address,
    pub listing_id: u64,
    pub amount: i128,
    pub expires_at: u64,
}

#[contracttype]
pub enum DataKey {
    Listing(u64),
    Bid(u64),
    NextListingId,
    NextBidId,
    FeeRate,   // basis points e.g. 200 = 2%
    FeeWallet,
    Admin,
}
