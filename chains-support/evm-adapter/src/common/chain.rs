use {
    super::config::Config,
    crate::result::{Error, Result},
    alloy::{
        providers::{Provider, ProviderBuilder},
        transports::http::reqwest::Url,
    },
    core::fmt,
    eql_macros::EnumVariants,
    serde::{Deserialize, Serialize},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum EvmChainError {
    #[error("Invalid chain: {0}")]
    InvalidChain(String),
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChainOrRpc {
    Chain(Chain),
    Rpc(Url),
}

impl ChainOrRpc {
    pub fn rpc_url(&self) -> Result<Url> {
        match self {
            Self::Chain(chain) => chain.rpc_url(),
            Self::Rpc(url) => Ok(url.clone()),
        }
    }

    pub async fn to_chain(&self) -> Result<Chain> {
        match self {
            Self::Chain(chain) => Ok(chain.clone()),
            Self::Rpc(rpc) => {
                let provider = ProviderBuilder::new().on_http(rpc.clone());
                let chain_id = provider.get_chain_id().await.map_err(|e| {
                    Error::EvmChainError(EvmChainError::InvalidRpcUrl(e.to_string()))
                })?;
                chain_id.try_into()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, EnumVariants, Serialize, Deserialize)]
pub enum Chain {
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
    Mekong,
}

impl Chain {
    pub fn from_selector(selector: &str) -> Result<Vec<ChainOrRpc>> {
        if selector == "*" {
            Ok(Chain::all_variants()
                .into_iter()
                .map(|c| ChainOrRpc::Chain(c.clone()))
                .collect())
        } else {
            selector
                .split(',')
                .map(str::trim)
                .map(|s| Chain::try_from(s).map(ChainOrRpc::Chain))
                .collect()
        }
    }

    pub fn rpc_url(&self) -> Result<Url> {
        let config = Config::new();

        match config.get_chain_default_rpc(self) {
            Ok(Some(url)) => Ok(url),
            Ok(None) => Url::parse(self.rpc_fallback())
                .map_err(|e| Error::EvmChainError(EvmChainError::InvalidRpcUrl(e.to_string()))),
            Err(_) => Err(Error::EvmChainError(EvmChainError::InvalidChain(
                self.to_string(),
            ))),
        }
    }

    fn rpc_fallback(&self) -> &'static str {
        use Chain::*;
        match self {
            Ethereum => "https://ethereum.drpc.org",
            Sepolia => "https://rpc.ankr.com/eth_sepolia",
            Arbitrum => "https://rpc.ankr.com/arbitrum",
            Base => "https://rpc.ankr.com/base",
            Blast => "https://rpc.ankr.com/blast",
            Optimism => "https://optimism.drpc.org",
            Polygon => "https://polygon.llamarpc.com",
            Mantle => "https://mantle.drpc.org",
            Zksync => "https://mainnet.era.zksync.io",
            Taiko => "https://rpc.taiko.xyz",
            Celo => "https://1rpc.io/celo",
            Avalanche => "https://avalanche.drpc.org",
            Scroll => "https://scroll.drpc.org",
            Bnb => "https://binance.llamarpc.com",
            Linea => "https://rpc.linea.build",
            Zora => "https://zora.drpc.org",
            Moonbeam => "https://moonbeam.drpc.org",
            Moonriver => "https://moonriver.drpc.org",
            Ronin => "https://ronin.drpc.org",
            Fantom => "https://fantom.drpc.org",
            Kava => "https://evm.kava.io",
            Gnosis => "https://gnosis.drpc.org",
            Mekong => "https://rpc.mekong.ethpandaops.io",
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::Ethereum
    }
}

impl TryFrom<&str> for Chain {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        use Chain::*;
        match s {
            "eth" => Ok(Ethereum),
            "sepolia" => Ok(Sepolia),
            "arb" => Ok(Arbitrum),
            "base" => Ok(Base),
            "blast" => Ok(Blast),
            "op" => Ok(Optimism),
            "polygon" => Ok(Polygon),
            "mantle" => Ok(Mantle),
            "zksync" => Ok(Zksync),
            "taiko" => Ok(Taiko),
            "celo" => Ok(Celo),
            "avalanche" => Ok(Avalanche),
            "scroll" => Ok(Scroll),
            "bnb" => Ok(Bnb),
            "linea" => Ok(Linea),
            "zora" => Ok(Zora),
            "moonbeam" => Ok(Moonbeam),
            "moonriver" => Ok(Moonriver),
            "ronin" => Ok(Ronin),
            "fantom" => Ok(Fantom),
            "kava" => Ok(Kava),
            "gnosis" => Ok(Gnosis),
            "mekong" => Ok(Mekong),
            _ => Err(Error::EvmChainError(EvmChainError::InvalidChain(
                s.to_string(),
            ))),
        }
    }
}

impl std::str::FromStr for Chain {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::try_from(s)
    }
}

impl From<&Chain> for u64 {
    fn from(chain: &Chain) -> Self {
        use Chain::*;
        match chain {
            Ethereum => 1,
            Sepolia => 11155111,
            Arbitrum => 42161,
            Base => 8453,
            Blast => 238,
            Optimism => 10,
            Polygon => 137,
            Mantle => 5000,
            Zksync => 324,
            Taiko => 167000,
            Celo => 42220,
            Avalanche => 43114,
            Scroll => 534352,
            Bnb => 56,
            Linea => 59144,
            Zora => 7777777,
            Moonbeam => 1284,
            Moonriver => 1285,
            Ronin => 2020,
            Fantom => 250,
            Kava => 2222,
            Gnosis => 100,
            Mekong => 7078815900,
        }
    }
}

impl TryFrom<u64> for Chain {
    type Error = Error;

    fn try_from(id: u64) -> Result<Self> {
        use Chain::*;
        match id {
            1 => Ok(Ethereum),
            11155111 => Ok(Sepolia),
            42161 => Ok(Arbitrum),
            8453 => Ok(Base),
            238 => Ok(Blast),
            10 => Ok(Optimism),
            137 => Ok(Polygon),
            5000 => Ok(Mantle),
            324 => Ok(Zksync),
            167000 => Ok(Taiko),
            42220 => Ok(Celo),
            43114 => Ok(Avalanche),
            534352 => Ok(Scroll),
            56 => Ok(Bnb),
            59144 => Ok(Linea),
            7777777 => Ok(Zora),
            1284 => Ok(Moonbeam),
            1285 => Ok(Moonriver),
            2020 => Ok(Ronin),
            250 => Ok(Fantom),
            2222 => Ok(Kava),
            100 => Ok(Gnosis),
            7078815900 => Ok(Mekong),
            _ => Err(Error::EvmChainError(EvmChainError::InvalidChain(
                id.to_string(),
            ))),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Chain::*;
        let name = match self {
            Ethereum => "eth",
            Sepolia => "sepolia",
            Arbitrum => "arb",
            Base => "base",
            Blast => "blast",
            Optimism => "op",
            Polygon => "polygon",
            Mantle => "mantle",
            Zksync => "zksync",
            Taiko => "taiko",
            Celo => "celo",
            Avalanche => "avalanche",
            Scroll => "scroll",
            Bnb => "bnb",
            Linea => "linea",
            Zora => "zora",
            Moonbeam => "moonbeam",
            Moonriver => "moonriver",
            Ronin => "ronin",
            Fantom => "fantom",
            Kava => "kava",
            Gnosis => "gnosis",
            Mekong => "mekong",
        };
        write!(f, "{}", name)
    }
}
