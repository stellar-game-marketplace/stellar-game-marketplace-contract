#![no_std]

mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env, String};
use storage::{DataKey, NFTMetadata};

#[contract]
pub struct NFTContract;

#[contractimpl]
impl NFTContract {
    /// Initialize with admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextId, &1u64);
    }

    /// Mint a new NFT to `to` with metadata
    pub fn mint(
        env: Env,
        to: Address,
        name: String,
        description: String,
        image_uri: String,
        game_id: String,
        rarity: String,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token_id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap();

        let metadata = NFTMetadata { name, description, image_uri, game_id, rarity };

        env.storage().persistent().set(&DataKey::Owner(token_id), &to);
        env.storage().persistent().set(&DataKey::Metadata(token_id), &metadata);
        env.storage().instance().set(&DataKey::NextId, &(token_id + 1));

        token_id
    }

    /// Transfer NFT from current owner to `to`
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u64) {
        from.require_auth();
        let owner: Address = Self::owner_of(env.clone(), token_id);
        if owner != from {
            panic!("not owner");
        }
        env.storage().persistent().set(&DataKey::Owner(token_id), &to);
        // clear any approval on transfer
        env.storage().persistent().remove(&DataKey::Approved(token_id));
    }

    /// Approve `spender` to transfer `token_id`
    pub fn approve(env: Env, owner: Address, spender: Address, token_id: u64) {
        owner.require_auth();
        let actual_owner: Address = Self::owner_of(env.clone(), token_id);
        if actual_owner != owner {
            panic!("not owner");
        }
        env.storage().persistent().set(&DataKey::Approved(token_id), &spender);
    }

    /// Transfer using an approval
    pub fn transfer_from(env: Env, spender: Address, to: Address, token_id: u64) {
        spender.require_auth();
        let approved: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Approved(token_id))
            .expect("no approval");
        if approved != spender {
            panic!("not approved");
        }
        env.storage().persistent().set(&DataKey::Owner(token_id), &to);
        env.storage().persistent().remove(&DataKey::Approved(token_id));
    }

    // --- Views ---

    pub fn owner_of(env: Env, token_id: u64) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Owner(token_id))
            .expect("token does not exist")
    }

    pub fn metadata(env: Env, token_id: u64) -> NFTMetadata {
        env.storage()
            .persistent()
            .get(&DataKey::Metadata(token_id))
            .expect("token does not exist")
    }

    pub fn total_supply(env: Env) -> u64 {
        let next: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(1);
        next - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn setup() -> (Env, NFTContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, NFTContract);
        let client = NFTContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    fn s(env: &Env, v: &str) -> String {
        String::from_str(env, v)
    }

    #[test]
    fn test_mint_and_owner() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        let id = client.mint(
            &user,
            &s(&env, "Sword"),
            &s(&env, "A sword"),
            &s(&env, "ipfs://abc"),
            &s(&env, "game1"),
            &s(&env, "Legendary"),
        );
        assert_eq!(id, 1);
        assert_eq!(client.owner_of(&id), user);
        assert_eq!(client.total_supply(), 1);
    }

    #[test]
    fn test_transfer() {
        let (env, client, _) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let id = client.mint(
            &alice,
            &s(&env, "Shield"),
            &s(&env, "desc"),
            &s(&env, "ipfs://x"),
            &s(&env, "g1"),
            &s(&env, "Rare"),
        );
        client.transfer(&alice, &bob, &id);
        assert_eq!(client.owner_of(&id), bob);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let (env, client, _) = setup();
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);
        let id = client.mint(
            &alice,
            &s(&env, "Axe"),
            &s(&env, "desc"),
            &s(&env, "ipfs://y"),
            &s(&env, "g2"),
            &s(&env, "Common"),
        );
        client.approve(&alice, &spender, &id);
        client.transfer_from(&spender, &bob, &id);
        assert_eq!(client.owner_of(&id), bob);
    }
}
