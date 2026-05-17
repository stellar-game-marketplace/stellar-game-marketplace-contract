#![no_std]

mod storage;

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use storage::{Bid, DataKey, Listing};

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    pub fn initialize(env: Env, admin: Address, fee_wallet: Address, fee_rate_bps: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeWallet, &fee_wallet);
        env.storage().instance().set(&DataKey::FeeRate, &fee_rate_bps);
        env.storage().instance().set(&DataKey::NextListingId, &1u64);
        env.storage().instance().set(&DataKey::NextBidId, &1u64);
    }

    /// Create a listing. Seller must have approved this contract on the NFT contract.
    pub fn list(
        env: Env,
        seller: Address,
        nft_contract: Address,
        token_id: u64,
        price: i128,
        expires_at: u64,
    ) -> u64 {
        seller.require_auth();

        let listing_id: u64 = env.storage().instance().get(&DataKey::NextListingId).unwrap();
        let listing = Listing {
            seller,
            nft_contract,
            token_id,
            price,
            expires_at,
            active: true,
        };
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
        env.storage().instance().set(&DataKey::NextListingId, &(listing_id + 1));
        listing_id
    }

    /// Cancel a listing
    pub fn cancel_listing(env: Env, seller: Address, listing_id: u64) {
        seller.require_auth();
        let mut listing: Listing = Self::get_listing(env.clone(), listing_id);
        if listing.seller != seller {
            panic!("not seller");
        }
        listing.active = false;
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
    }

    /// Buy a listed NFT. Buyer pays price in XLM (native token).
    /// Transfers fee to fee_wallet, remainder to seller.
    /// Caller must also invoke NFT transfer_from separately (or use atomic-swap-contract).
    pub fn buy(env: Env, buyer: Address, listing_id: u64, payment_token: Address) {
        buyer.require_auth();

        let mut listing: Listing = Self::get_listing(env.clone(), listing_id);
        if !listing.active {
            panic!("listing not active");
        }
        if env.ledger().timestamp() > listing.expires_at {
            panic!("listing expired");
        }

        let fee_rate: u32 = env.storage().instance().get(&DataKey::FeeRate).unwrap();
        let fee_wallet: Address = env.storage().instance().get(&DataKey::FeeWallet).unwrap();

        let fee = (listing.price * fee_rate as i128) / 10_000;
        let seller_amount = listing.price - fee;

        let token = token::Client::new(&env, &payment_token);
        token.transfer(&buyer, &fee_wallet, &fee);
        token.transfer(&buyer, &listing.seller, &seller_amount);

        listing.active = false;
        env.storage().persistent().set(&DataKey::Listing(listing_id), &listing);
    }

    /// Place a bid on a listing
    pub fn place_bid(
        env: Env,
        bidder: Address,
        listing_id: u64,
        amount: i128,
        expires_at: u64,
    ) -> u64 {
        bidder.require_auth();

        let listing: Listing = Self::get_listing(env.clone(), listing_id);
        if !listing.active {
            panic!("listing not active");
        }

        let bid_id: u64 = env.storage().instance().get(&DataKey::NextBidId).unwrap();
        let bid = Bid { bidder, listing_id, amount, expires_at };
        env.storage().persistent().set(&DataKey::Bid(bid_id), &bid);
        env.storage().instance().set(&DataKey::NextBidId, &(bid_id + 1));
        bid_id
    }

    // --- Views ---

    pub fn get_listing(env: Env, listing_id: u64) -> Listing {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("listing not found")
    }

    pub fn get_bid(env: Env, bid_id: u64) -> Bid {
        env.storage()
            .persistent()
            .get(&DataKey::Bid(bid_id))
            .expect("bid not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, MarketplaceContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, MarketplaceContract);
        let client = MarketplaceContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let fee_wallet = Address::generate(&env);
        client.initialize(&admin, &fee_wallet, &200u32); // 2% fee
        (env, client)
    }

    #[test]
    fn test_list_and_get() {
        let (env, client) = setup();
        let seller = Address::generate(&env);
        let nft = Address::generate(&env);
        let listing_id = client.list(&seller, &nft, &1u64, &1000i128, &9999999u64);
        let listing = client.get_listing(&listing_id);
        assert_eq!(listing.seller, seller);
        assert_eq!(listing.price, 1000);
        assert!(listing.active);
    }

    #[test]
    fn test_cancel_listing() {
        let (env, client) = setup();
        let seller = Address::generate(&env);
        let nft = Address::generate(&env);
        let id = client.list(&seller, &nft, &2u64, &500i128, &9999999u64);
        client.cancel_listing(&seller, &id);
        let listing = client.get_listing(&id);
        assert!(!listing.active);
    }

    #[test]
    fn test_place_bid() {
        let (env, client) = setup();
        let seller = Address::generate(&env);
        let bidder = Address::generate(&env);
        let nft = Address::generate(&env);
        let listing_id = client.list(&seller, &nft, &3u64, &800i128, &9999999u64);
        let bid_id = client.place_bid(&bidder, &listing_id, &750i128, &9999999u64);
        let bid = client.get_bid(&bid_id);
        assert_eq!(bid.bidder, bidder);
        assert_eq!(bid.amount, 750);
    }
}
