use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    env, ext_contract, near,
    serde::{Deserialize, Serialize},
    AccountId, NearToken, Promise,
};

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

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    Stake,
    Unstake,
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
                    .on_stake_changed(owner, token_id.to_string(), Action::Unstake),
            )
    }

    #[private]
    pub fn on_stake_changed(&mut self, account_id: AccountId, token_id: TokenId, action: Action) {
        match action {
            Action::Stake => {
                self.account_to_token_id
                    .set(account_id.clone(), Some(token_id.clone()));

                self.token_id_to_account
                    .set(token_id.clone(), Some(account_id));
                self.total_nft_staked += 1;
            }
            Action::Unstake => {
                self.account_to_token_id.remove(&account_id);
                self.token_id_to_account.remove(&token_id);

                self.total_nft_staked -= 1;
            }
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
        contract.on_stake_changed(alice.clone(), "1".to_string(), Action::Unstake);

        assert_eq!(contract.account_to_token_id.get(&alice), None);
        assert_eq!(contract.total_nft_staked, 0);
    }

    #[test]
    fn test_query_staker_of_nft_correctly() {
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
            contract.primary_nft_of(alice.clone()),
            Some("1".to_string())
        );

        let staker = contract.token_id_to_account.get("1").unwrap();
        assert_eq!(staker, &alice);

        let staker = contract.staker_of("1".to_string());
        assert_eq!(staker, Some(alice));
    }
}
