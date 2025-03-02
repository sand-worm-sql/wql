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
pub enum EvmChain {
    Ethereum,
    Sepolia,
    Arbitrum,
    Base,
    Blast,
    Optimism,
    Polygon,
    Mantle,
    Zksync,
    Taiko,
    Celo,
    Avalanche,
    Scroll,
    Bnb,
    Linea,
    Zora,
    Moonbeam,
    Moonriver,
    Ronin,
    Fantom,
    Kava,
    Gnosis,

    // Short-lived Pectra testnet
    Mekong,
}

impl EvmChain {
    fn rpc_fallback(&self) -> &str {
        match self {
            EvmChain::Ethereum => "https://ethereum.drpc.org",
            EvmChain::Sepolia => "https://rpc.ankr.com/eth_sepolia",
            EvmChain::Arbitrum => "https://rpc.ankr.com/arbitrum",
            EvmChain::Base => "https://rpc.ankr.com/base",
            EvmChain::Blast => "https://rpc.ankr.com/blast",
            EvmChain::Optimism => "https://optimism.drpc.org",
            EvmChain::Polygon => "https://polygon.llamarpc.com",
            EvmChain::Mantle => "https://mantle.drpc.org",
            EvmChain::Zksync => "https://mainnet.era.zksync.io",
            EvmChain::Taiko => "https://rpc.taiko.xyz",
            EvmChain::Celo => "https://1rpc.io/celo",
            EvmChain::Avalanche => "https://avalanche.drpc.org",
            EvmChain::Scroll => "https://scroll.drpc.org",
            EvmChain::Bnb => "https://binance.llamarpc.com",
            EvmChain::Linea => "https://rpc.linea.build",
            EvmChain::Zora => "https://zora.drpc.org",
            EvmChain::Moonbeam => "https://moonbeam.drpc.org",
            EvmChain::Moonriver => "https://moonriver.drpc.org",
            EvmChain::Ronin => "https://ronin.drpc.org",
            EvmChain::Fantom => "https://fantom.drpc.org",
            EvmChain::Kava => "https://evm.kava.io",
            EvmChain::Gnosis => "https://gnosis.drpc.org",
            EvmChain::Mekong => "https://rpc.mekong.ethpandaops.io",
        }
    }
}
