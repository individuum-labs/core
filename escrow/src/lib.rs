#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use stylus_sdk::{
    alloy_primitives::{address, Address, FixedBytes, U256},
    contract, msg,
    prelude::*,
    stylus_proc::{entrypoint, external, sol_interface},
};

sol_storage! {
    #[entrypoint]
    pub struct RewardPool {
        uint reward_rate;
        uint total_funds;
        string required_string;
        mapping(uint => uint) contributions;
        bool initialized;
        address verifier
    }
}

const USDC: Address = address!("75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");

sol_interface! {
    interface IERC20 {
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

    interface Verifier {
        function verify_proof(uint, uint, string calldata, bytes32, bytes32, bytes1) pure returns (bool);
    }
}

#[external]
impl RewardPool {
    pub fn initialize_reward_pool(
        &mut self,
        initial_funds: U256,
        reward_rate: U256,
        required_string: String,
        verifier: Address,
    ) {
        assert!(!self.initialized.get());

        self.reward_rate.set(reward_rate);
        self.total_funds.set(initial_funds);
        self.required_string.set_str(required_string);
        self.initialized.set(true);
        self.verifier.set(verifier);

        IERC20::new(USDC)
            .transfer_from(self, msg::sender(), contract::address(), initial_funds)
            .unwrap();
    }

    pub fn process_post(
        &mut self,
        favorite_count: U256,
        post_id: U256,
        full_text: String,
        signature_a: FixedBytes<32>,
        signature_b: FixedBytes<32>,
        recid: FixedBytes<1>,
    ) {
        assert!(Verifier::new(self.verifier.get())
            .verify_proof(
                &*self,
                favorite_count,
                post_id,
                full_text.clone(),
                signature_a,
                signature_b,
                recid,
            )
            .unwrap());

        assert!(full_text.contains(&self.required_string.get_string()));

        let claimed_likes = self.contributions.get(post_id);
        let likes_to_claim = favorite_count - claimed_likes;

        self.contributions.insert(post_id, favorite_count);

        let amount = likes_to_claim * self.reward_rate.get();

        IERC20::new(USDC)
            .transfer(self, msg::sender(), amount)
            .unwrap();
    }
}
