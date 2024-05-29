use near_sdk::{json_types::U128, AccountId};
use near_workspaces::Contract;

pub async fn ft_balance_of(contract: &Contract, account_id: &AccountId) -> anyhow::Result<u128> {
    let res = contract
        .call("ft_balance_of")
        .args_json((account_id,))
        .view()
        .await?;

    Ok(res.json::<U128>()?.0)
}
