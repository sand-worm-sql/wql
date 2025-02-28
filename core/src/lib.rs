#![deny(clippy::str_to_string)]

// re-export
pub use {chrono, sqlparser};

// mod glue;
// mod mock;
mod result;
mod chains_adapter;

pub mod ast;
pub mod ast_builder;
pub mod data;
pub mod parse_sql;
pub mod plan;
pub mod translate;
pub mod executor;

pub mod prelude {
    pub use crate::{
        ast::DataType,
        data::{Key, Value},
        //executor::{execute, Payload, PayloadVariable},
        //glue::Glue,
        parse_sql::parse,
        //plan::plan,
        result::{Error, Result},
        translate::translate,
    };
}

pub mod error {
    pub use crate::result::*;
}
