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
        }

        PromiseOrValue::Value(U128(0))
    }
}
