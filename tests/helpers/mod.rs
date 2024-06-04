use anyhow::Ok;
use near_sdk::{json_types::U128, NearToken};
use near_workspaces::result::{ExecutionFinalResult, ExecutionResult, Value};
use owo_colors::OwoColorize;

pub mod call;
pub mod setup;
pub mod view;

pub fn log_tx_result(
    ident: &str,
    res: ExecutionFinalResult,
) -> anyhow::Result<ExecutionResult<Value>> {
    for failure in res.receipt_failures() {
        println!("{:#?}", failure.bright_red());
    }
    for outcome in res.receipt_outcomes() {
        if !outcome.logs.is_empty() {
            for log in outcome.logs.iter() {
                println!("{}", log.bright_yellow());
            }
        }
    }
    println!(
        "{} gas burnt: {:.3} {}",
        ident.italic(),
        res.total_gas_burnt.as_tgas().bright_magenta().bold(),
        "TGas".bright_magenta().bold()
    );
    Ok(res.into_result()?)
}

pub struct Ether(pub u128);

impl From<u128> for Ether {
    fn from(value: u128) -> Self {
        Self(value * 1_000_000_000_000_000_000)
    }
}

impl Into<U128> for Ether {
    fn into(self) -> U128 {
        U128(self.0)
    }
}
