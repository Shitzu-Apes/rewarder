use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{env, ext_contract, near, AccountId, NearToken, Promise};

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
    pub fn unstake(&mut self) -> Promise {
        let owner = env::predecessor_account_id();

        let token_id = self
            .account_to_token_id
            .get(&owner)
            .expect("No NFT found for the owner");

        nft::ext(self.nft.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .nft_transfer(
                owner.clone(),
                token_id.clone(),
                None,
                Some("Return old primary NFT".to_string()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_unused_gas_weight(1)
                    .on_stake_changed(owner, None),
            )
    }

    #[private]
    pub fn on_stake_changed(&mut self, account_id: AccountId, token_id: Option<TokenId>) {
        self.account_to_token_id
            .set(account_id.clone(), token_id.clone());

        if let Some(_) = token_id {
            self.total_nft_staked += 1;
        } else {
            self.total_nft_staked -= 1;
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
        assert_eq!(
            contract.account_to_token_id.get(&alice),
            Some(&"1".to_string())
        );
        assert_eq!(contract.total_nft_staked, 1);

        // Alice unstakes NFT 1
        let context = VMContextBuilder::new()
            .predecessor_account_id(alice.clone())
            .build();
        testing_env!(context);
        contract.unstake();
        contract.on_stake_changed(alice.clone(), None);

        assert_eq!(contract.account_to_token_id.get(&alice), None);
        assert_eq!(contract.total_nft_staked, 0);
    }
}
