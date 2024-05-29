use anyhow::Ok;
use helpers::{call, setup::setup, view};
use near_sdk::NearToken;

mod helpers;

#[tokio::test]
async fn test_only_contract_can_send_shitzu() -> anyhow::Result<()> {
    let (_worker, token, _nft, rewarder, accounts) = setup().await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let amount = NearToken::from_near(1_000_000).as_yoctonear();
    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    assert_eq!(view::ft_balance_of(&token, rewarder.id()).await?, amount);

    assert!(bob
        .call(rewarder.id(), "send_rewards")
        .args_json((alice.id(), amount))
        .transact()
        .await?
        .into_result()
        .is_err());

    assert!(rewarder
        .call("send_rewards")
        .args_json((alice.id(), amount / 2))
        .transact()
        .await?
        .into_result()
        .is_ok());

    Ok(())
}
