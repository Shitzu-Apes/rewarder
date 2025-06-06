use near_sdk::{near, AccountId};

use crate::{Contract, ContractExt, FarmConfig};

#[near]
impl Contract {
    pub fn get_user_checkpoint(&self, account_id: AccountId) -> Option<u64> {
        self.checkpoint.get(&account_id)
    }

    pub fn get_farm_configs(&self) -> (FarmConfig, FarmConfig, FarmConfig) {
        (self.xref.clone(), self.shitzu.clone(), self.lp.clone())
    }
}
