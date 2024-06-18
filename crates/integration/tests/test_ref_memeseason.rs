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
        shitzu_staking,
        ..
    } = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let amount: U128 = Ether::from(1_000_000).into();
    call::storage_deposit(&shitzu, alice, None, None).await?;
    call::storage_deposit(&shitzu, alice, Some(rewarder.id()), None).await?;
    call::storage_deposit(&shitzu, alice, Some(shitzu_staking.id()), None).await?;

    call::storage_deposit(&shitzu_staking, alice, None, None).await?;

    call::mint_token(&shitzu, alice.id(), amount).await?;
    call::mint_token(&shitzu, rewarder.id(), amount).await?;

    call::stake_seed(
        &alice,
        shitzu.id(),
        U128(Ether::from(100).0),
        shitzu_staking.id(),
    )
    .await?;

    let seed = view::get_farmer_seed(&shitzu_staking, alice.id(), &shitzu.id().to_string()).await?;
    assert!(
        seed.free_amount.0 > 0,
        "Expected free amount to be greater than 0, got {}",
        seed.free_amount.0
    );

    Ok(())
}
