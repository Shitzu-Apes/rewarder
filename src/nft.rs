use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{env, ext_contract, near, AccountId, NearToken};

use crate::{Contract, ContractExt};

#[ext_contract(nft)]
#[allow(dead_code)]
trait ShitzuNft {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
}

#[near]
impl Contract {
    pub fn unstake(&mut self) {
        let owner = env::predecessor_account_id();

        let token_id = self
            .primary_nft
            .remove(&owner)
            .expect("No NFT found for the owner");

        self.participant_count -= 1;

        self.internal_nft_transfer(owner.clone(), token_id.clone());
    }
}

impl Contract {
    pub(crate) fn internal_record_nft(
        &mut self,
        account_id: AccountId,
        token_id: TokenId,
    ) -> Option<TokenId> {
        self.primary_nft.insert(account_id.clone(), token_id)
    }

    pub(crate) fn internal_nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId) {
        nft::ext(self.nft.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .nft_transfer(
                receiver_id,
                token_id,
                None,
                Some("Return old primary NFT".to_string()),
            );
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
    fn test_unstake() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context);

        // Alice stakes NFT 1
        contract.nft_on_transfer(accounts(2), alice.clone(), "1".to_string(), "".to_string());
        assert_eq!(contract.primary_nft.get(&alice), Some(&"1".to_string()));
        assert_eq!(contract.participant_count, 1);

        // Alice unstakes NFT 1
        let context = VMContextBuilder::new()
            .predecessor_account_id(alice.clone())
            .build();
        testing_env!(context);
        contract.unstake();

        assert_eq!(contract.primary_nft.get(&alice), None);
        assert_eq!(contract.participant_count, 0);
    }
}
