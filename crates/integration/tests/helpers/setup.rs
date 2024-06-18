use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use futures::future::join_all;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::AccountId;
use near_sdk::NearToken;
use near_workspaces::{network::Sandbox, Account, Contract, Worker};
use serde_json::json;
use tokio::task::JoinHandle;

use super::call;
use super::log_tx_result;

const SHITZU_TOKEN_WASM_FILEPATH: &str = "../../res/test_token.wasm";
const SHITZU_NFT_WASM_FILEPATH: &str = "../../res//shitzu_nft.wasm";
const REWARDER_WASM_FILEPATH: &str = "../../res/rewarder.wasm";
const REF_FARM_WASM_FILEPATH: &str = "../../res/ref_farm.wasm";
const MEMESEASON_WASM_FILEPATH: &str = "../../res/memeseason.wasm";

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
        .create_subaccount("nft")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        "Deployed NFT contract nft",
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

pub async fn setup_rewarder(
    near: &Account,
    owner_id: &AccountId,
    operator_id: &AccountId,
    reward_token: &Contract,
    nft: &Contract,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(REWARDER_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount("rewarder")
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
                    "owner": owner_id,
                    "operator": operator_id,
                    "whitelisted_record_score_ids": [],
                    "reward_token": reward_token.id(),
                    "nft": nft.id()
                }
            ))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub async fn setup_ref_farm(
    near: &Account,
    subaccount: String,
    owner_id: &AccountId,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(REF_FARM_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount(&subaccount)
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        "Deployed ref farm contract",
        contract
            .call("new")
            .args_json(json!(
                {
                    "owner_id": owner_id,
                }
            ))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FarmConfig {
    pub farm_id: AccountId,
    pub seed_id: String,
    pub factor: U128,
    pub cap: U128,
    pub decimals: u8,
}

pub async fn setup_memeseason(
    near: &Account,
    rewarder: &Contract,
    xref: &FarmConfig,
    shitzu: &FarmConfig,
    lp: &FarmConfig,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(MEMESEASON_WASM_FILEPATH)?;

    let contract = near
        .create_subaccount("memeseason")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    log_tx_result(
        "Deployed memeseason contract",
        contract
            .call("new")
            .args_json(json!(
                {
                    "rewarder": rewarder.id(),
                    "xref": xref,
                    "shitzu": shitzu,
                    "lp": lp,
                }
            ))
            .transact()
            .await?,
    )?;

    Ok(contract)
}

pub struct SetupResult {
    pub dao: Account,
    pub tgbot: Account,
    pub shitzu: Contract,
    pub nft: Contract,
    pub rewarder: Contract,
    pub memeseason: Contract,
    pub accounts: Vec<Account>,
}

pub async fn setup(worker: &Worker<Sandbox>) -> anyhow::Result<SetupResult> {
    let near = worker.root_account()?;
    let dao = near
        .create_subaccount("dao")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?;
    let tgbot = near
        .create_subaccount("tgbot")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?;

    let shitzu = setup_token(&near, "SHITZU", "SHITZU", 18).await?;
    let nft = setup_nft(&near).await?;
    let rewarder = setup_rewarder(&near, dao.id(), tgbot.id(), &shitzu, &nft).await?;

    let ref_admin = near
        .create_subaccount("ref_admin")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?;

    let xref = setup_token(&near, "xref", "xref", 18).await?;
    let xref_staking =
        setup_ref_farm(&near, "xref_staking".parse().unwrap(), ref_admin.id()).await?;
    call::create_seed(&ref_admin, xref_staking.id(), xref.id(), 18).await?;
    let xref_config = FarmConfig {
        farm_id: xref_staking.id().clone(),
        seed_id: xref.id().to_string(),
        factor: U128("11000000000000000000000".parse().unwrap()),
        cap: U128(200),
        decimals: 18,
    };

    let shitzu_staking =
        setup_ref_farm(&near, "shitzu_staking".parse().unwrap(), ref_admin.id()).await?;
    call::create_seed(&ref_admin, shitzu_staking.id(), shitzu.id(), 18).await?;
    let shitzu_config = FarmConfig {
        farm_id: shitzu_staking.id().clone(),
        seed_id: shitzu.id().to_string(),
        factor: U128("547000000000000000000000".parse().unwrap()),
        cap: U128(100),
        decimals: 18,
    };

    let mock_lp_token = setup_token(&near, "ref_4369", "ref_4369", 24).await?;
    let lp_staking = setup_ref_farm(&near, "lp_staking".parse().unwrap(), ref_admin.id()).await?;
    call::create_seed(&ref_admin, lp_staking.id(), mock_lp_token.id(), 24).await?;
    let lp_config = FarmConfig {
        farm_id: lp_staking.id().clone(),
        seed_id: mock_lp_token.id().to_string(),
        factor: U128("17000000000000000000000000".parse().unwrap()),
        cap: U128(100),
        decimals: 24,
    };

    let memeseason =
        setup_memeseason(&near, &rewarder, &xref_config, &shitzu_config, &lp_config).await?;

    call::whitelist(&dao, rewarder.id(), memeseason.id()).await?;

    let mut tasks: Vec<JoinHandle<anyhow::Result<Account>>> = Vec::new();

    for _ in 0..10 {
        let name: String = Name(EN).fake::<String>().to_lowercase();

        // Make sure it is a valid account name
        let name = name.replace(' ', "_").replace(['\'', '-'], "");

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

    Ok(SetupResult {
        dao,
        tgbot,
        shitzu,
        nft,
        rewarder,
        memeseason,
        accounts,
    })
}
