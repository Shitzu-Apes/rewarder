mod view;

use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    near, require,
    serde::{Deserialize, Serialize},
    AccountId, NearToken, PanicOnDefault, Promise, PromiseResult,
};
use primitive_types::U256;

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

#[ext_contract(rewarder)]
#[allow(dead_code)]
trait Rewarder {
    fn primary_nft_of(&mut self, account_id: AccountId) -> Option<(TokenId, U128)>;
    fn on_track_score(&mut self, primary_nft: TokenId, amount: U128);
}

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct FarmConfig {
    pub farm_id: AccountId,
    pub seed_id: SeedId,
    pub factor: u128,
    pub cap: u128,
    pub decimals: u8,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    rewarder: AccountId,
    xref: FarmConfig,
    shitzu: FarmConfig,
    lp: FarmConfig,
    checkpoint: near_sdk::collections::UnorderedMap<AccountId, u64>,
}

#[near]
impl Contract {
    pub fn claim_ref_memeseason(&mut self) -> Promise {
        require!(
            self.checkpoint
                .get(&env::predecessor_account_id())
                .map_or(true, |checkpoint| {
                    env::block_timestamp() - checkpoint > 24 * 60 * 60
                }),
            "Too soon to claim the reward.",
        );

        let primary_nft =
            rewarder::ext(self.rewarder.clone()).primary_nft_of(env::predecessor_account_id());

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

        primary_nft
            .and(xref_staking)
            .and(shitzu_staking)
            .and(lp_staking)
            .then(Self::ext(env::current_account_id()).on_claim_ref_memeseason())
    }

    #[private]
    pub fn on_claim_ref_memeseason(
        &mut self,
        #[callback_unwrap] primary_nft: Option<(TokenId, U128)>,
    ) -> Promise {
        let primary_nft = primary_nft.expect("Primary NFT not found");

        let xref_staking_result = env::promise_result(1);
        let xref_score = match self.parse_promise_result(xref_staking_result) {
            Some(farmer_seed) => {
                // 120k xref currently staked
                // sqrt(120k / 1000) = ~11.0 SHITZU per day
                // 11 * 100 / 100000 = 0.011
                // shitstars = Math.min(sqrt(xref_staking) / 0.011, 200)

                // XRef is 18 decimals
                // amount * 10**18 * (10**24) / (11 * 10**21) = amount * 10**18 / 0.011
                self.internal_calculate_staking_score(
                    farmer_seed.free_amount.0,
                    11000000000000000000000,
                    200,
                    18,
                )
            }
            None => 0,
        };

        let shitzu_staking_result = env::promise_result(2);
        let shitzu_score = match self.parse_promise_result(shitzu_staking_result) {
            Some(farmer_seed) => {
                // SHITZU total supply is 300M, supposed 1000 nft holder has equal share, sqrt(300M / 1000) = ~547 SHITZU per day
                // 547 * 100 / 10000 = 5.47
                // shitstars = Math.min(sqrt(shitzu_staking) / 5.47, 100)

                // SHITZU is 18 decimals
                // amount * 10**18 * (10**24) / (547 * 10**21) = amount * 10**18 / 5.47
                self.internal_calculate_staking_score(
                    farmer_seed.free_amount.0,
                    547000000000000000000000,
                    100,
                    18,
                )
            }
            None => 0,
        };

        let lp_staking_result = env::promise_result(3);
        let lp_score = match self.parse_promise_result(lp_staking_result) {
            Some(farmer_seed) => {
                // Total Supply of LP is 30, so sqrt(30 / 1000) = ~0.17 SHITZU per day
                // 0.17 * 100 / 100000 = 0.00017
                // shitstars = Math.min(sqrt(lp_staking) / 0.00017, 100)

                // LP is 24 decimals
                // amount * 10**24 * (10**24) / (17 * 10**21) = amount * 10**27 / 17
                self.internal_calculate_staking_score(
                    farmer_seed.free_amount.0,
                    17000000000000000000000000,
                    100,
                    24,
                )
            }
            None => 0,
        };

        self.checkpoint
            .insert(&env::predecessor_account_id(), &env::block_timestamp());

        rewarder::ext(self.rewarder.clone())
            .on_track_score(primary_nft.0, U128(xref_score + shitzu_score + lp_score))
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

    fn internal_calculate_staking_score(
        &self,
        amount: u128,
        divisor: u128,
        cap: u128,
        decimals: u8,
    ) -> u128 {
        // amount * 10**decimals * (10**24) / (factor * 10**21) = amount * 10**decimals / factor
        let score = (U256::from(amount).integer_sqrt()
            * U256::from(NearToken::from_near(1).as_yoctonear())
            / U256::from(divisor)
            / U256::exp10((decimals - 18).into()))
        .min(U256::from(cap) * U256::exp10(18));

        score.as_u128()
    }
}
