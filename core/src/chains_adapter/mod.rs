mod error;
// mod evm;
// mod sui;

pub use error::ChainAdapterError;

type Result<T> = std::result::Result<T, ChainAdapterError>;

pub enum ChainAdapter {
    // Evm(evm::EvmChain),
    // Sui(sui::SuiChain),
}

impl TryFrom<&str> for ChainAdapter {
    type Error = ChainAdapterError;

    fn try_from(v: &str) -> Result<Self> {
        // if let Ok(evm_chain) = evm::EvmChain::try_from(v) {
        //     return Ok(ChainAdapter::Evm(evm_chain));
        // }

        // if let Ok(sui_chain) = sui::SuiChain::try_from(v) {
        //     return Ok(ChainAdapter::Sui(sui_chain));
        // }

        Err(ChainAdapterError::InvalidChain(v.to_string()))
    }
}
