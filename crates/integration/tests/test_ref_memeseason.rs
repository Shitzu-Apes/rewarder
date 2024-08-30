use helpers::{
    assert_approx_eq, call,
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
        nft,
        ..
    } = setup(&worker).await?;

    let [alice, bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let amount: U128 = Ether::from(900).into();
    call::storage_deposit(&shitzu, alice, None, None).await?;
    call::storage_deposit(&shitzu, alice, Some(rewarder.id()), None).await?;
    call::storage_deposit(&shitzu, alice, Some(shitzu_staking.id()), None).await?;

    call::storage_deposit(&shitzu_staking, alice, None, None).await?;

    call::mint_token(&shitzu, alice.id(), amount).await?;
    call::mint_token(&shitzu, rewarder.id(), amount).await?;

    call::stake_seed(alice, shitzu.id(), amount, shitzu_staking.id()).await?;

    let seed = view::get_farmer_seed(&shitzu_staking, alice.id(), shitzu.id().as_ref()).await?;
    assert!(
        seed.free_amount.0 > 0,
        "Expected free amount to be greater than 0, got {}",
        seed.free_amount.0
    );

    let [alice_nft_token_id, ..] = &call::mint_nft(alice, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 NFT token, got 0")
    };
    call::stake(alice, rewarder.id(), nft.id(), &alice_nft_token_id.token_id).await?;
    call::claim_ref_memeseason(alice, memeseason.id()).await?;

    let score = view::score_of(&rewarder, alice_nft_token_id.token_id.clone()).await?;
    let expected_score = U128(56000000000000000000);

    assert_approx_eq(
        score,
        expected_score,
        10, // 0.1%
        &format!("Expected score to be {}, got {}", expected_score.0, score.0),
    )?;

    let checkpoint = view::get_user_checkpoint(&memeseason, alice.id()).await?;
    assert!(
        checkpoint.is_some(),
        "Expected checkpoint to be Some, got None"
    );

    assert!(
        checkpoint.unwrap() > 0,
        "Expected checkpoint to be greater than 0, got {}",
        checkpoint.unwrap()
    );

    assert!(
        call::claim_ref_memeseason(alice, memeseason.id())
            .await
            .is_err(),
        "Try to claim too soon, expected error"
    );

    worker.fast_forward(600).await?;

    assert!(
        call::claim_ref_memeseason(alice, memeseason.id())
            .await
            .is_ok(),
        "Expected successful claim"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_farms() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let SetupResult {
        shitzu,
        rewarder,
        accounts,
        memeseason,
        xref,
        xref_staking,
        shitzu_staking,
        lp_token,
        lp_staking,
        nft,
        ..
    } = setup(&worker).await?;

    let [alice, _bob, ..] = &accounts[..] else {
        anyhow::bail!("Expected at least 4 accounts, got {}", accounts.len())
    };

    let [alice_nft_token_id, ..] = &call::mint_nft(alice, nft.id(), 1).await?[..] else {
        anyhow::bail!("Expected at least 1 NFT token, got 0")
    };

    call::stake(alice, rewarder.id(), nft.id(), &alice_nft_token_id.token_id).await?;

    let shitzu_amount: U128 = Ether::from(900).into();
    call::storage_deposit(&shitzu, alice, None, None).await?;
    call::storage_deposit(&shitzu, alice, Some(shitzu_staking.id()), None).await?;
    call::storage_deposit(&shitzu_staking, alice, None, None).await?;

    call::mint_token(&shitzu, alice.id(), shitzu_amount).await?;
    // 50 + 6 points
    call::stake_seed(alice, shitzu.id(), shitzu_amount, shitzu_staking.id()).await?;

    let xref_amount: U128 = Ether::from(100).into();
    call::storage_deposit(&xref, alice, None, None).await?;
    call::storage_deposit(&xref, alice, Some(xref_staking.id()), None).await?;
    call::storage_deposit(&xref_staking, alice, None, None).await?;

    call::mint_token(&xref, alice.id(), xref_amount).await?;
    // 100 + 10 points
    call::stake_seed(alice, xref.id(), xref_amount, xref_staking.id()).await?;

    let lp_amount: U128 = U128::from(10_u128.pow(22)); // 0.01 * 10^24
    call::storage_deposit(&lp_token, alice, None, None).await?;
    call::storage_deposit(&lp_token, alice, Some(lp_staking.id()), None).await?;
    call::storage_deposit(&lp_staking, alice, None, None).await?;

    call::mint_token(&lp_token, alice.id(), lp_amount).await?;
    // 50 + 10 points
    call::stake_seed(alice, lp_token.id(), lp_amount, lp_staking.id()).await?;

    call::claim_ref_memeseason(alice, memeseason.id()).await?;
    let score = view::score_of(&rewarder, alice_nft_token_id.token_id.clone()).await?;
    let expected_score = U128(226 * 10_u128.pow(18));

    assert!(
        score == expected_score,
        "Expected score to be {}, got {}",
        expected_score.0,
        score.0
    );

    Ok(())
}
