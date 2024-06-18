use near_sdk::{near, AccountId};

use crate::{Contract, ContractExt};

#[near]
impl Contract {
    pub fn get_user_checkpoint(&self, account_id: AccountId) -> Option<u64> {
        self.checkpoint.get(&account_id)
    }
}
