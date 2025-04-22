use {
    super::JsonStorage,
    wql_core::store::{Index, IndexMut},
};

impl Index for JsonStorage {}
impl IndexMut for JsonStorage {}
