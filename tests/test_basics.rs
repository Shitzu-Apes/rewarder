use anyhow::Ok;
use helpers::{call, setup::setup, view};
use near_sdk::{json_types::U128, NearToken};

mod helpers;

#[tokio::test]
async fn test_only_contract_can_send_shitzu() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (token, _nft, rewarder, accounts) = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let amount = U128(NearToken::from_near(1_000_000).as_yoctonear());
    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    assert_eq!(view::ft_balance_of(&token, rewarder.id()).await?, amount);

    assert!(bob
        .call(rewarder.id(), "send_rewards")
        .args_json((alice.id(), amount))
        .max_gas()
        .transact()
        .await?
        .into_result()
        .is_err());

    assert!(rewarder
        .call("send_rewards")
        .args_json((alice.id(), amount))
        .max_gas()
        .transact()
        .await?
        .into_result()
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_double_reward_nft_staker() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (token, nft, rewarder, accounts) = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 2 accounts, got {}", accounts.len())
    };

    let amount = U128(NearToken::from_near(1_000_000).as_yoctonear());
    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, bob, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    let [alice_token, ..] = &call::mint_nft(alice, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 token, got 0")
    };

    call::stake(alice, rewarder.id(), &nft.id(), &alice_token.token_id).await?;

    let token_id = view::primary_nft_of(&rewarder, &alice.id())
        .await?
        .expect("Fail to stake NFT");

    assert_eq!(token_id, alice_token.token_id);

    let reward = U128(NearToken::from_near(100).as_yoctonear());

    call::send_rewards(&rewarder, alice.id(), reward).await?;
    assert_eq!(
        view::ft_balance_of(&token, alice.id()).await?,
        U128(reward.0 * 2)
    );

    call::send_rewards(&rewarder, bob.id(), reward).await?;
    assert_eq!(view::ft_balance_of(&token, bob.id()).await?, reward);

    Ok(())
}

#[tokio::test]
async fn test_unstake() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (token, nft, rewarder, accounts) = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 2 accounts, got {}", accounts.len())
    };

    let amount = U128(NearToken::from_near(1_000_000).as_yoctonear());
    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, bob, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    let [alice_token1, alice_token2, ..] = &call::mint_nft(alice, nft.id(), 2).await?[..] else {
        anyhow::bail!("Expected at least 2 token, got less")
    };

    let [bob_token1, ..] = &call::mint_nft(bob, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 token, got less")
    };

    call::stake(alice, rewarder.id(), &nft.id(), &alice_token1.token_id).await?;
    call::stake(bob, rewarder.id(), &nft.id(), &bob_token1.token_id).await?;
    assert_eq!(
        view::nft_tokens_for_owner(&nft, rewarder.id())
            .await?
            .iter()
            .map(|t| t.token_id.to_owned())
            .collect::<Vec<_>>(),
        vec![alice_token1.token_id.clone(), bob_token1.token_id.clone()]
    );

    // Staking again with another token should not work
    call::stake(alice, rewarder.id(), &nft.id(), &alice_token2.token_id).await?;
    assert_eq!(
        view::nft_tokens_for_owner(&nft, alice.id()).await?,
        vec![alice_token2.clone()]
    );

    // Need to unstake before staking again
    call::unstake(alice, rewarder.id()).await?;
    call::stake(alice, rewarder.id(), &nft.id(), &alice_token2.token_id).await?;

    assert_eq!(
        view::nft_tokens_for_owner(&nft, rewarder.id())
            .await?
            .iter()
            .map(|t| t.token_id.to_owned())
            .collect::<Vec<_>>(),
        vec![bob_token1.token_id.clone(), alice_token2.token_id.clone(),]
    );

    Ok(())
}

#[tokio::test]
async fn test_nft_score_persist() -> anyhow::Result<()> {
    // This test is to make sure that the score of the NFT is persisted after unstaking
    // or even the owner of the NFT is changed and come back to stake again
    let worker = near_workspaces::sandbox().await?;
    let (token, nft, rewarder, accounts) = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 2 accounts, got {}", accounts.len())
    };

    let amount = U128(NearToken::from_near(1_000_000).as_yoctonear());

    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, bob, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;
    call::transfer_token(&token.id(), alice, rewarder.id(), amount).await?;

    let [nft_token, ..] = &call::mint_nft(alice, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 token, got 0")
    };

    let reward = U128(NearToken::from_near(100).as_yoctonear());

    call::stake(alice, rewarder.id(), &nft.id(), &nft_token.token_id).await?;

    call::send_rewards(&rewarder, alice.id(), reward).await?;

    call::unstake(alice, rewarder.id()).await?;

    assert_eq!(
        view::score_of(&rewarder, nft_token.token_id.clone()).await?,
        U128(reward.0 * 2)
    );

    call::transfer_nft(&alice, bob.id(), &nft.id(), &nft_token.token_id).await?;

    call::stake(&bob, rewarder.id(), nft.id(), &nft_token.token_id).await?;

    call::send_rewards(&rewarder, bob.id(), reward).await?;

    assert_eq!(
        view::score_of(&rewarder, nft_token.token_id.clone()).await?,
        U128(reward.0 * 4)
    );

    Ok(())
}

#[tokio::test]
async fn test_donation_quadruple_score() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (token, nft, rewarder, accounts) = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 2 accounts, got {}", accounts.len())
    };

    let amount = U128(NearToken::from_near(1_000_000).as_yoctonear());

    call::storage_deposit(&token, alice, None, None).await?;
    call::storage_deposit(&token, bob, None, None).await?;
    call::storage_deposit(&token, alice, Some(rewarder.id()), None).await?;

    call::mint_token(&token, alice.id(), amount).await?;

    let [nft_token, ..] = &call::mint_nft(alice, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 token, got 0")
    };
    call::stake(alice, rewarder.id(), &nft.id(), &nft_token.token_id).await?;

    call::donate(&alice, token.id(), rewarder.id(), amount).await?;

    assert_eq!(
        view::score_of(&rewarder, nft_token.token_id.clone()).await?,
        U128(amount.0 * 4)
    );

    Ok(())
}
