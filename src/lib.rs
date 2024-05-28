mod token_receiver;
mod view;

// Find all our documentation at https://docs.near.org
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{
    log, near,
    store::{LookupMap, TreeMap},
    AccountId, NearToken,
};

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    reward_token: AccountId,
    total_out: u128,
    total_received: u128,
    ranking: TreeMap<u128, Vec<AccountId>>,
    score: LookupMap<AccountId, u128>,
    count: u128,
}

#[near]
impl Contract {
    #[init]
    pub fn new(reward_token: AccountId) -> Self {
        Self {
            reward_token,
            total_out: 0,
            total_received: 0,
            ranking: TreeMap::new(b"r".to_vec()),
            score: LookupMap::new(b"s".to_vec()),
            count: 0,
        }
    }

    #[private]
    pub fn send_rewards(&mut self, account_id: AccountId, amount: u128) {
        // Check
        assert!(
            self.total_out + amount <= self.total_received,
            "Not enough funds"
        );

        // Effect
        self.total_out += amount;
        self.total_received -= amount;
        let score = self.score.get(&account_id).unwrap_or(&0);
        self.score.set(account_id.clone(), Some(score + amount));
        let score = self.score.get(&account_id).unwrap();

        let mut ranking = self.ranking.get(&score).unwrap_or(&Vec::new()).clone();
        ranking.push(account_id.clone());
        self.ranking.insert(score.clone(), ranking);

        let rank: u128 = self
            .ranking
            .iter()
            .position(|x| x.0 == score)
            .unwrap_or(self.count as usize)
            .try_into()
            .unwrap();

        log!(
            "User {} received {} score, total score: {}, rank: {}",
            account_id,
            amount,
            score,
            rank
        );

        // Interaction
        ext_ft_core::ext(self.reward_token.clone())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(account_id.clone(), amount.into(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_rewards() {
        let mut contract = Contract::new("token.0xshitzu.near".parse().unwrap());
        let alice_id: AccountId = "alice.near".parse().unwrap();
        let bob_id: AccountId = "bob.near".parse().unwrap();
        let charlie_id: AccountId = "charlie.near".parse().unwrap();
        let dan_id: AccountId = "dan.near".parse().unwrap();
        let amount = 100;

        let rank = send_and_get_rank(&mut contract, alice_id.clone(), amount);
        assert_eq!(rank, 0);

        let rank = send_and_get_rank(&mut contract, bob_id.clone(), amount - 50);
        assert_eq!(rank, 0);
        assert_eq!(get_rank(&contract, alice_id.clone()), 1);

        let rank = send_and_get_rank(&mut contract, charlie_id.clone(), amount + 50);
        assert_eq!(get_rank(&contract, bob_id.clone()), 0);
        assert_eq!(get_rank(&contract, alice_id.clone()), 1);
        assert_eq!(rank, 2);

        let rank = send_and_get_rank(&mut contract, dan_id.clone(), amount + 50);
        assert_eq!(get_rank(&contract, bob_id.clone()), 0);
        assert_eq!(get_rank(&contract, alice_id.clone()), 1);
        assert_eq!(get_rank(&contract, charlie_id.clone()), 2);
        assert_eq!(rank, 2);

        let rank = send_and_get_rank(&mut contract, alice_id.clone(), amount);
        assert_eq!(get_rank(&contract, bob_id.clone()), 0);
        assert_eq!(get_rank(&contract, dan_id.clone()), 2);
        assert_eq!(get_rank(&contract, charlie_id.clone()), 2);
        assert_eq!(rank, 3);
    }

    fn send_and_get_rank(contract: &mut Contract, account_id: AccountId, amount: u128) -> u128 {
        contract.send_rewards(account_id.clone(), amount);
        let rank = get_rank(contract, account_id);
        rank as u128
    }

    fn get_rank(contract: &Contract, account_id: AccountId) -> u128 {
        let score = contract.score.get(&account_id).unwrap();
        let rank = contract.ranking.iter().position(|x| x.0 == score).unwrap();
        rank as u128
    }
}
