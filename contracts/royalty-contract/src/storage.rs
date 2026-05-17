use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct RoyaltyConfig {
    pub creator: Address,
    pub rate_bps: u32,  // basis points, e.g. 500 = 5%
}

#[contracttype]
pub enum DataKey {
    Royalty(Address, u64),  // (nft_contract, token_id) -> RoyaltyConfig
    Admin,
}
