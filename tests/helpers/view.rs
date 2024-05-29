use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::{json_types::U128, AccountId};
use near_workspaces::Contract;

pub async fn ft_balance_of(contract: &Contract, account_id: &AccountId) -> anyhow::Result<u128> {
    let res = contract
        .call("ft_balance_of")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<U128>()?.0)
}

pub async fn primary_nft_of(
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<Option<TokenId>> {
    let res = contract
        .call("primary_nft_of")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<Option<TokenId>>()?)
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
