#![deny(clippy::str_to_string)]

// re-export
pub use {chrono, sqlparser};

// mod worm;
mod mock;
mod result;

pub mod ast;
pub mod ast_builder;
pub mod chains_adapter;
pub mod data;
pub mod executor;
pub mod parse_sql;
pub mod plan;
pub mod store;
pub mod translate;

pub mod prelude {
    pub use crate::{
        ast::DataType,
        chains_adapter::ChainEnitityResolver,
        data::{Key, Value},
        //executor::{execute, Payload, PayloadVariable},
        //worm::Glue,
        parse_sql::parse,
        plan::plan,
        result::{Error, Result},
        translate::translate,
    };
}

pub mod error {
    pub use crate::result::*;
}
