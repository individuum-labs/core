#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use alloc::string::String;
use alloc::vec::Vec;
use k256::ecdsa::{self, RecoveryId};
use stylus_sdk::{
    alloy_primitives::{address, keccak256, Address, U256},
    alloy_sol_types::SolValue,
    block, contract, msg,
    prelude::*,
    stylus_proc::{entrypoint, external, sol_interface},
};

sol_storage! {
    #[entrypoint]
    pub struct RewardPool {
        address owner;
        uint reward_rate;
        uint total_funds;
        string required_string;
        mapping(uint => uint) contributions;
        bool initialized
    }
}

const USDC: Address = address!("75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");
const VERIFIER: Address = address!("49Dc27B14CfEe893e4AC9E47984Ca6B2Dccd7A2E");

sol_interface! {
    interface IERC20 {
        /**
         * @dev Emitted when `value` tokens are moved from one account (`from`) to
         * another (`to`).
         *
         * Note that `value` may be zero.
         */
        event Transfer(address indexed from, address indexed to, uint256 value);

        /**
         * @dev Emitted when the allowance of a `spender` for an `owner` is set by
         * a call to {approve}. `value` is the new allowance.
         */
        event Approval(address indexed owner, address indexed spender, uint256 value);

        /**
         * @dev Returns the value of tokens in existence.
         */
        function totalSupply() external view returns (uint256);

        /**
         * @dev Returns the value of tokens owned by `account`.
         */
        function balanceOf(address account) external view returns (uint256);

        /**
         * @dev Moves a `value` amount of tokens from the caller's account to `to`.
         *
         * Returns a boolean value indicating whether the operation succeeded.
         *
         * Emits a {Transfer} event.
         */
        function transfer(address to, uint256 value) external returns (bool);

        /**
         * @dev Returns the remaining number of tokens that `spender` will be
         * allowed to spend on behalf of `owner` through {transferFrom}. This is
         * zero by default.
         *
         * This value changes when {approve} or {transferFrom} are called.
         */
        function allowance(address owner, address spender) external view returns (uint256);

        /**
         * @dev Sets a `value` amount of tokens as the allowance of `spender` over the
         * caller's tokens.
         *
         * Returns a boolean value indicating whether the operation succeeded.
         *
         * IMPORTANT: Beware that changing an allowance with this method brings the risk
         * that someone may use both the old and the new allowance by unfortunate
         * transaction ordering. One possible solution to mitigate this race
         * condition is to first reduce the spender's allowance to 0 and set the
         * desired value afterwards:
         * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
         *
         * Emits an {Approval} event.
         */
        function approve(address spender, uint256 value) external returns (bool);

        /**
         * @dev Moves a `value` amount of tokens from `from` to `to` using the
         * allowance mechanism. `value` is then deducted from the caller's
         * allowance.
         *
         * Returns a boolean value indicating whether the operation succeeded.
         *
         * Emits a {Transfer} event.
         */
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

// sol! {
//     #[derive(Debug, Default, PartialEq)]
//     event Initialized(address indexed owner, uint256 initialFunds, uint256 rewardRate, uint256 deadline);
//     event Funded(address indexed contributor, uint256 amount);
//     event LikesVerified(address indexed user, uint256 verifiedLikes);
//     event RewardDistributed(address indexed user, uint256 amount);
//     event SettingsUpdated(uint256 new_reward_rate, uint256 new_deadline);
//     event Refunded(address indexed contributor, uint256 amount);
//     event PostSubmitted(address indexed user, string post_link, uint256 claimed_likes);
// }

#[external]
impl RewardPool {
    pub fn initialize_reward_pool(
        &mut self,
        initial_funds: U256,
        reward_rate: U256,
        deadline: U256,
        required_string: String,
    ) {
        assert!(
            msg::sender() == self.owner.get(),
            "Only owner can initialize"
        );
        assert!(!self.initialized.get());
        assert!(self.total_funds.get() == U256::ZERO, "Already initialized");
        assert!(
            initial_funds > U256::ZERO,
            "Initial funds must be greater than zero"
        );
        assert!(
            reward_rate > U256::ZERO,
            "Reward rate must be greater than zero"
        );
        assert!(
            deadline > U256::from(block::number()),
            "Deadline must be in the future"
        );

        self.reward_rate.set(reward_rate);
        self.total_funds.set(initial_funds);
        self.required_string.set_str(required_string);
        self.initialized.set(true);

        IERC20::new(USDC)
            .transfer_from(self, msg::sender(), contract::address(), initial_funds)
            .unwrap();

        // Trigger an event to log the initialization
        // evm::log(Initialized {
        //     owner: msg::sender(),
        //     initialFunds: initial_funds,
        //     rewardRate: reward_rate,
        //     deadline: deadline,
        // });
    }

    pub fn process_post(
        &mut self,
        favorite_count: U256,
        post_id: U256,
        full_text: String,
        signature: Vec<u8>,
        recid: u8,
    ) {
        let raw_data = (favorite_count, post_id, &full_text).abi_encode();
        let recovered = Address::from_public_key(
            &k256::ecdsa::VerifyingKey::recover_from_prehash(
                keccak256(raw_data).as_slice(),
                &ecdsa::Signature::from_slice(signature.as_slice()).unwrap(),
                RecoveryId::from_byte(recid).unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(recovered, VERIFIER);

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
