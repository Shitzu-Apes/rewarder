use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    env, ext_contract,
    json_types::U128,
    near, require,
    serde::{Deserialize, Serialize},
    AccountId, NearToken, Promise, PromiseResult,
};
use primitive_types::U256;

use crate::{Contract, ContractExt};

type SeedId = String;

#[derive(Serialize, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct FarmerSeed {
    pub free_amount: U128,
}

#[ext_contract(ref_finance)]
#[allow(dead_code)]
trait BoostFarm {
    fn get_farmer_seed(&self, farmer_id: AccountId, seed_id: SeedId) -> Option<FarmerSeed>;
}

#[near]
impl Contract {
    pub fn claim_ref_memeseason(&mut self) -> Promise {
        require!(
            self.ref_memeseason_reward_checkpoint
                .get(&env::predecessor_account_id())
                .map_or(true, |checkpoint| {
                    env::block_timestamp() - checkpoint > 24 * 60 * 60
                }),
            "Too soon to claim the reward.",
        );

        let primary_nft = self
            .account_to_token_id
            .get(&env::predecessor_account_id())
            .expect("No NFT found for the owner");

        let xref_staking = ref_finance::ext("memefarm-xref-shitzu.ref-labs.near".parse().unwrap())
            .get_farmer_seed(
                env::current_account_id(),
                "xtoken.ref-finance.near".parse().unwrap(),
            );

        let shitzu_staking = ref_finance::ext("meme-farming_011.ref-labs.near".parse().unwrap())
            .get_farmer_seed(
                env::current_account_id(),
                "token.0xshitzu.near".parse().unwrap(),
            );

        let lp_staking = ref_finance::ext("boostfarm.ref-labs.near".parse::<AccountId>().unwrap())
            .get_farmer_seed(
                env::current_account_id(),
                "v2.ref-finance.near@4369".parse().unwrap(),
            );

        xref_staking
            .and(shitzu_staking)
            .and(lp_staking)
            .then(Self::ext(env::current_account_id()).on_claim_ref_memeseason(primary_nft.clone()))
    }

    #[private]
    pub fn on_claim_ref_memeseason(&mut self, primary_nft: TokenId) {
        let xref_staking_result = env::promise_result(0);
        let xref_score = match self.parse_promise_result(xref_staking_result) {
            Some(farmer_seed) => {
                self.internal_calculate_xref_staking_score(farmer_seed.free_amount.0)
            }
            None => 0,
        };

        let shitzu_staking_result = env::promise_result(1);
        let shitzu_score = match self.parse_promise_result(shitzu_staking_result) {
            Some(farmer_seed) => {
                self.internal_calculate_shitzu_staking_score(farmer_seed.free_amount.0)
            }
            None => 0,
        };

        let lp_staking_result = env::promise_result(2);
        let lp_score = match self.parse_promise_result(lp_staking_result) {
            Some(farmer_seed) => {
                self.internal_calculate_lp_staking_score(farmer_seed.free_amount.0)
            }
            None => 0,
        };

        self.ref_memeseason_reward_checkpoint
            .insert(env::predecessor_account_id(), env::block_timestamp());

        self.internal_record_score(primary_nft, xref_score + shitzu_score + lp_score);
    }
}

impl Contract {
    fn parse_promise_result(&self, promise_result: PromiseResult) -> Option<FarmerSeed> {
        match promise_result {
            PromiseResult::Successful(x) => {
                // let result: Option<FarmerSeed> = x.unwrap_json();
                if let Ok(result) = near_sdk::serde_json::from_slice::<FarmerSeed>(&x) {
                    return Some(result);
                } else {
                    None
                }
            }
            PromiseResult::Failed => None,
        }
    }

    fn internal_calculate_xref_staking_score(&self, amount: u128) -> u128 {
        // 120k xref currently staked
        // sqrt(120k / 1000) = ~11.0 SHITZU per day
        // 11 * 100 / 100000 = 0.011
        // shitstars = Math.min(sqrt(xref_staking) / 0.011, 200)

        // XRef is 18 decimals
        // amount * 10**18 * (10**24) / (11 * 10**21) = amount * 10**18 / 0.011
        let score = (U256::from(amount).integer_sqrt() * U256::from(90))
            .min(U256::from(200) * U256::exp10(18));

        score.as_u128()
    }

    fn internal_calculate_shitzu_staking_score(&self, amount: u128) -> u128 {
        // SHITZU total supply is 300M, supposed 1000 nft holder has equal share, sqrt(300M / 1000) = ~547 SHITZU per day
        // 547 * 100 / 10000 = 5.47
        // shitstars = Math.min(sqrt(shitzu_staking) / 5.47, 100)

        // SHITZU is 18 decimals
        // amount * 10**18 * (10**24) / (547 * 10**21) = amount * 10**18 / 5.47
        let score = (U256::from(amount).integer_sqrt()
            * U256::from(NearToken::from_near(1).as_yoctonear())
            / U256::from(NearToken::from_millinear(5470).as_yoctonear()))
        .min(U256::from(100) * U256::exp10(18));

        score.as_u128()
    }

    fn internal_calculate_lp_staking_score(&self, amount: u128) -> u128 {
        // Total Supply of LP is 30, so sqrt(30 / 1000) = ~0.17 SHITZU per day
        // 0.17 * 100 / 100000 = 0.00017
        // shitstars = Math.min(sqrt(lp_staking) / 0.00017, 100)

        // LP is 24 decimals
        // amount * 10**24 * (10**24) / (17 * 10**21) = amount * 10**27 / 17
        let score = (U256::from(amount).integer_sqrt()
            * U256::from(NearToken::from_near(1).as_yoctonear())
            / U256::from(NearToken::from_near(17).as_yoctonear())
            / U256::exp10(6))
        .min(U256::from(100) * U256::exp10(18));

        score.as_u128()
    }
}
