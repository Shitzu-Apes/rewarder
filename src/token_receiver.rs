use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
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
        if env::predecessor_account_id() == self.reward_token {
            log!(
                "{} sent {} tokens with message: {}",
                sender_id,
                amount.0,
                msg
            );
            self.total_received += amount.0;

            let donation_amount = self.donation_amounts.get(&sender_id).unwrap_or_else(|| {
                self.donor_count += 1;
                &0
            });

            self.donation_amounts
                .set(sender_id.clone(), Some(donation_amount + amount.0));

            let donation_amount = self.donation_amounts.get(&sender_id).unwrap();

            let mut donor_ranking = self
                .donor_ranking
                .get(&donation_amount)
                .unwrap_or(&Vec::new())
                .clone();

            donor_ranking.push(sender_id.clone());

            self.donor_ranking
                .insert(donation_amount.clone(), donor_ranking);

            let rank: u128 = self
                .donor_ranking
                .iter()
                .rev()
                .position(|x| x.1.contains(&sender_id))
                .unwrap() as u128;

            log!("{} is ranked {}", sender_id, rank);
        }

        PromiseOrValue::Value(U128(0))
    }
}
