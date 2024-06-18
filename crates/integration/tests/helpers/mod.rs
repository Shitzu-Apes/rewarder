pub mod call;
pub mod events;
pub mod setup;
pub mod view;

use events::{ContractEvent, FtBurn, FtMint, ShitzurewarderEventKind, KNOWN_EVENT_KINDS};
use near_sdk::{json_types::U128, serde::Serialize};
use near_workspaces::result::{ExecutionFinalResult, ExecutionResult, Value};
use owo_colors::OwoColorize;
use serde_json::json;
use std::fmt;

pub fn log_tx_result(
    ident: &str,
    res: ExecutionFinalResult,
) -> anyhow::Result<(ExecutionResult<Value>, Vec<ContractEvent>)> {
    for failure in res.receipt_failures() {
        println!("{:#?}", failure.bright_red());
    }
    let mut events = vec![];
    for outcome in res.receipt_outcomes() {
        if !outcome.logs.is_empty() {
            for log in outcome.logs.iter() {
                if log.starts_with("EVENT_JSON:") {
                    if let Ok(event) =
                        serde_json::from_str::<ContractEvent>(&log.replace("EVENT_JSON:", ""))
                    {
                        events.push(event.clone());
                        println!(
                            "{}: {}\n{}",
                            "account".bright_cyan(),
                            outcome.executor_id,
                            event
                        );
                    }
                } else {
                    println!("{}", log.bright_yellow());
                }
            }
        }
    }
    println!(
        "{} gas burnt: {:.3} {}",
        ident.italic(),
        res.total_gas_burnt.as_tgas().bright_magenta().bold(),
        "TGas".bright_magenta().bold()
    );
    Ok((res.into_result()?, events))
}

pub fn assert_event_emits<T>(actual: &T, events: Vec<ShitzurewarderEventKind>) -> anyhow::Result<()>
where
    T: Serialize + fmt::Debug + Clone,
{
    let mut actual = serde_json::to_value(actual)?;
    actual.as_array_mut().unwrap().retain(|ac| {
        let event_str = ac
            .as_object()
            .unwrap()
            .get("event")
            .unwrap()
            .as_str()
            .unwrap();
        KNOWN_EVENT_KINDS.contains(&event_str)
    });
    let mut expected = vec![];
    for event in events {
        let mut expected_event = serde_json::to_value(event)?;
        let ev = expected_event.as_object_mut().unwrap();
        ev.insert("standard".into(), "shitzurewarder".into());
        ev.insert("version".into(), "1.0.0".into());
        expected.push(expected_event);
    }
    assert_eq!(
        &actual,
        &serde_json::to_value(&expected)?,
        "actual and expected events did not match.\nActual: {:#?}\nExpected: {:#?}",
        &actual,
        &expected
    );
    Ok(())
}

pub fn assert_ft_mint_events<T>(actual: &T, events: Vec<FtMint>) -> anyhow::Result<()>
where
    T: Serialize + fmt::Debug + Clone,
{
    let mut actual = serde_json::to_value(actual)?;
    actual.as_array_mut().unwrap().retain(|ac| {
        let event_str = ac
            .as_object()
            .unwrap()
            .get("event")
            .unwrap()
            .as_str()
            .unwrap();
        event_str == "ft_mint"
    });
    let mut expected = vec![];
    for event in events {
        expected.push(json!({
            "event": "ft_mint",
            "standard": "nep141",
            "version": "1.0.0",
            "data": [event]
        }));
    }
    assert_eq!(
        &actual,
        &serde_json::to_value(&expected)?,
        "actual and expected events did not match.\nActual: {:#?}\nExpected: {:#?}",
        &actual,
        &expected
    );
    Ok(())
}

pub fn assert_ft_burn_events<T>(actual: &T, events: Vec<FtBurn>) -> anyhow::Result<()>
where
    T: Serialize + fmt::Debug + Clone,
{
    let mut actual = serde_json::to_value(actual)?;
    actual.as_array_mut().unwrap().retain(|ac| {
        let event_str = ac
            .as_object()
            .unwrap()
            .get("event")
            .unwrap()
            .as_str()
            .unwrap();
        event_str == "ft_burn"
    });
    let mut expected = vec![];
    for event in events {
        expected.push(json!({
            "event": "ft_burn",
            "standard": "nep141",
            "version": "1.0.0",
            "data": [event]
        }));
    }
    assert_eq!(
        &actual,
        &serde_json::to_value(&expected)?,
        "actual and expected events did not match.\nActual: {:#?}\nExpected: {:#?}",
        &actual,
        &expected
    );
    Ok(())
}

pub struct Ether(pub u128);

impl From<u128> for Ether {
    fn from(value: u128) -> Self {
        Self(value * 1_000_000_000_000_000_000)
    }
}

impl From<Ether> for U128 {
    fn from(value: Ether) -> Self {
        Self(value.0)
    }
}

pub fn assert_approx_eq(
    actual: U128,
    expected: U128,
    perc: u128, // 2 decimal places e.g. 1000 = 10.00%
    message: &str,
) -> anyhow::Result<()> {
    let actual = actual.0;
    let expected = expected.0;
    let perc = perc as f64 / 10000.0;
    let diff = (actual as f64 - expected as f64).abs();
    let avg = (actual as f64 + expected as f64) / 2.0;
    let diff_perc = diff / avg;
    if diff_perc > perc {
        anyhow::bail!(
            "{}\nExpected: {}\nActual: {}\nDiff: {}\nDiff %: {:.2}%",
            message,
            expected,
            actual,
            diff,
            diff_perc * 100.0
        );
    }
    Ok(())
}
