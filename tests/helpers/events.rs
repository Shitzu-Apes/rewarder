use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    AccountId,
};
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

pub const KNOWN_EVENT_KINDS: [&str; 4] = [
    "reward_sent",
    "score_recorded",
    "nft_staked",
    "nft_unstaked",
];

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "standard")]
#[serde(rename_all = "kebab-case")]
pub enum ContractEvent {
    Shitzurewarder(ShitzurewarderEvent),
    Nep141(Nep141Event),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ShitzurewarderEvent {
    pub version: String,
    #[serde(flatten)]
    pub event_kind: ShitzurewarderEventKind,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ShitzurewarderEventKind {
    RewardSent(RewardSent),
    ScoreRecorded(ScoreRecorded),
    NftStaked(NftStaked),
    NftUnstaked(NftUnstaked),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardSent {
    pub account_id: AccountId,
    pub amount: U128,
    pub token_id: Option<TokenId>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ScoreRecorded {
    pub token_id: TokenId,
    pub score: U128,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftStaked {
    pub account_id: AccountId,
    pub token_id: TokenId,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftUnstaked {
    pub account_id: AccountId,
    pub token_id: TokenId,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Nep141Event {
    pub version: String,
    #[serde(flatten)]
    pub event_kind: Nep141EventKind,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Nep141EventKind {
    FtTransfer(Vec<FtTransfer>),
    FtMint(Vec<FtMint>),
    FtBurn(Vec<FtBurn>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtTransfer {
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtMint {
    pub owner_id: AccountId,
    pub amount: U128,
    pub memo: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtBurn {
    pub owner_id: AccountId,
    pub amount: U128,
    pub memo: Option<String>,
}

impl Display for ContractEvent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ContractEvent::Shitzurewarder(event) => formatter.write_fmt(format_args!("{}", event)),
            ContractEvent::Nep141(event) => formatter.write_fmt(format_args!("{}", event)),
        }
    }
}

impl Display for ShitzurewarderEvent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match &self.event_kind {
            &ShitzurewarderEventKind::RewardSent(_) => {
                formatter.write_fmt(format_args!("{}: reward_sent", "event".bright_cyan()))?;
            }
            &ShitzurewarderEventKind::ScoreRecorded(_) => {
                formatter.write_fmt(format_args!("{}: score_recorded", "event".bright_cyan()))?;
            }
            ShitzurewarderEventKind::NftStaked(_) => {
                formatter.write_fmt(format_args!("{}: nft_staked", "event".bright_cyan()))?;
            }
            ShitzurewarderEventKind::NftUnstaked(_) => {
                formatter.write_fmt(format_args!("{}: nft_unstaked", "event".bright_cyan()))?;
            }
        }
        formatter.write_fmt(format_args!("\n{}: nep141", "standard".bright_cyan(),))?;
        formatter.write_fmt(format_args!(
            "\n{}: {}",
            "version".bright_cyan(),
            self.version
        ))?;
        match &self.event_kind {
            ShitzurewarderEventKind::RewardSent(data) => {
                formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
            }
            ShitzurewarderEventKind::ScoreRecorded(data) => {
                formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
            }
            ShitzurewarderEventKind::NftStaked(data) => {
                formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
            }
            ShitzurewarderEventKind::NftUnstaked(data) => {
                formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
            }
        }
        Ok(())
    }
}

impl Display for RewardSent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:?}", &self))?;
        Ok(())
    }
}

impl Display for ScoreRecorded {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:?}", &self))?;
        Ok(())
    }
}

impl Display for NftStaked {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:?}", &self))?;
        Ok(())
    }
}

impl Display for NftUnstaked {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{:?}", &self))?;
        Ok(())
    }
}

impl Display for Nep141Event {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match &self.event_kind {
            Nep141EventKind::FtTransfer(_) => {
                formatter.write_fmt(format_args!("{}: ft_transfer", "event".bright_cyan()))?;
            }
            Nep141EventKind::FtMint(_) => {
                formatter.write_fmt(format_args!("{}: ft_mint", "event".bright_cyan()))?;
            }
            Nep141EventKind::FtBurn(_) => {
                formatter.write_fmt(format_args!("{}: ft_burn", "event".bright_cyan()))?;
            }
        }
        formatter.write_fmt(format_args!("\n{}: nep141", "standard".bright_cyan(),))?;
        formatter.write_fmt(format_args!(
            "\n{}: {}",
            "version".bright_cyan(),
            self.version
        ))?;
        match &self.event_kind {
            Nep141EventKind::FtTransfer(datas) => {
                for data in datas {
                    formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
                }
            }
            Nep141EventKind::FtMint(datas) => {
                for data in datas {
                    formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
                }
            }
            Nep141EventKind::FtBurn(datas) => {
                for data in datas {
                    formatter.write_fmt(format_args!("\n{}: {}", "data".bright_cyan(), data))?;
                }
            }
        }
        Ok(())
    }
}

impl Display for FtTransfer {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if let Some(memo) = &self.memo {
            formatter.write_fmt(format_args!(
                "{} --> {} ({}) --> {}",
                self.old_owner_id.bright_blue(),
                self.amount.bright_blue(),
                memo,
                self.new_owner_id.bright_blue(),
            ))?;
        } else {
            formatter.write_fmt(format_args!(
                "{} --> {} --> {}",
                self.old_owner_id.bright_blue(),
                self.amount.bright_blue(),
                self.new_owner_id.bright_blue(),
            ))?;
        }
        Ok(())
    }
}

impl Display for FtMint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if let Some(memo) = &self.memo {
            formatter.write_fmt(format_args!(
                "{} ({}) --> {}",
                self.amount.0.bright_blue(),
                memo,
                self.owner_id.bright_blue(),
            ))?;
        } else {
            formatter.write_fmt(format_args!(
                "{} --> {}",
                self.amount.0.bright_blue(),
                self.owner_id.bright_blue(),
            ))?;
        }
        Ok(())
    }
}

impl Display for FtBurn {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if let Some(memo) = &self.memo {
            formatter.write_fmt(format_args!(
                "{} --> {} ({}) 🔥",
                self.owner_id.bright_blue(),
                self.amount.0.bright_blue(),
                memo,
            ))?;
        } else {
            formatter.write_fmt(format_args!(
                "{} --> {} 🔥",
                self.owner_id.bright_blue(),
                self.amount.0.bright_blue(),
            ))?;
        }
        Ok(())
    }
}
