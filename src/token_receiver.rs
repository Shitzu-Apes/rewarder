use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::{core::NonFungibleTokenReceiver, TokenId},
};
use near_sdk::{env, json_types::U128, log, near, AccountId, PromiseOrValue};

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

        if let Some(token_id) = self.primary_nft.get(&sender_id) {
            log!(
                "{} sent {} tokens with message: {}",
                sender_id,
                amount.0,
                msg
            );

            let donation_amount = self.donation_amounts.get(token_id).unwrap_or_else(|| {
                self.donor_count += 1;
                &0
            });

            self.donation_amounts
                .set(token_id.clone(), Some(donation_amount + amount.0));

            let donation_amount = self.donation_amounts.get(token_id).unwrap();

            let mut donor_ranking = self
                .donor_ranking
                .get(&donation_amount)
                .unwrap_or(&Vec::new())
                .clone();

            donor_ranking.push(token_id.clone());

            self.donor_ranking
                .insert(donation_amount.clone(), donor_ranking);
        }
        self.total_received += amount.0;

        PromiseOrValue::Value(U128(0))
    }
}

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

        log!(
            "{} transferred token {} from {} with message: {}",
            sender_id,
            token_id,
            previous_owner_id,
            msg
        );

        if let Some(old_primary_nft) = self.primary_nft.insert(previous_owner_id.clone(), token_id)
        {
            // Set this old primary NFT to be claimable by the previous owner
            self.unstaked_nft.insert(old_primary_nft, previous_owner_id);
        } else {
            self.participant_count += 1;
        }

        PromiseOrValue::Value(false)
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    // #[test]
    // fn test_ft_on_transfer() {
    //     let reward_token: AccountId = "reward_token".parse().unwrap();

    //     let mut contract = Contract::new(reward_token, "nft".parse().unwrap());
    //     let context = VMContextBuilder::new()
    //         .predecessor_account_id("not_reward_token".parse().unwrap())
    //         .attached_deposit(NearToken::from_yoctonear(1))
    //         .build();
    //     testing_env!(context.clone());

    //     // Test when sender is not the reward token
    //     contract.ft_on_transfer(accounts(1), 5.into(), "".into());
    //     assert_eq!(contract.total_received, 0);
    //     assert_eq!(contract.donor_count, 0);
    //     assert_eq!(contract.donation_amounts.get(&accounts(1)), None);
    //     assert_eq!(contract.donor_ranking.get(&0), None);

    //     // Test when sender is the reward token
    //     let context = VMContextBuilder::new()
    //         .predecessor_account_id(reward_token.clone())
    //         .attached_deposit(NearToken::from_yoctonear(1))
    //         .build();
    //     contract.ft_on_transfer(accounts(0), 5.into(), "Test".to_string());
    //     assert_eq!(contract.total_received, 5);
    //     assert_eq!(contract.donor_count, 1);
    //     assert_eq!(contract.donation_amounts.get(&accounts(0)), Some(5));
    //     assert_eq!(
    //         contract.donor_ranking.get(&5),
    //         Some(vec![accounts(0).to_string()])
    //     );
    //     assert_eq!(contract.donor_ranking.get(&0), None);
    // }

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

        assert_eq!(contract.primary_nft.get(&alice), Some(&"1".to_string()));
        assert_eq!(contract.participant_count, 1);
    }

    #[test]
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
        contract.nft_on_transfer(accounts(0), alice.clone(), "2".to_string(), "".to_string());

        assert_eq!(contract.primary_nft.get(&alice), Some(&"2".to_string()));
        assert_eq!(
            contract.unstaked_nft.get(&"1".to_string()),
            Some(&alice.clone())
        );
    }
}
