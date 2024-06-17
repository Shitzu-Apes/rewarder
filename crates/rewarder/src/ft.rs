use crate::{Contract, ContractExt};
use near_contract_standards::fungible_token::{
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleTokenCore,
};
use near_sdk::{json_types::U128, near_bindgen, AccountId, PromiseOrValue};

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    #[allow(unused_variables)]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        unimplemented!();
    }

    #[payable]
    #[allow(unused_variables)]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        unimplemented!()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_score.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        U128(
            if let Some(token_id) = self.account_to_token_id.get(&account_id) {
                let score = self.scores.get(token_id).unwrap_or(&0);
                *score
            } else {
                0
            },
        )
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: "Shit Stars".to_string(),
            symbol: "SHITSTARS".to_string(),
            icon: Some("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256' xml:space='preserve' style='fill-rule:evenodd;clip-rule:evenodd;stroke-linejoin:round;stroke-miterlimit:2'%3E%3Cpath d='M128.48 96.54c2.86 4.29 1.27 8.25-.85 12.42a.998.998 0 0 1-.65.53l-6.6 1.93a.989.989 0 0 1-1.25-.82c-.78-6.04.393-11.617 3.52-16.73.213-.353.51-.607.89-.76a9.128 9.128 0 0 0 3.14-2.15c.48-.5 1.013-.567 1.6-.2.867.547 1.453 1.323 1.76 2.33a1.258 1.258 0 0 1-.45 1.36l-.86.64c-.54.407-.623.89-.25 1.45ZM100.71 96.26l1.31-.26a.733.733 0 0 1 .87.79c-.26 2.473-.433 4.933-.52 7.38-.12 3.34-1.8 7.59-5.55 8.21-.447.073-.783-.087-1.01-.48a107.04 107.04 0 0 0-3.69-6c-1.84-2.8-.76-5.59.9-8.11 1.34-2.05 3.32-2.59 5.2-3.9a.59.59 0 0 1 .88.27l.61 1.56a.883.883 0 0 0 1 .54ZM103.39 136.44c-1.713.687-2.89 1.73-3.53 3.13a1.24 1.24 0 0 1-.91.73c-.827.187-1.293-.123-1.4-.93-.18-1.47-.18-2.91.78-4.15 2.6-3.33 6.38-6.36 2.92-11-.97-1.3-1.41-2.79-1.56-4.39-.033-.353.11-.61.43-.77 1.987-.98 4.24-1.473 6.76-1.48 2.74 0 4.44 2.42 5.66 4.46a.507.507 0 0 1-.21.713l-.04.017c-2.51.95-3.91 2.71-5.33 4.96-1.28 2.02.91 3.51 2.16 4.76.267.273.39.6.37.98a8.8 8.8 0 0 0 .94 4.6c.221.437.305.928.24 1.41-.28 1.94-1.2 2.64-2.76 2.1a.902.902 0 0 1-.61-.78c-.14-1.747-1.24-3.187-3.3-4.32a.66.66 0 0 0-.61-.04Z' style='fill:%23030a20;fill-rule:nonzero' transform='matrix(4.93449 0 0 4.93449 -417.878 -445.1)'/%3E%3Cpath d='m9.5.5-.63 1.375L7.5 2.5l1.37.63.63 1.37.625-1.37L11.5 2.5l-1.375-.625M4.5 2 3.25 4.75.5 6l2.75 1.25L4.5 10l1.25-2.75L8.5 6 5.75 4.75M9.5 7.5l-.63 1.37-1.37.63 1.37.625.63 1.375.625-1.375L11.5 9.5l-1.375-.63' style='fill:%2331c891;fill-rule:nonzero' transform='rotate(15.623 -68.736 152.209) scale(4.91391)'/%3E%3Cpath d='m9.5.5-.63 1.375L7.5 2.5l1.37.63.63 1.37.625-1.37L11.5 2.5l-1.375-.625M4.5 2 3.25 4.75.5 6l2.75 1.25L4.5 10l1.25-2.75L8.5 6 5.75 4.75M9.5 7.5l-.63 1.37-1.37.63 1.37.625.63 1.375.625-1.375L11.5 9.5l-1.375-.63' style='fill:%2331c891;fill-rule:nonzero' transform='scale(4.118) rotate(16.721 -2.75 151.836)'/%3E%3C/svg%3E".to_string()),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}
