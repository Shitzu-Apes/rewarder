mod view;

use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    borsh::BorshSerialize,
    collections::LookupMap,
    env, ext_contract,
    json_types::U128,
    near, require,
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseResult,
};
use primitive_types::U256;

#[cfg(not(feature = "integration-test"))]
pub const INTERVAL: u64 = 60 * 60 * 16 * 1_000_000_000;
#[cfg(feature = "integration-test")]
pub const INTERVAL: u64 = 10 * 1_000_000_000;

type SeedId = String;

#[derive(Default)]
#[near(serializers = [json])]
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

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct FarmConfig {
    pub farm_id: AccountId,
    pub seed_id: SeedId,
    pub factor: U128,
    pub base: U128,
    pub cap: U128,
    pub decimals: u8,
}

#[near(serializers = [json])]
pub struct FarmConfigs {
    pub xref: FarmConfig,
    pub shitzu: FarmConfig,
    pub lp: FarmConfig,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    rewarder: AccountId,
    xref: FarmConfig,
    shitzu: FarmConfig,
    lp: FarmConfig,
    checkpoint: LookupMap<AccountId, u64>,
}

#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Checkpoint,
}

#[near]
impl Contract {
    #[init]
    pub fn new(rewarder: AccountId, xref: FarmConfig, shitzu: FarmConfig, lp: FarmConfig) -> Self {
        Self {
            rewarder,
            xref,
            shitzu,
            lp,
            checkpoint: LookupMap::new(StorageKey::Checkpoint),
        }
    }

    #[private]
    pub fn change_farm_configs(&mut self, xref: FarmConfig, shitzu: FarmConfig, lp: FarmConfig) {
        self.xref = xref;
        self.shitzu = shitzu;
        self.lp = lp;
    }

    pub fn claim_ref_memeseason(&mut self) -> Promise {
        require!(
            self.checkpoint
                .get(&env::predecessor_account_id())
                .is_none_or(|checkpoint| {
                    env::block_timestamp() - checkpoint > INTERVAL
                }),
            "Too soon to claim the reward.",
        );

        let primary_nft =
            rewarder::ext(self.rewarder.clone()).primary_nft_of(env::predecessor_account_id());

        let xref_staking = ref_finance::ext(self.xref.farm_id.clone())
            .get_farmer_seed(env::predecessor_account_id(), self.xref.seed_id.clone());

        let shitzu_staking = ref_finance::ext(self.shitzu.farm_id.clone())
            .get_farmer_seed(env::predecessor_account_id(), self.shitzu.seed_id.clone());

        let lp_staking = ref_finance::ext(self.lp.farm_id.clone())
            .get_farmer_seed(env::predecessor_account_id(), self.lp.seed_id.clone());

        primary_nft
            .and(xref_staking)
            .and(shitzu_staking)
            .and(lp_staking)
            .then(
                Self::ext(env::current_account_id())
                    .on_claim_ref_memeseason(env::predecessor_account_id()),
            )
    }

    #[private]
    pub fn on_claim_ref_memeseason(
        &mut self,
        claimer: AccountId,
        #[callback_unwrap] primary_nft: Option<(TokenId, U128)>,
    ) -> Promise {
        let primary_nft = primary_nft.expect("Primary NFT not found");

        let xref_score = 0;
        let shitzu_score = 0;

        let lp_staking_result = env::promise_result(3);
        let lp_score = match self.parse_promise_result(lp_staking_result) {
            Some(farmer_seed) => {
                // Total Supply of LP is 30, so sqrt(30 / 1000) = ~0.17 SHITZU per interval
                // 0.17 * 100 / 100000 = 0.00017
                // shitstars = Math.min(sqrt(lp_staking) / 0.00017, 100)

                // LP is 24 decimals
                // amount * 10**24 * (10**24) / (17 * 10**21) = amount * 10**27 / 17
                self.internal_calculate_staking_score(farmer_seed.free_amount.0, &self.lp)
            }
            None => 0,
        };

        self.checkpoint.insert(&claimer, &env::block_timestamp());

        rewarder::ext(self.rewarder.clone())
            .on_track_score(primary_nft.0, U128(xref_score + shitzu_score + lp_score))
    }
}

impl Contract {
    fn parse_promise_result(&self, promise_result: PromiseResult) -> Option<FarmerSeed> {
        match promise_result {
            PromiseResult::Successful(x) => {
                near_sdk::serde_json::from_slice::<FarmerSeed>(&x).ok()
            }
            PromiseResult::Failed => None,
        }
    }

    fn internal_calculate_staking_score(&self, amount: u128, config: &FarmConfig) -> u128 {
        // amount * 10**decimals * (10**24) / (factor * 10**21) = amount * 10**decimals / factor
        let FarmConfig {
            factor,
            base,
            cap,
            decimals,
            ..
        } = config;
        let amount = U256::from(amount) * U256::exp10(decimals.to_owned().into()); // since we are about to square root it we need to multiply it by 10^decimals
        let mut score = amount.integer_sqrt() // square root of amount * 10**(decimals*2) is sqrt(amount) * 10**decimals
        * U256::from(NearToken::from_near(1).as_yoctonear())
            / U256::from(factor.0)
            / U256::exp10((decimals - 18).into());

        score = U256::from(base.0) + score;
        score = score.min(U256::from(cap.0));

        score.as_u128()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn test_internal_calculate_staking_score() {
        // Setup context
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let xref_config = FarmConfig {
            farm_id: accounts(2),
            seed_id: "seed1".to_string(),
            factor: U128(1000000000000000000000000), // 1 * 10^24
            base: U128(100000000000000000000),       // 100 * 10^18
            cap: U128(200000000000000000000),        // 200 * 10^18
            decimals: 18,
        };
        let shitzu_config = FarmConfig {
            farm_id: accounts(3),
            seed_id: "seed2".to_string(),
            factor: U128(5000000000000000000000000), // 5 * 10^24
            base: U128(50000000000000000000),        // 50 * 10^18
            cap: U128(100000000000000000000),        // 100 * 10^18
            decimals: 18,
        };
        let lp_config = FarmConfig {
            farm_id: accounts(4),
            seed_id: "seed3".to_string(),
            factor: U128(10000000000000000000000), // 0.01 * 10^24
            base: U128(50000000000000000000),      // 50 * 10^18
            cap: U128(100000000000000000000),      // 100 * 10^18
            decimals: 24,
        };

        // Create a dummy contract instance
        let contract = Contract::new(
            accounts(1),
            xref_config.clone(),
            shitzu_config.clone(),
            lp_config.clone(),
        );

        // Test cases
        let test_cases = vec![
            (
                100_000000000000000000,
                &xref_config,
                110_000000000000000000, // 100 base + 10 extra
            ),
            (
                0,
                &xref_config,
                100_000000000000000000, // base
            ),
            (
                10_000_000_000_000_000_000_000,
                &shitzu_config,
                70_000000000000000000, // 50 base + 40 extra
            ),
            (0, &lp_config, 50_000000000000000000),
            (
                90000000000000000000000, // 0.09, sqrt is 0.3, divided by 0.01 is 30
                &lp_config,
                80_000000000000000000, // cap
            ),
            (
                1_000_000_000_000_000_000_000_000,
                &lp_config,
                100_000000000000000000, // cap
            ),
        ];

        for (amount, config, expected_score) in test_cases {
            let score = contract.internal_calculate_staking_score(amount, config);
            assert_eq!(score, expected_score);
        }
    }
}
