use near_sdk::NearToken;
use near_workspaces::{network::Sandbox, Account, Contract, Worker};
use serde_json::json;

use super::log_tx_result;

const SHITZU_TOKEN_WASM_FILEPATH: &str = "./res/test_token.wasm";
const SHITZU_NFT_WASM_FILEPATH: &str = "./res//shitzu_nft.wasm";
const REWARDER_WASM_FILEPATH: &str = "./res/rewarder.wasm";

pub async fn setup_token(
    near: &Account,
    name: &str,
    symbol: &str,
    decimals: u8,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(SHITZU_TOKEN_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount(&symbol.to_lowercase())
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        &format!("Deployed token contract {}", symbol),
        contract
            .call("new")
            .args_json((name, symbol, Option::<&str>::None, decimals))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub async fn setup_nft(near: &Account) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(SHITZU_NFT_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount("nft".into())
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        &format!("Deployed NFT contract nft"),
        contract
            .call("new_init")
            .args_json(json!(
                {
                    "owner_id": near.id(),
                    "icon": ""
                }
            ))
            .max_gas()
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub async fn setup_contract(
    near: &Account,
    reward_token: &Contract,
    nft: &Contract,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(REWARDER_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount("rewarder".into())
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        "Deployed rewarder contract",
        contract
            .call("new")
            .args_json(json!(
                {
                    "reward_token": reward_token.id(),
                    "nft": nft.id()
                }
            ))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub async fn setup() -> anyhow::Result<(Worker<Sandbox>, Contract, Contract, Contract, Vec<Account>)>
{
    let worker = near_workspaces::sandbox().await?;

    let near = worker.root_account()?;

    let shitzu = setup_token(&near, "SHITZU", "SHITZU", 24).await?;
    let nft = setup_nft(&near).await?;
    let contract = setup_contract(&near, &shitzu, &nft).await?;

    let mut accounts = vec![];

    for i in 0..10 {
        let account = near
            .create_subaccount(&format!("account{}", i))
            .initial_balance(NearToken::from_near(100))
            .transact()
            .await?
            .into_result()?;
        accounts.push(account);
    }

    Ok((worker, shitzu, nft, contract, accounts))
}
