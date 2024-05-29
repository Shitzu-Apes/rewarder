use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::{Account, Contract};

pub async fn storage_deposit(
    contract: &Contract,
    sender: &Account,
    account_id: Option<&AccountId>,
    deposit: Option<NearToken>,
) -> anyhow::Result<()> {
    sender
        .call(contract.id(), "storage_deposit")
        .args_json((account_id, None::<bool>))
        .deposit(deposit.unwrap_or(NearToken::from_near(1)))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn mint_token(
    contract: &Contract,
    receiver: &AccountId,
    amount: u128,
) -> anyhow::Result<()> {
    contract
        .call("mint")
        .args_json((receiver, U128::from(amount)))
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn mint_nft(sender: &Account, nft: &AccountId, quantity: u32) -> anyhow::Result<()> {
    sender
        .call(nft, "nft_mint")
        .args_json((quantity,))
        .deposit(NearToken::from_near(5))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn transfer_token(
    token: &AccountId,
    sender: &Account,
    receiver: &AccountId,
    amount: u128,
) -> anyhow::Result<()> {
    sender
        .call(token, "ft_transfer")
        .args_json((receiver, U128::from(amount), "".to_string()))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn stake(
    staker: &Account,
    receiver: &AccountId,
    nft: &AccountId,
    token_id: &TokenId,
) -> anyhow::Result<()> {
    staker
        .call(nft, "nft_transfer_call")
        .args_json((
            receiver,
            token_id,
            None::<u64>,
            None::<String>,
            "".to_string(),
        ))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(())
}
