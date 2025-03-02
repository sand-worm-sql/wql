use {
    crate::{
        data::{Interval, Value},
        result::{Error, Result},
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
