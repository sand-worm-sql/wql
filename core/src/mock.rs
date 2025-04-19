use {
    crate::{
        data::{Key, Schema},
        result::{Error, Result},
        store::{DataRow, Metadata, RowIter, Store, StoreMut},
    },
    async_trait::async_trait,
    std::collections::HashMap,
};

#[cfg(test)]
use {
    crate::{parse_sql::parse, translate::translate},
    futures::executor::block_on,
};

#[cfg(test)]
pub fn run(sql: &str) -> MockStorage {
    let mut storage = MockStorage::default();

    for parsed in parse(sql).unwrap() {
        let statement = translate(&parsed).unwrap();

        //block_on(execute(&mut storage, &statement)).unwrap();
    }

    storage
}

#[derive(Default, Debug)]
pub struct MockStorage {
    schema_map: HashMap<String, Schema>,
}

impl Metadata for MockStorage {}

#[async_trait(?Send)]
impl Store for MockStorage {
    async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        if table_name == "__Err__" {
            return Err(Error::StorageMsg(
                "[MockStorage] fetch_schema - user triggered error".to_owned(),
            ));
        }

        self.schema_map
            .get(table_name)
            .map(|schema| Ok(schema.clone()))
            .transpose()
    }

    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
        let msg = "[Storage] fetch_all_schemas not supported".to_owned();

        Err(Error::StorageMsg(msg))
    }

    async fn fetch_data(&self, _table_name: &str, _key: &Key) -> Result<Option<DataRow>> {
        Err(Error::StorageMsg(
            "[MockStorage] fetch_data not supported".to_owned(),
        ))
    }

    async fn scan_data(&self, _table_name: &str) -> Result<RowIter<'_>> {
        Err(Error::StorageMsg(
            "[MockStorage] scan_data not supported".to_owned(),
        ))
    }
}

#[async_trait(?Send)]
impl StoreMut for MockStorage {
    async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
        let table_name = schema.table_name.clone();
        let schema = schema.clone();

        self.schema_map.insert(table_name, schema);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::MockStorage,
        crate::{
            data::Key,
            store::{Metadata, Store, StoreMut},
        },
        futures::executor::block_on,
    };

    #[test]
    fn empty() {
        let storage = MockStorage::default();

        assert!(block_on(storage.scan_data("Foo")).is_err());
        assert!(block_on(storage.fetch_data("Foo", &Key::None)).is_err());
        assert!(block_on(storage.fetch_schema("__Err__")).is_err());

        assert!(matches!(block_on(storage.fetch_schema("Foo")), Ok(None)));
    }
}
