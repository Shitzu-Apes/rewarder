use anyhow::Ok;
use helpers::{call, setup::setup, view};
use near_contract_standards::non_fungible_token::Token;
use near_sdk::NearToken;
use serde_json::json;

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

#[tokio::test]
async fn test_double_reward_nft_staker() -> anyhow::Result<()> {
    let (_worker, token, nft, rewarder, accounts) = setup().await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 2 accounts, got {}", accounts.len())
    };

    let amount = NearToken::from_near(1_000_000).as_yoctonear();
    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, bob, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    call::mint_nft(alice, nft.id(), 1).await?;
    let alice_token = view::nft_first_token_of(&nft, &alice.id())
        .await?
        .expect("Fail to mint NFT");

    call::stake(alice, rewarder.id(), &nft.id(), &alice_token.token_id).await?;

    let token_id = view::primary_nft_of(&rewarder, &alice.id())
        .await?
        .expect("Fail to stake NFT");

    assert_eq!(token_id, alice_token.token_id);

    let reward = NearToken::from_near(100).as_yoctonear();

    call::send_rewards(&rewarder, alice.id(), reward).await?;
    assert_eq!(view::ft_balance_of(&token, alice.id()).await?, reward * 2);

    call::send_rewards(&rewarder, bob.id(), reward).await?;
    assert_eq!(view::ft_balance_of(&token, bob.id()).await?, reward);

    Ok(())
}
