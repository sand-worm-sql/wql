use {
    crate::{
        data::{Interval, Value},
        chains_adapter::error::ChainAdapterError
    },
    alloy::{
        providers::{Provider, ProviderBuilder},
        transports::http::reqwest::Url,
    },
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    serde::{Deserialize, Serialize},
    std::{cmp::Ordering, fmt::Debug},
    thiserror::Error as ThisError,
};

type Result<T> = std::result::Result<T, ChainAdapterError>;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuiChain {
    SuiDevnet,
    SuiTestnet,
    SuiMainnet,
}

impl SuiChain {
    fn rpc_fallback(&self) -> &str {
        match self {
            SuiChain::SuiDevnet => "https://fullnode.devnet.sui.io:443",
            SuiChain::SuiTestnet => "https://fullnode.testnet.sui.io:443",
            SuiChain::SuiMainnet => "https://fullnode.mainnet.sui.io:443",
        }
    }
}


impl TryFrom<&str> for SuiChain {
    type Error = ChainAdapterError;

    fn try_from(v: &str) -> Result<Self> {
        Ok(match v {
            "sui" => SuiChain::SuiMainnet,
            "sui_testnet" => SuiChain::SuiTestnet,
            "sui_devnet" => SuiChain::SuiDevnet,
            _ => {
                return Err(ChainAdapterError::InvalidChain(v.to_string()));
            },
        })
    }
}