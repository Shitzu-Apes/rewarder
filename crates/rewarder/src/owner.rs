use near_sdk::{env, near, require, Promise};

use crate::{Contract, ContractExt};

#[near]
impl Contract {
    pub fn migrate(&mut self) {
        // empty for now
    }

    pub fn upgrade(&self) -> Promise {
        self.require_owner();

        let code = env::input().expect("Error: No input").to_vec();

        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .as_return()
    }
}

impl Contract {
    fn require_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner,
            "Only owner can call this function"
        );
    }
}
