use super::{
    chain_factor::ChainQueryType, show_chain_entities::ShowChainEntitiesNode,
    show_chain_entities_columns::ShowChainEntitiesColumnsNode, ChainFactorNode,
    SelectNode,
};

#[derive(Clone, Debug)]
pub struct ChainNode {
    pub chain_name: String,
}

impl<'a> ChainNode {
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

    pub fn alias_as(self, entity_name: &str, table_alias: &str) -> ChainFactorNode<'a> {
        ChainFactorNode {
            chain_name: self.chain_name,
            chain_query_type: ChainQueryType::Table,
            table_alias: Some(table_alias.to_owned()),
            entity_name: Some(entity_name.to_owned()),
            index: None,
        }
    }

    /// Construct a `ChainFactorNode` referring to the given entity.
    ///
    /// For example, `chain("my_chain").entity("my_entity")` will generate a
    /// `ChainFactorNode` referring to the entity `my_entity` in the chain
    /// `my_chain`.
    pub fn entity(self, entity_name: &str) -> ChainFactorNode {
        ChainFactorNode {
            chain_name: self.chain_name,
            chain_query_type: ChainQueryType::Table,
            table_alias: None,
            entity_name: Some(entity_name.to_owned()),
            index: None,
        }
    }

    pub fn select(self, entity_name: &str) -> SelectNode<'a> {
        let chain_factor = ChainFactorNode {
            chain_name: self.chain_name,
            chain_query_type: ChainQueryType::Table,
            entity_name: Some(entity_name.to_owned()),
            table_alias: None,
            index: None,
        };

        SelectNode::new(chain_factor)
    }
}

pub fn chain(chain_name: &str) -> ChainNode {
    let chain_name = chain_name.to_owned();

    ChainNode { chain_name }
}
