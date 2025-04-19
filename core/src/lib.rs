#![deny(clippy::str_to_string)]

// re-export
pub use {chrono, sqlparser};

mod mock;
mod result;
mod worm;

pub mod adapter;
pub mod ast;
pub mod ast_builder;
pub mod data;
pub mod executor;
pub mod parse_sql;
pub mod plan;
pub mod store;
pub mod translate;

pub mod prelude {
    pub use crate::{
        ast::DataType,
        data::{Key, Row, Value},
        executor::{execute, Payload, PayloadVariable},
        parse_sql::parse,
        plan::plan,
        result::{Error, Result},
        translate::translate,
        worm::Worm,
    };
}

pub mod error {
    pub use crate::result::*;
}
