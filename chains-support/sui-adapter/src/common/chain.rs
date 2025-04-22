use {
    crate::result::{Error, Result},
    alloy::transports::http::reqwest::Url,
    serde::{Deserialize, Serialize},
    std::fmt,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum SuiChainError {
    #[error("Invalid Sui chain: {0}")]
    InvalidChain(String),
    #[error("Invalid Sui RPC URL: {0}")]
    InvalidRpcUrl(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SuiChain {
    Mainnet,
    Testnet,
    Devnet,
}

impl SuiChain {
    pub fn rpc_url(&self) -> Result<Url> {
        let url = match self {
            SuiChain::Mainnet => "https://fullnode.mainnet.sui.io:443",
            SuiChain::Testnet => "https://fullnode.testnet.sui.io:443",
            SuiChain::Devnet => "https://fullnode.devnet.sui.io:443",
        };
        Url::parse(url).map_err(|e| Error::SuiChainError(SuiChainError::InvalidRpcUrl(e.to_string())))
    }
}

impl TryFrom<&str> for SuiChain {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "mainnet" => Ok(SuiChain::Mainnet),
            "testnet" => Ok(SuiChain::Testnet),
            "devnet" => Ok(SuiChain::Devnet),
            _ => Err(Error::SuiChainError(SuiChainError::InvalidChain(value.to_string()))),
        }
    }
}

impl fmt::Display for SuiChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SuiChain::Mainnet => "mainnet",
            SuiChain::Testnet => "testnet",
            SuiChain::Devnet => "devnet",
        };
        write!(f, "{}", name)
    }
}
