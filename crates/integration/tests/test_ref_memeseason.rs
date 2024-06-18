use helpers::{
    call,
    setup::{setup, SetupResult},
    view, Ether,
};
use near_sdk::json_types::U128;

mod helpers;

#[tokio::test]
async fn test_farmer_can_claim() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let SetupResult {
        shitzu,
        rewarder,
        accounts,
        memeseason,
        ..
    } = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let amount: U128 = Ether::from(1_000_000).into();
    call::storage_deposit(&shitzu, alice, None, None).await?;
    call::storage_deposit(&shitzu, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&shitzu, alice.id(), amount).await?;
    call::transfer_token(shitzu.id(), alice, rewarder.id(), amount).await?;

    Ok(())
}
