use anyhow::Ok;
use near_workspaces::result::ExecutionFinalResult;

pub mod call;
pub mod setup;
pub mod view;

pub fn log_tx_result(ident: &str, res: ExecutionFinalResult) -> anyhow::Result<()> {
    Ok(())
}
