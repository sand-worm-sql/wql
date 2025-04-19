use {serde::Serialize, std::fmt::Debug, thiserror::Error};

#[derive(Error, Serialize, Debug, PartialEq, Eq)]
pub enum AdapterError {
    #[error("failed to use adapter: {0}")]
    FailedLinkAdapter(String),
}
