mod event;
mod ft;
mod nft;
mod owner;
mod token_receiver;
mod view;

use event::RewarderEvent;
use near_contract_standards::{
    fungible_token::{core::ext_ft_core, events::FtMint},
    non_fungible_token::TokenId,
};
use near_sdk::{
    borsh::BorshSerialize,
    env,
    json_types::U128,
    near, require,
    store::{LookupMap, TreeMap},
    AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise,
};
use primitive_types::U256;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(8);
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(10);

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    operator: AccountId,
    whitelisted_record_score_ids: Vec<AccountId>,

    reward_token: AccountId,
    nft: AccountId,

    account_to_token_id: LookupMap<AccountId, TokenId>,
    token_id_to_account: LookupMap<TokenId, AccountId>,
    total_nft_staked: u128,

    total_score: u128,
    total_distribute: u128,
    total_donation: u128,
    scores: LookupMap<TokenId, u128>,
    ranking: TreeMap<u128, Vec<TokenId>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    PrimaryNFT,
    OwnerNFT,
    Ranking,
    Scores,
    DonationAmounts,
    DonorRanking,
}

#[near]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        operator: AccountId,
        whitelisted_record_score_ids: Vec<AccountId>,
        reward_token: AccountId,
        nft: AccountId,
    ) -> Self {
        Self {
            owner,
            operator,
            whitelisted_record_score_ids,

            reward_token,
            nft,

            account_to_token_id: LookupMap::new(StorageKey::PrimaryNFT),
            token_id_to_account: LookupMap::new(StorageKey::OwnerNFT),
            total_nft_staked: 0,

            total_score: 0,
            total_distribute: 0,
            total_donation: 0,
            ranking: TreeMap::new(StorageKey::Ranking),
            scores: LookupMap::new(StorageKey::Scores),
        }
    }

    pub fn send_rewards(&mut self, account_id: AccountId, amount: U128) -> Promise {
        self.require_operator();

        let (some_primary_nft_token_id, amount) =
            if let Some(primary_nft) = self.account_to_token_id.get(&account_id) {
                (Some(primary_nft.clone()), U128(amount.0 * 2))
            } else {
                (None, amount)
            };

        RewarderEvent::RewardSent {
            account_id: account_id.clone(),
            token_id: some_primary_nft_token_id.clone(),
            amount,
        }
        .emit();

        ext_ft_core::ext(self.reward_token.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(account_id.clone(), amount, None)
            .then(
                Self::ext(env::current_account_id())
                    .with_unused_gas_weight(1)
                    .on_reward_sent(some_primary_nft_token_id, amount),
            )
    }

    #[private]
    pub fn on_reward_sent(&mut self, primary_nft: Option<TokenId>, amount: U128) {
        if let Some(primary_nft) = primary_nft {
            self.internal_record_score(primary_nft, amount.0);
        }

        self.total_distribute =
            (U256::from(self.total_distribute) + U256::from(amount.0)).as_u128();
    }

    pub fn on_track_score(&mut self, primary_nft: TokenId, amount: U128) {
        require!(
            self.whitelisted_record_score_ids
                .contains(&env::predecessor_account_id()),
            "Only whitelisted contracts can call this function"
        );

        self.internal_record_score(primary_nft, amount.0);
    }
}

impl Contract {
    fn require_operator(&self) {
        require!(
            env::predecessor_account_id() == self.operator,
            "Only operator can send rewards"
        );
    }

    fn internal_record_score(&mut self, primary_nft: TokenId, amount: u128) -> u128 {
        let score = *self.scores.get(&primary_nft).unwrap_or(&0);
        let new_score = (U256::from(score) + U256::from(amount)).as_u128();
        self.scores.set(primary_nft.clone(), Some(new_score));

        // remove from old ranking
        let mut ranking = self.ranking.get(&score).unwrap_or(&Vec::new()).clone();
        ranking.retain(|x| x != &primary_nft);

        if ranking.is_empty() {
            self.ranking.remove(&score);
        } else {
            self.ranking.insert(score, ranking);
        }

        let mut ranking = self.ranking.get(&new_score).unwrap_or(&Vec::new()).clone();
        ranking.push(primary_nft.clone());
        self.ranking.insert(new_score, ranking);

        self.total_score += amount;
        let owner_id = self.token_id_to_account.get(&primary_nft).unwrap();
        FtMint {
            owner_id,
            amount: amount.into(),
            memo: None,
        }
        .emit();

        RewarderEvent::ScoreRecorded {
            token_id: primary_nft,
            score: U128(new_score),
        }
        .emit();

        amount
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
    #[should_panic(expected = "Only operator can send rewards")]
    fn test_panic_non_operator_cannot_send_rewards() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(alice_id.clone())
            .build();

        testing_env!(context.clone());
        contract.send_rewards(alice_id.clone(), U128(amount));
        assert_eq!(contract.total_distribute, 0);
    }

