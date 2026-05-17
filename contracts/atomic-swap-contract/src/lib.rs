#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, IntoVal};

/// Represents a pending swap locked on-chain
#[contracttype]
#[derive(Clone)]
pub struct SwapOffer {
    pub buyer: Address,
    pub seller: Address,
    pub nft_contract: Address,
    pub token_id: u64,
    pub payment_token: Address,
    pub payment_amount: i128,
    pub expires_at: u64,
    pub executed: bool,
}

#[contracttype]
pub enum DataKey {
    Offer(u64),
    NextOfferId,
}

#[contract]
pub struct AtomicSwapContract;

#[contractimpl]
impl AtomicSwapContract {
    pub fn initialize(env: Env) {
        env.storage().instance().set(&DataKey::NextOfferId, &1u64);
    }

    /// Buyer creates a swap offer, locking payment into this contract.
    /// Buyer must have approved this contract on the payment token.
    pub fn create_offer(
        env: Env,
        buyer: Address,
        seller: Address,
        nft_contract: Address,
        token_id: u64,
        payment_token: Address,
        payment_amount: i128,
        expires_at: u64,
    ) -> u64 {
        buyer.require_auth();

        // Lock buyer's payment into this contract
        let token = token::Client::new(&env, &payment_token);
        token.transfer(&buyer, &env.current_contract_address(), &payment_amount);

        let offer_id: u64 = env.storage().instance().get(&DataKey::NextOfferId).unwrap();
        let offer = SwapOffer {
            buyer,
            seller,
            nft_contract,
            token_id,
            payment_token,
            payment_amount,
            expires_at,
            executed: false,
        };
        env.storage().persistent().set(&DataKey::Offer(offer_id), &offer);
        env.storage().instance().set(&DataKey::NextOfferId, &(offer_id + 1));
        offer_id
    }

    /// Seller accepts the offer. Seller must have approved this contract on the NFT contract.
    /// Executes atomically: NFT → buyer, payment → seller.
    pub fn execute(env: Env, seller: Address, offer_id: u64) {
        seller.require_auth();

        let mut offer: SwapOffer = Self::get_offer(env.clone(), offer_id);

        if offer.executed {
            panic!("already executed");
        }
        if offer.seller != seller {
            panic!("wrong seller");
        }
        if env.ledger().timestamp() > offer.expires_at {
            panic!("offer expired");
        }

        // Transfer NFT from seller to buyer via cross-contract call to NFT contract's transfer_from.
        // Seller must have approved this contract on the NFT contract beforehand.
        let _: () = env.invoke_contract(
            &offer.nft_contract,
            &soroban_sdk::Symbol::new(&env, "transfer_from"),
            soroban_sdk::vec![
                &env,
                env.current_contract_address().into_val(&env),
                offer.buyer.clone().into_val(&env),
                offer.token_id.into_val(&env),
            ],
        );

        // Release locked payment to seller
        let token = token::Client::new(&env, &offer.payment_token);
        token.transfer(&env.current_contract_address(), &seller, &offer.payment_amount);

        offer.executed = true;
        env.storage().persistent().set(&DataKey::Offer(offer_id), &offer);
    }

    /// Buyer cancels and reclaims payment after expiry
    pub fn cancel(env: Env, buyer: Address, offer_id: u64) {
        buyer.require_auth();

        let mut offer: SwapOffer = Self::get_offer(env.clone(), offer_id);

        if offer.executed {
            panic!("already executed");
        }
        if offer.buyer != buyer {
            panic!("not buyer");
        }
        if env.ledger().timestamp() <= offer.expires_at {
            panic!("offer not yet expired");
        }

        // Refund buyer
        let token = token::Client::new(&env, &offer.payment_token);
        token.transfer(&env.current_contract_address(), &buyer, &offer.payment_amount);

        offer.executed = true; // mark consumed
        env.storage().persistent().set(&DataKey::Offer(offer_id), &offer);
    }

    pub fn get_offer(env: Env, offer_id: u64) -> SwapOffer {
        env.storage()
            .persistent()
            .get(&DataKey::Offer(offer_id))
            .expect("offer not found")
    }
}
