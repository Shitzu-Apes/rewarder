use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::{core::NonFungibleTokenReceiver, TokenId},
};
use near_sdk::{env, json_types::U128, log, near, AccountId, PromiseOrValue};
use primitive_types::U256;

use crate::{Contract, ContractExt};

#[near]
impl FungibleTokenReceiver for Contract {
    #[payable]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(
            env::predecessor_account_id() == self.reward_token,
            "Only receive tokens from reward token"
        );

        if let Some(token_id) = self.account_to_token_id.get(&sender_id) {
            log!(
                "{} sent {} tokens with message: {}, for NFT {}",
                sender_id,
                amount.0,
                msg,
                token_id
            );

            self.internal_record_score(token_id.clone(), amount.0 * 4);
            self.total_donation =
                (U256::from(self.total_donation) + U256::from(amount.0)).as_u128();
        }

        PromiseOrValue::Value(U128(0))
    }
}

#[near]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert!(
            env::predecessor_account_id() == self.nft,
            "Only receive tokens from NFT contract"
        );

        assert!(
            self.account_to_token_id.get(&previous_owner_id).is_none(),
            "User already has a primary NFT"
        );

        log!(
            "{} transferred token {} from {} with message: {}",
            sender_id,
            token_id,
            previous_owner_id,
            msg
        );

        self.on_stake_changed(previous_owner_id.clone(), Some(token_id.clone()));

        PromiseOrValue::Value(false)
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, NearToken};

    use super::*;

    #[test]
    fn test_ft_on_transfer_with_nft_quadruple_score() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let alice = accounts(1);

        // Alice stakes NFT 1
        let mut contract = Contract::new(reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context.clone());

        testing_env!(context.clone());
        contract.nft_on_transfer(accounts(0), alice.clone(), "1".to_string(), "".to_string());

        let context = VMContextBuilder::new()
            .predecessor_account_id(reward_token.clone())
            .attached_deposit(NearToken::from_yoctonear(1))
            .build();

        let amount = 1000 * 10_u128.pow(18);
        testing_env!(context.clone());
        contract.ft_on_transfer(alice.clone(), U128(amount), "".to_string());

        assert_eq!(contract.total_donation, amount);

        let score = contract.score_of("1".to_string());
        assert_eq!(score.0, &amount * 4);
    }

    #[test]
    #[should_panic]
    fn test_nft_on_transfer_incorrect_nft() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();

        let mut contract = Contract::new(reward_token, nft);
        let context = VMContextBuilder::new()
            .predecessor_account_id("not_nft".parse().unwrap())
            .build();
        testing_env!(context.clone());

        // Test when sender is not the NFT contract
        testing_env!(context.clone());
        contract.nft_on_transfer(
            accounts(1),
            accounts(2),
            "token".to_string(),
            "".to_string(),
        );
    }

    #[test]
    fn test_nft_on_transfer_record_correctly() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context.clone());

        // Test when sender is not the NFT contract
        testing_env!(context.clone());
        contract.nft_on_transfer(accounts(0), alice.clone(), "1".to_string(), "".to_string());

        assert_eq!(
            contract.account_to_token_id.get(&alice),
            Some(&"1".to_string())
        );
        assert_eq!(contract.total_nft_staked, 1);
    }

    #[test]
    #[should_panic]
    fn test_nft_on_transfer_switch_primary_nft() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context.clone());

        // Test when sender is not the NFT contract
        testing_env!(context.clone());
        contract.nft_on_transfer(accounts(0), alice.clone(), "1".to_string(), "".to_string());

        // Need to unstake before staking again
        contract.nft_on_transfer(accounts(0), alice.clone(), "2".to_string(), "".to_string());
    }
}
