use crate::{Contract, ContractExt};
use near_sdk::{near, AccountId};

#[near]
impl Contract {
    pub fn get_leaderboard(&self, limit: u64) -> Vec<(u128, Vec<AccountId>)> {
        self.ranking
            .iter()
            .rev()
            .take(limit as usize)
            .map(|x| (*x.0, x.1.clone()))
            .collect()
    }
}
