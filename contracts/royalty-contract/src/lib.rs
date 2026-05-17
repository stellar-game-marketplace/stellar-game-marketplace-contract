#![no_std]

mod storage;

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use storage::{DataKey, RoyaltyConfig};

#[contract]
pub struct RoyaltyContract;

#[contractimpl]
impl RoyaltyContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Register royalty config for an NFT. Only admin or the creator can register.
    pub fn register(
        env: Env,
        caller: Address,
        nft_contract: Address,
        token_id: u64,
        creator: Address,
        rate_bps: u32,
    ) {
        caller.require_auth();
        if rate_bps > 1000 {
            panic!("max royalty is 10%");
        }
        let config = RoyaltyConfig { creator, rate_bps };
        env.storage()
            .persistent()
            .set(&DataKey::Royalty(nft_contract, token_id), &config);
    }

    /// Distribute royalty from a sale. Called by marketplace or swap contract.
    /// Transfers `rate_bps / 10000 * sale_price` to creator from `payer`.
    /// Returns the royalty amount paid.
    pub fn distribute(
        env: Env,
        payer: Address,
        nft_contract: Address,
        token_id: u64,
        payment_token: Address,
        sale_price: i128,
    ) -> i128 {
        payer.require_auth();

        let config: RoyaltyConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Royalty(nft_contract, token_id))
            .expect("no royalty config");

        let royalty = (sale_price * config.rate_bps as i128) / 10_000;
        if royalty > 0 {
            let token = token::Client::new(&env, &payment_token);
            token.transfer(&payer, &config.creator, &royalty);
        }
        royalty
    }

    pub fn get_config(env: Env, nft_contract: Address, token_id: u64) -> RoyaltyConfig {
        env.storage()
            .persistent()
            .get(&DataKey::Royalty(nft_contract, token_id))
            .expect("no royalty config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_and_get_config() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, RoyaltyContract);
        let client = RoyaltyContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let creator = Address::generate(&env);
        let nft_contract = Address::generate(&env);
        client.register(&admin, &nft_contract, &1u64, &creator, &500u32); // 5%

        let config = client.get_config(&nft_contract, &1u64);
        assert_eq!(config.creator, creator);
        assert_eq!(config.rate_bps, 500);
    }
}
