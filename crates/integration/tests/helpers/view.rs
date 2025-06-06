use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::{json_types::U128, serde::Deserialize, AccountId};
use near_workspaces::Contract;

use super::setup::FarmConfigs;

pub async fn ft_balance_of(contract: &Contract, account_id: &AccountId) -> anyhow::Result<U128> {
    let res = contract
        .call("ft_balance_of")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<U128>()?)
}

pub async fn ft_total_supply(contract: &Contract) -> anyhow::Result<U128> {
    let res = contract.call("ft_total_supply").view().await?;

    Ok(res.json::<U128>()?)
}

pub async fn primary_nft_of(
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<(TokenId, U128)> {
    let res = contract
        .call("primary_nft_of")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<_>()?)
}

pub async fn nft_tokens_for_owner(
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<Vec<Token>> {
    let res: near_workspaces::result::ViewResultDetails = contract
        .call("nft_tokens_for_owner")
        .args_json((account_id, None::<U128>, None::<u64>))
        .view()
        .await?;

    Ok(res.json::<Vec<Token>>()?)
}

pub async fn score_of(contract: &Contract, token_id: TokenId) -> anyhow::Result<U128> {
    let res = contract
        .call("score_of")
        .args_json((token_id,))
        .view()
        .await?;

    Ok(res.json::<U128>()?)
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmerSeed {
    pub free_amount: U128,
}

pub async fn get_farmer_seed(
    contract: &Contract,
    farmer_id: &AccountId,
    seed_id: &str,
) -> anyhow::Result<FarmerSeed> {
    let res = contract
        .call("get_farmer_seed")
        .args_json((farmer_id, seed_id))
        .view()
        .await?;

    Ok(res.json::<FarmerSeed>()?)
}

pub async fn get_user_checkpoint(
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<Option<u64>> {
    let res = contract
        .call("get_user_checkpoint")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<Option<u64>>()?)
}

pub async fn get_farm_configs(contract: &Contract) -> anyhow::Result<FarmConfigs> {
    let res = contract.call("get_farm_configs").view().await?;

    Ok(res.json::<FarmConfigs>()?)
}
