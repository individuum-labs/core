#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::alloy_primitives::U256;
    use stylus_sdk::msg;
    use stylus_sdk::evm;
    use stylus_sdk::block::number as block_number;
    use stylus_sdk::{alloy_primitives::Address, alloy_sol_types::SolEvent};

    #[test]
    fn test_initialize_reward_pool() {
        let mut reward_pool = RewardPool::new();
        let initial_funds = U256::from(1000);
        let reward_rate = U256::from(10);
        let deadline = U256::from(block_number() + 100);

        reward_pool.initialize_reward_pool(initial_funds, reward_rate, deadline);

        assert_eq!(reward_pool.total_funds, initial_funds);
        assert_eq!(reward_pool.reward_rate, reward_rate);
        assert_eq!(reward_pool.deadline, deadline);
        assert_eq!(reward_pool.contributors.get(&msg::sender()).unwrap(), &initial_funds);
    }

    #[test]
    fn test_fund_reward_pool() {
        let mut reward_pool = RewardPool::new();
        let initial_funds = U256::from(1000);
        let reward_rate = U256::from(10);
        let deadline = U256::from(block_number() + 100);

        reward_pool.initialize_reward_pool(initial_funds, reward_rate, deadline);

        let additional_funds = U256::from(500);
        reward_pool.fund_reward_pool(additional_funds);

        assert_eq!(reward_pool.total_funds, initial_funds + additional_funds);
        assert_eq!(reward_pool.contributors.get(&msg::sender()).unwrap(), &(initial_funds + additional_funds));
    }

    #[test]
    fn test_verify_likes() {
        let reward_pool = RewardPool::new();
        let user = Address::from([0u8; 20]);
        let post_id = String::from("post123");

        let verified_likes = reward_pool.verify_likes(user, post_id);

        assert_eq!(verified_likes, U256::from(100)); // Mock value
    }

    #[test]
    fn test_distribute_rewards() {
        let mut reward_pool = RewardPool::new();
        let initial_funds = U256::from(1000);
        let reward_rate = U256::from(10);
        let deadline = U256::from(block_number() + 100);

        reward_pool.initialize_reward_pool(initial_funds, reward_rate, deadline);

        let user_address = Address::from([1u8; 20]);
        let verified_likes = U256::from(50);

        let result = reward_pool.distribute_rewards(user_address, verified_likes);

        assert!(result.is_ok());
        assert_eq!(reward_pool.total_funds, initial_funds - reward_pool.calculate_reward(verified_likes));
    }
}