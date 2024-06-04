use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{json_types::U128, near_bindgen, AccountId};

#[near_bindgen(event_json(standard = "shitzurewarder"))]
#[derive(Debug)]
pub enum RewarderEvent {
    #[event_version("1.0.0")]
    RewardSent {
        account_id: AccountId,
        amount: U128,
        token_id: Option<TokenId>,
    },
    #[event_version("1.0.0")]
    ScoreRecorded { token_id: TokenId, score: U128 },
    #[event_version("1.0.0")]
    NftStaked {
        account_id: AccountId,
        token_id: TokenId,
    },
    #[event_version("1.0.0")]
    NftUnstaked {
        account_id: AccountId,
        token_id: TokenId,
    },
}
