use crate::{Contract, ContractExt};
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{json_types::U128, near, AccountId};

#[near]
impl Contract {
    pub fn get_leaderboard(&self, limit: Option<u64>) -> Vec<(U128, Vec<(TokenId, AccountId)>)> {
        let limit = limit.unwrap_or(10);
        self.ranking
            .iter()
            .rev()
            .take(limit as usize)
            .map(|x| {
                let stakers_with_score =
                    x.1.iter()
                        .map(|token_id| {
                            let staker = self.staker_of(token_id.clone()).unwrap();
                            (token_id.clone(), staker)
                        })
                        .collect::<Vec<_>>();

                (U128(*x.0), stakers_with_score)
            })
            .collect()
    }

    pub fn primary_nft_of(&self, account_id: AccountId) -> Option<(TokenId, U128)> {
        if let Some(token_id) = self.account_to_token_id.get(&account_id) {
            let score = self.scores.get(token_id).unwrap_or(&0);
            Some((token_id.clone(), U128(*score)))
        } else {
            None
        }
    }

    pub fn staker_of(&self, token_id: TokenId) -> Option<AccountId> {
        self.token_id_to_account.get(&token_id).cloned()
    }

    pub fn score_of(&self, token_id: TokenId) -> U128 {
        U128(*self.scores.get(&token_id).unwrap_or(&0))
    }
}
