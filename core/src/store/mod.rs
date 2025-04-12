mod data_row;
mod function;
mod index;
mod metadata;
mod transaction;

pub trait GStore: Store + Index + Metadata + CustomFunction {}
impl<S: Store + Index + Metadata + CustomFunction> GStore for S {}

pub trait GStoreMut:
     IndexMut + Transaction + CustomFunction + CustomFunctionMut
{
}
impl<S: IndexMut + Transaction + CustomFunction + CustomFunctionMut>
    GStoreMut for S
{
}

pub use {
    data_row::DataRow,
    function::{CustomFunction, CustomFunctionMut},
    index::{Index, IndexError, IndexMut},
    metadata::{MetaIter, Metadata},
    transaction::Transaction,
};

use {
    crate::{
        data::{Key, Schema},
        result::{Error, Result},
    },
    async_trait::async_trait,
    futures::stream::Stream,
    std::pin::Pin,
};

pub type RowIter<'a> = Pin<Box<dyn Stream<Item = Result<(Key, DataRow)>> + 'a>>;

/// By implementing `Store` trait, you can run `SELECT` query.
#[async_trait(?Send)]
pub trait Store {
    async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>>;

    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>>;

    async fn fetch_data(&self, table_name: &str, key: &Key) -> Result<Option<DataRow>>;

    async fn scan_data(&self, table_name: &str) -> Result<RowIter<'_>>;
}

