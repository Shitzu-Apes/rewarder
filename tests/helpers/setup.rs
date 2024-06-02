use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use futures::future::join_all;
use near_sdk::AccountId;
use near_sdk::NearToken;
use near_workspaces::{network::Sandbox, Account, Contract, Worker};
use serde_json::json;
use tokio::task::JoinHandle;

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
    owner_id: &AccountId,
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
                    "owner_id": owner_id,
                    "reward_token": reward_token.id(),
                    "nft": nft.id()
                }
            ))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub async fn setup(
    worker: &Worker<Sandbox>,
) -> anyhow::Result<(Contract, Contract, Contract, Vec<Account>)> {
    let near = worker.root_account()?;
    let dao = near
        .create_subaccount("dao".into())
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?;

    let shitzu = setup_token(&near, "SHITZU", "SHITZU", 24).await?;
    let nft = setup_nft(&near).await?;
    let contract = setup_contract(&near, dao.id(), &shitzu, &nft).await?;

    let mut tasks: Vec<JoinHandle<anyhow::Result<Account>>> = Vec::new();

    for _ in 0..10 {
        let name: String = Name(EN).fake::<String>().to_lowercase();

        // Make sure it is a valid account name
        let name = name.replace(" ", "_").replace("'", "").replace("-", "");

        let near = near.clone();
        let task: JoinHandle<anyhow::Result<Account>> = tokio::spawn(async move {
            let account = near
                .create_subaccount(&name)
                .initial_balance(NearToken::from_near(10_000))
                .transact()
                .await?
                .into_result()
                .map_err(|e| anyhow::anyhow!("Error creating account: {}", e))?;

            anyhow::Ok(account)
        });

        tasks.push(task);
    }
    let mut accounts = vec![];

    let result = join_all(tasks).await;
    result.into_iter().for_each(|r| match r {
        Ok(account) => accounts.push(account.unwrap()),
        Err(e) => eprintln!("Error creating account: {}", e),
    });

    Ok((shitzu, nft, contract, accounts))
}
