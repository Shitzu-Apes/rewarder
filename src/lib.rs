mod token_receiver;
mod view;

// Find all our documentation at https://docs.near.org
use near_contract_standards::{fungible_token::core::ext_ft_core, non_fungible_token::TokenId};
use near_sdk::{
    log, near,
    store::{LookupMap, TreeMap},
    AccountId, NearToken,
};

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    reward_token: AccountId,
    nft: AccountId,

    primary_nft: LookupMap<AccountId, TokenId>,
    unstaked_nft: LookupMap<TokenId, AccountId>,

    total_distribute: u128,
    scores: LookupMap<TokenId, u128>,
    ranking: TreeMap<u128, Vec<TokenId>>,
    participant_count: u128,

    total_received: u128,
    donation_amounts: LookupMap<TokenId, u128>,
    donor_ranking: TreeMap<u128, Vec<TokenId>>,
    donor_count: u128,
}

#[near]
impl Contract {
    #[init]
    pub fn new(reward_token: AccountId, nft: AccountId) -> Self {
        Self {
            reward_token,
            nft,
            unstaked_nft: LookupMap::new(b"u".to_vec()),

            primary_nft: LookupMap::new(b"p".to_vec()),

            total_distribute: 0,
            ranking: TreeMap::new(b"r".to_vec()),
            scores: LookupMap::new(b"s".to_vec()),
            participant_count: 0,

            total_received: 0,
            donation_amounts: LookupMap::new(b"d".to_vec()),
            donor_ranking: TreeMap::new(b"dr".to_vec()),
            donor_count: 0,
        }
    }

    #[private]
    pub fn send_rewards(&mut self, account_id: AccountId, amount: u128) {
        // Check
        assert!(
            self.total_distribute + amount <= self.total_received,
            "Not enough funds"
        );

        let mut amount = amount;

        if let Some(primary_nft) = self.primary_nft.get(&account_id) {
            amount *= 2;

            // Effect
            self.total_distribute += amount;
            let new_score = self.scores.get(primary_nft).unwrap_or(&0) + amount;
            self.scores.set(primary_nft.clone(), Some(new_score));

            let mut ranking = self.ranking.get(&new_score).unwrap_or(&Vec::new()).clone();
            ranking.push(primary_nft.clone());
            self.ranking.insert(new_score, ranking);
        }

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
        let mut contract = Contract::new(
            "token.0xshitzu.near".parse().unwrap(),
            "shitzu.bodega-lab.near".parse().unwrap(),
        );
        let alice_id: AccountId = "alice.near".parse().unwrap();
        let bob_id: AccountId = "bob.near".parse().unwrap();
        let charlie_id: AccountId = "charlie.near".parse().unwrap();
        let dan_id: AccountId = "dan.near".parse().unwrap();
        let amount = 100;
    }
}
