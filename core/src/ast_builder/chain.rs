use super::{
    show_chain_entities::ShowChainEntitiesNode,
    show_chain_entities_columns::ShowChainEntitiesColumnsNode,
};

#[derive(Clone, Debug)]
pub struct ChainNode {
    pub chain_name: String,
}

impl ChainNode {
    pub fn new(chain_name: &str) -> Self {
        Self {
            chain_name: chain_name.to_owned(),
        }
    }

    pub fn show_chain_entities(self) -> ShowChainEntitiesNode {
        ShowChainEntitiesNode::new(self.chain_name)
    }

    pub fn show_chain_entities_columns(self, entity_name: &str) -> ShowChainEntitiesColumnsNode {
        ShowChainEntitiesColumnsNode::new(self.chain_name, entity_name.to_owned())
    }

    pub fn select(self) -> SelectNode<'a> {
        let table_factor = TableFactorNode {
            table_name: self.table_name,
            table_type: TableType::Table,
            table_alias: None,
            index: None,
        };

        SelectNode::new(table_factor)
    }
}

pub fn chain(chain_name: &str) -> ChainNode {
    let chain_name = chain_name.to_owned();

    ChainNode { chain_name }
}
