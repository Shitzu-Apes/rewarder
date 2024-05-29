use crate::{Contract, ContractExt};
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{json_types::U128, near, AccountId};

#[near]
impl Contract {
    pub fn get_leaderboard(&self, limit: Option<u64>) -> Vec<(U128, Vec<TokenId>)> {
        let limit = limit.unwrap_or(10);
        self.ranking
            .iter()
            .rev()
            .take(limit as usize)
            .map(|x| (U128(*x.0), x.1.clone()))
            .collect()
    }

    pub fn primary_nft_of(&self, account_id: AccountId) -> Option<TokenId> {
        self.primary_nft.get(&account_id).cloned()
    }
}