    #[test]
    fn test_operator_can_send_reward() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(
            dao,
            operator.clone(),
            vec![],
            reward_token.clone(),
            nft.clone(),
        );

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(operator.clone())
            .attached_deposit(NearToken::from_yoctonear(1))
            .build();
        testing_env!(context.clone());
        contract.send_rewards(alice_id.clone(), U128(amount));
    }

    #[test]
    fn test_double_reward_nft_staker() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();

        testing_env!(context.clone());
        contract.nft_on_transfer(accounts(1), alice_id.clone(), "1".into(), "".into());
        contract.internal_record_score("1".into(), amount * 2);

        assert_eq!(contract.scores.get("1"), Some(&(amount * 2)));
    }

    #[test]
    fn test_nft_ranking() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let bob_id: AccountId = "bob.near".parse().unwrap();
        let charlie_id: AccountId = "charlie.near".parse().unwrap();
        let dan_id: AccountId = "dan.near".parse().unwrap();
        let amount = 100 * 10_u128.pow(18);
        let fifty = 50 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();

        testing_env!(context.clone());

        // Alice stakes NFT 1 and receives the lowest score
        contract.nft_on_transfer(accounts(1), alice_id.clone(), "1".into(), "".into());
        contract.internal_record_score("1".into(), (amount - fifty) * 2);

        // Bob stakes NFT 2 and receives the highest score
        contract.nft_on_transfer(accounts(1), bob_id.clone(), "2".into(), "".into());
        contract.internal_record_score("2".into(), (amount + fifty) * 2);

        // Charlie stakes NFT 3 and receives the lowest score (same as Alice)
        contract.nft_on_transfer(accounts(1), charlie_id.clone(), "3".into(), "".into());
        contract.internal_record_score("3".into(), (amount - fifty) * 2);

        // Dan stakes NFT 4 and receives the middle score
        contract.nft_on_transfer(accounts(1), dan_id.clone(), "4".into(), "".into());
        contract.internal_record_score("4".into(), amount * 2);

        // Expect the ranking to be [((amount + fifty) * 2, [2]), (amount * 2, [4]), ((amount - fifty) * 2, [1, 3])]
        let ranking = contract.get_leaderboard(Some(3));
        assert_eq!(
            ranking,
            vec![
                (
                    U128((amount + fifty) * 2),
                    vec![("2".to_string(), Some(bob_id.clone()))]
                ),
                (
                    U128(amount * 2),
                    vec![("4".to_string(), Some(dan_id.clone()))]
                ),
                (
                    U128((amount - fifty) * 2),
                    vec![
                        ("1".to_string(), Some(alice_id.clone())),
                        ("3".to_string(), Some(charlie_id.clone()))
                    ]
                )
            ]
        );

        // Dan scores another fifty points and should move to the same score as Bob
        contract.internal_record_score("4".into(), fifty * 2);

        // Expect the ranking to be [((amount + fifty) * 2, [2, 4]), ((amount - fifty) * 2, [1, 3])]
        let ranking = contract.get_leaderboard(None);
        assert_eq!(
            ranking,
            vec![
                (
                    U128((amount + fifty) * 2),
                    vec![
                        ("2".to_string(), Some(bob_id.clone())),
                        ("4".to_string(), Some(dan_id.clone()))
                    ]
                ),
                (
                    U128((amount - fifty) * 2),
                    vec![
                        ("1".to_string(), Some(alice_id.clone())),
                        ("3".to_string(), Some(charlie_id.clone()))
                    ]
                )
            ]
        );
    }

    #[test]
    fn test_nft_has_score_after_unstake() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(dao, operator, vec![], reward_token.clone(), nft.clone());

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(nft.clone())
            .build();

        testing_env!(context.clone());

        // Alice stakes NFT 1
        contract.nft_on_transfer(accounts(1), alice_id.clone(), "1".into(), "".into());
        contract.internal_record_score("1".into(), amount * 2);

        // Alice unstakes NFT 1
        let context = VMContextBuilder::new()
            .predecessor_account_id(alice_id.clone())
            .build();
        testing_env!(context.clone());
        contract.on_unstake(alice_id.clone(), "1".into());

        let ranking = contract.get_leaderboard(None);
        assert_eq!(
            ranking,
            vec![(U128(amount * 2), vec![("1".to_string(), None)])]
        );
    }

    #[test]
    fn test_emit_reward_sent_event() {
        let reward_token: AccountId = "reward_token".parse().unwrap();
        let nft: AccountId = "nft".parse().unwrap();
        let dao: AccountId = "dao".parse().unwrap();
        let operator: AccountId = "operator".parse().unwrap();

        let mut contract = Contract::new(
            dao,
            operator.clone(),
            vec![],
            reward_token.clone(),
            nft.clone(),
        );

        let alice_id: AccountId = "alice.near".parse().unwrap();
        let amount = 1000 * 10_u128.pow(18);

        let context = VMContextBuilder::new()
            .predecessor_account_id(operator.clone())
            .attached_deposit(NearToken::from_yoctonear(1))
            .build();
        testing_env!(context.clone());
        contract.send_rewards(alice_id.clone(), U128(amount));

        let logs = get_logs();
        assert!(logs.len() == 1);
        assert_eq!(
            logs[0],
            format!(
                r#"EVENT_JSON:{{"standard":"shitzurewarder","version":"1.0.0","event":"reward_sent","data":{{"account_id":"{}","amount":"{}","token_id":null}}}}"#,
                alice_id, amount
            )
        );
    }
}
