use {
    super::ParquetStorage,
    wql_core::store::{Index, IndexMut},
};

impl Index for ParquetStorage {}
impl IndexMut for ParquetStorage {}
