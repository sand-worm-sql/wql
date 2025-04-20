use {serde::Serialize, std::fmt::Debug, thiserror::Error as ThisError};

pub use crate::{
 
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum Error {
 
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
