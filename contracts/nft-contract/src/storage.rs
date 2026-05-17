use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image_uri: String,  // ipfs://...
    pub game_id: String,
    pub rarity: String,
}

#[contracttype]
pub enum DataKey {
    Owner(u64),       // token_id -> Address
    Metadata(u64),    // token_id -> NFTMetadata
    Approved(u64),    // token_id -> Address
    NextId,
    Admin,
}
