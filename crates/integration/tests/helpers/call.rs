use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::{Account, Contract};

use super::{events::ContractEvent, log_tx_result};

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
    amount: U128,
) -> anyhow::Result<()> {
    contract
        .call("mint")
        .args_json((receiver, amount))
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn mint_nft(
    sender: &Account,
    nft: &AccountId,
    quantity: u32,
) -> anyhow::Result<Vec<Token>> {
    let res = sender
        .call(nft, "nft_mint")
        .args_json((quantity,))
        .deposit(NearToken::from_near(50))
        .max_gas()
        .transact()
        .await?;

    Ok(res.json::<Vec<Token>>()?)
}

pub async fn transfer_token(
    token: &AccountId,
    sender: &Account,
    receiver: &AccountId,
    amount: U128,
) -> anyhow::Result<()> {
    sender
        .call(token, "ft_transfer")
        .args_json((receiver, amount, "".to_string()))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

pub async fn transfer_nft(
    sender: &Account,
    receiver: &AccountId,
    nft: &AccountId,
    token_id: &TokenId,
) -> anyhow::Result<()> {
    sender
        .call(nft, "nft_transfer")
        .args_json((receiver, token_id, None::<u64>, None::<String>))
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
) -> anyhow::Result<Vec<ContractEvent>> {
    let (_, events) = log_tx_result(
        "stake",
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
            .await?,
    )?;

    Ok(events)
}

pub async fn send_rewards(
    operator: &Account,
    contract: &AccountId,
    account_id: &AccountId,
    amount: U128,
) -> anyhow::Result<Vec<ContractEvent>> {
    let (_, events) = log_tx_result(
        "send_rewards",
        operator
            .call(contract, "send_rewards")
            .args_json((account_id, amount))
            .max_gas()
            .transact()
            .await?,
    )?;

    Ok(events)
}

pub async fn unstake(staker: &Account, rewarder: &AccountId) -> anyhow::Result<Vec<ContractEvent>> {
    let (_, events) = log_tx_result(
        "unstake",
        staker
            .call(rewarder, "unstake")
            .max_gas()
            .transact()
            .await?,
    )?;

    Ok(events)
}

pub async fn donate(
    donor: &Account,
    token: &AccountId,
    rewarder: &AccountId,
    amount: U128,
) -> anyhow::Result<Vec<ContractEvent>> {
    let (_, events) = log_tx_result(
        "donate",
        donor
            .call(token, "ft_transfer_call")
            .args_json((rewarder, amount, "".to_string(), "".to_string()))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?,
    )?;

    Ok(events)
}

pub async fn create_seed(
    owner: &Account,
    farm_id: &AccountId,
    seed_id: &AccountId,
    seed_decimal: u32,
) -> anyhow::Result<Vec<ContractEvent>> {
    let (_, events) = log_tx_result(
        "create_seed",
        owner
            .call(farm_id, "create_seed")
            .args_json((seed_id, seed_decimal, None::<U128>, None::<u32>))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?,
    )?;

    Ok(events)
}
