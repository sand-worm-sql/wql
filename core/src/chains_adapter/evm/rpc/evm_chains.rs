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
    std,
    thiserror::Error as ThisError,
};


type Result<T> = std::result::Result<T, ChainAdapterError>;

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


impl TryFrom<&str> for EvmChain {
    type Error = ChainAdapterError;

    fn try_from(v: &str) -> Result<Self> {
        Ok(match v {
            "eth" => EvmChain::Ethereum,
            "sepolia" => EvmChain::Sepolia,
            "arb" => EvmChain::Arbitrum,
            "base" => EvmChain::Base,
            "blast" => EvmChain::Blast,
            "op" => EvmChain::Optimism,
            "polygon" => EvmChain::Polygon,
            "mantle" => EvmChain::Mantle,
            "zksync" => EvmChain::Zksync,
            "taiko" => EvmChain::Taiko,
            "celo" => EvmChain::Celo,
            "avalanche" => EvmChain::Avalanche,
            "scroll" => EvmChain::Scroll,
            "bnb" => EvmChain::Bnb,
            "linea" => EvmChain::Linea,
            "zora" => EvmChain::Zora,
            "moonbeam" => EvmChain::Moonbeam,
            "moonriver" => EvmChain::Moonriver,
            "ronin" => EvmChain::Ronin,
            "fantom" => EvmChain::Fantom,
            "kava" => EvmChain::Kava,
            "gnosis" => EvmChain::Gnosis,
            "mekong" => EvmChain::Mekong,
            _ => {
                return Err(ChainAdapterError::InvalidChain(v.to_string()));
            },
        })
    }
}


impl From<&EvmChain> for u64 {
    fn from(v: &EvmChain) -> Self {
        match v {
            EvmChain::Ethereum => 1,
            EvmChain::Sepolia => 11155111,
            EvmChain::Arbitrum => 42161,
            EvmChain::Base => 8453,
            EvmChain::Blast => 238,
            EvmChain::Optimism => 10,
            EvmChain::Polygon => 137,
            EvmChain::Mantle => 5000,
            EvmChain::Zksync => 324,
            EvmChain::Taiko => 167000,
            EvmChain::Celo => 42220,
            EvmChain::Avalanche => 43114,
            EvmChain::Scroll => 534352,
            EvmChain::Bnb => 56,
            EvmChain::Linea => 59144,
            EvmChain::Zora => 7777777,
            EvmChain::Moonbeam => 1284,
            EvmChain::Moonriver => 1285,
            EvmChain::Ronin => 2020,
            EvmChain::Fantom => 250,
            EvmChain::Kava => 2222,
            EvmChain::Gnosis => 100,
            EvmChain::Mekong => 7078815900,
        }
    }
}

impl TryFrom<u64> for EvmChain {
    type Error = ChainAdapterError;

    fn try_from(chain_id: u64) -> Result<Self> {
        Ok(
        match chain_id {
            1 => EvmChain::Ethereum,
            11155111 => EvmChain::Sepolia,
            42161 => EvmChain::Arbitrum,
            8453 => EvmChain::Base,
            238 => EvmChain::Blast,
            10 => EvmChain::Optimism,
            137 => EvmChain::Polygon,
            5000 => EvmChain::Mantle,
            324 => EvmChain::Zksync,
            167000 => EvmChain::Taiko,
            42220 => EvmChain::Celo,
            43114 => EvmChain::Avalanche,
            534352 => EvmChain::Scroll,
            56 => EvmChain::Bnb,
            59144 => EvmChain::Linea,
            7777777 => EvmChain::Zora,
            1284 => EvmChain::Moonbeam,
            1285 => EvmChain::Moonriver,
            2020 => EvmChain::Ronin,
            250 => EvmChain::Fantom,
            2222 => EvmChain::Kava,
            100 => EvmChain::Gnosis,
            _ => {
               return  Err(ChainAdapterError::InvalidChain(chain_id.to_string()));
            }
        })
    }
}