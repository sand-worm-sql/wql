use {
    super::ens::NameOrAddress,
    alloy::hex::FromHexError,
    eql_macros::EnumVariants,
    serde::{Deserialize, Serialize},
    std::{fmt::Display, str::FromStr},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum AccountError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error(transparent)]
    AccountFieldError(#[from] AccountFieldError),

    #[error(transparent)]
    AccountFilterError(#[from] AccountFilterError),

    #[error("Invalid Hex: {0}")]
    FromHexError(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Account {
    id: Option<Vec<NameOrAddress>>,
    filter: Option<Vec<AccountFilter>>,
    fields: Vec<AccountField>,
}

impl Account {
    pub fn new(
        id: Option<Vec<NameOrAddress>>,
        filter: Option<Vec<AccountFilter>>,
        fields: Vec<AccountField>,
    ) -> Self {
        Self { id, filter, fields }
    }

    pub fn ids(&self) -> Option<&Vec<NameOrAddress>> {
        self.id.as_ref()
    }

    pub fn filter(&self) -> Option<Vec<AccountFilter>> {
        self.filter.clone()
    }

    pub fn fields(&self) -> Vec<AccountField> {
        self.fields.clone()
    }
}

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum AccountFilterError {
    #[error("Unexpected token {0} for account filter")]
    UnexpectedToken(String),

    #[error("Invalid Hex: {0}")]
    FromHexError(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum AccountFilter {
    Address(NameOrAddress),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum AccountField {
    Address,
    Nonce,
    Balance,
    Code,
    Chain,
}

impl Display for AccountField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountField::Address => write!(f, "address"),
            AccountField::Nonce => write!(f, "nonce"),
            AccountField::Balance => write!(f, "balance"),
            AccountField::Code => write!(f, "code"),
            AccountField::Chain => write!(f, "chain"),
        }
    }
}

#[derive(ThisError, Serialize, Debug, PartialEq, Eq)]
pub enum AccountFieldError {
    #[error("Invalid field for entity Account: {0}")]
    InvalidField(String),

    #[error("Invalid Hex: {0}")]
    FromHexError(String),
}

impl TryFrom<&str> for AccountField {
    type Error = AccountFieldError;

    fn try_from(value: &str) -> Result<Self, AccountFieldError> {
        match value {
            "address" => Ok(AccountField::Address),
            "nonce" => Ok(AccountField::Nonce),
            "balance" => Ok(AccountField::Balance),
            "code" => Ok(AccountField::Code),
            "chain" => Ok(AccountField::Chain),
            invalid_field => Err(AccountFieldError::InvalidField(invalid_field.to_string())),
        }
    }
}
