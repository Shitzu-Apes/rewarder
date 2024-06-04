use crate::{event::RewarderEvent, Contract, ContractExt, GAS_FOR_NFT_TRANSFER};
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{env, ext_contract, near, AccountId, NearToken, Promise};

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
            .with_static_gas(GAS_FOR_NFT_TRANSFER)
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
                    .on_unstake(owner, token_id.to_string()),
            )
    }

    #[private]
    pub fn on_stake(&mut self, account_id: AccountId, token_id: TokenId) {
        self.account_to_token_id
            .set(account_id.clone(), Some(token_id.clone()));

        self.token_id_to_account
            .set(token_id.clone(), Some(account_id.clone()));
        self.total_nft_staked += 1;

        RewarderEvent::NFTStaked {
            account_id: account_id.clone(),
            token_id: token_id.clone(),
        }
        .emit();
    }

    #[private]
    pub fn on_unstake(&mut self, account_id: AccountId, token_id: TokenId) {
        self.account_to_token_id.remove(&account_id);
        self.token_id_to_account.remove(&token_id);

        self.total_nft_staked -= 1;

        RewarderEvent::NFTUnstaked {
            account_id: account_id.clone(),
            token_id: token_id.clone(),
        }
        .emit();
    }
}

#[cfg(test)]
mod tests {
    use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
    use near_sdk::{
        json_types::U128,
        test_utils::{accounts, get_logs, VMContextBuilder},
        testing_env,
    };

    use super::*;

    #[test]
    fn test_unstake() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());
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
        contract.on_unstake(alice.clone(), "1".to_string());

        assert_eq!(contract.account_to_token_id.get(&alice), None);
        assert_eq!(contract.total_nft_staked, 0);
    }

    #[test]
    fn test_query_staker_of_nft_correctly() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();

        let alice = accounts(1);

        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context);

        // Alice stakes NFT 1
        contract.nft_on_transfer(accounts(2), alice.clone(), "1".to_string(), "".to_string());
        assert_eq!(
            contract.primary_nft_of(alice.clone()),
            Some(("1".to_string(), U128(0)))
        );

        let staker = contract.token_id_to_account.get("1").unwrap();
        assert_eq!(staker, &alice);

        let staker = contract.staker_of("1".to_string());
        assert_eq!(staker, Some(alice));
    }

    #[test]
    fn test_emit_nft_staked_event() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());
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

        let logs = get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0],
            format!(
                r#"EVENT_JSON:{{"standard":"shitzurewarder","version":"1.0.0","event":"n_f_t_staked","data":{{"account_id":"{}","token_id":"1"}}}}"#,
                alice
            )
        );
    }

    #[test]
    fn test_emit_nft_unstaked_event() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();
        let alice = accounts(1);

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());
        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();
        testing_env!(context);

        // Alice stakes NFT 1
        contract.nft_on_transfer(accounts(2), alice.clone(), "1".to_string(), "".to_string());
        contract.on_unstake(alice.clone(), "1".to_string());

        assert_eq!(contract.account_to_token_id.get(&alice), None);
        assert_eq!(contract.total_nft_staked, 0);

        let logs = get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(
            logs[1],
            format!(
                r#"EVENT_JSON:{{"standard":"shitzurewarder","version":"1.0.0","event":"n_f_t_unstaked","data":{{"account_id":"{}","token_id":"1"}}}}"#,
                alice
            )
        );
    }
}
