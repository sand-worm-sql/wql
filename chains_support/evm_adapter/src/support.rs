use crate::chains_adapter::error::ChainAdapterError;
use std::convert::TryInto;

impl EvmChain {
    pub fn is_supported(chain: &str) -> bool {
        matches!(chain.try_into() as Result<EvmChain, ChainAdapterError>, Ok(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        assert!(EvmChain::is_supported("eth"));
        assert!(EvmChain::is_supported("polygon"));
        assert!(EvmChain::is_supported("zksync"));
        assert!(!EvmChain::is_supported("unknown"));
    }
}
