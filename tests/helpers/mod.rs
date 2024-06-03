use anyhow::Ok;
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
