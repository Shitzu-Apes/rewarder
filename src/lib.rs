mod nft;
mod token_receiver;
mod view;

// Find all our documentation at https://docs.near.org
use near_contract_standards::{fungible_token::core::ext_ft_core, non_fungible_token::TokenId};
use near_sdk::{
    near,
    store::{LookupMap, TreeMap},
    AccountId, NearToken,
};

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    reward_token: AccountId,
    nft: AccountId,

    primary_nft: LookupMap<AccountId, TokenId>,

    total_distribute: u128,
    scores: LookupMap<TokenId, u128>,
    ranking: TreeMap<u128, Vec<TokenId>>,
    participant_count: u128,

    total_dontation: u128,
    donation_amounts: LookupMap<TokenId, u128>,
    donor_ranking: TreeMap<u128, Vec<TokenId>>,
    donor_count: u128,
}

#[near]
impl Default for Contract {
    #[init]
    fn default() -> Self {
        panic!("No default initialization")
    }
}

#[near]
impl Contract {
    #[init]
    pub fn new(reward_token: AccountId, nft: AccountId) -> Self {
        Self {
            reward_token,
            nft,

            primary_nft: LookupMap::new(b"p".to_vec()),

            total_distribute: 0,
            ranking: TreeMap::new(b"r".to_vec()),
            scores: LookupMap::new(b"s".to_vec()),
            participant_count: 0,

            total_dontation: 0,
            donation_amounts: LookupMap::new(b"d".to_vec()),
            donor_ranking: TreeMap::new(b"dr".to_vec()),
            donor_count: 0,
        }
    }

    #[private]
    pub fn send_rewards(&mut self, account_id: AccountId, amount: u128) {
        let amount = self.internal_record_score(account_id.clone(), amount);

        self.total_distribute += amount;

        // Interaction
        ext_ft_core::ext(self.reward_token.clone())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(account_id.clone(), amount.into(), None);
    }
}

impl Contract {
    fn internal_record_score(&mut self, account_id: AccountId, amount: u128) -> u128 {
        if let Some(primary_nft) = self.primary_nft.get(&account_id) {
            let amount = amount * 2;
            let score = self.scores.get(primary_nft).unwrap_or(&0).clone();
            let new_score = score + amount;
            self.scores.set(primary_nft.clone(), Some(new_score));

            // remove from old ranking
            let mut ranking = self.ranking.get(&score).unwrap_or(&Vec::new()).clone();
            ranking.retain(|x| x != primary_nft);

            if ranking.is_empty() {
                self.ranking.remove(&score);
            } else {
                self.ranking.insert(score, ranking);
            }

            let mut ranking = self.ranking.get(&new_score).unwrap_or(&Vec::new()).clone();
            ranking.push(primary_nft.clone());
            self.ranking.insert(new_score, ranking);

            amount
        } else {
            amount
        }
    }
}

#[cfg(test)]
mod tests {
    use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
    use near_sdk::{
        test_utils::{accounts, VMContextBuilder},
        testing_env,
    };

    use super::*;

    #[test]
    fn test_double_reward_nft_staker() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();

        let mut contract = Contract::new(reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();

        testing_env!(context.clone());
        contract.nft_on_transfer(accounts(1), alice_id.clone(), "1".into(), "".into());
        contract.internal_record_score(alice_id, amount);

        assert_eq!(contract.scores.get("1".into()), Some(&(amount * 2)));
    }

    #[test]
    fn test_nft_ranking() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();

        let mut contract = Contract::new(reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let bob_id: AccountId = "bob.near".parse().unwrap();
        let charlie_id: AccountId = "charlie.near".parse().unwrap();
        let dan_id: AccountId = "dan.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);
        let fifty = 50 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();

        testing_env!(context.clone());

        // Alice stakes NFT 1 and receives the lowest score
        contract.nft_on_transfer(accounts(1), alice_id.clone(), "1".into(), "".into());
        contract.internal_record_score(alice_id.clone(), amount - fifty);

        // Bob stakes NFT 2 and receives the highest score
        contract.nft_on_transfer(accounts(1), bob_id.clone(), "2".into(), "".into());
        contract.internal_record_score(bob_id.clone(), amount + fifty);

        // Charlie stakes NFT 3 and receives the lowest score (same as Alice)
        contract.nft_on_transfer(accounts(1), charlie_id.clone(), "3".into(), "".into());
        contract.internal_record_score(charlie_id.clone(), amount - fifty);

        // Dan stakes NFT 4 and receives the middle score
        contract.nft_on_transfer(accounts(1), dan_id.clone(), "4".into(), "".into());
        contract.internal_record_score(dan_id.clone(), amount);

        // Expect the ranking to be [((amount + fifty) * 2, [2]), (amount * 2, [4]), ((amount - fifty) * 2, [1, 3])]
        let ranking = contract.get_leaderboard(Some(3));
        assert_eq!(
            ranking,
            vec![
                ((amount + fifty) * 2, vec!["2".to_string()]),
                (amount * 2, vec!["4".to_string()]),
                ((amount - fifty) * 2, vec!["1".to_string(), "3".to_string()])
            ]
        );

        // Dan scores another fifty points and should move to the same score as Bob
        contract.internal_record_score(dan_id, fifty);

        // Expect the ranking to be [((amount + fifty) * 2, [2, 4]), ((amount - fifty) * 2, [1, 3])]
        let ranking = contract.get_leaderboard(None);
        assert_eq!(
            ranking,
            vec![
                ((amount + fifty) * 2, vec!["2".to_string(), "4".to_string()]),
                ((amount - fifty) * 2, vec!["1".to_string(), "3".to_string()])
            ]
        );
    }
}
