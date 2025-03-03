use {
    super::Build,
    crate::{ast::Statement, result::Result},
};

#[derive(Clone, Debug)]
pub struct ShowChainEntitiesColumnsNode {
    chain_name: String,
    entity_name: String,
}

impl ShowChainEntitiesColumnsNode {
    pub fn new(chain_name: String, entity_name: String) -> Self {
        Self {
            chain_name,
            entity_name,
        }
    }
}

impl Build for ShowChainEntitiesColumnsNode {
    fn build(self) -> Result<Statement> {
        let chain_name = self.chain_name;
        let entity_name = self.entity_name;
        Ok(Statement::ShowChainEntitiesColumns {
            chain_name,
            entity_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_builder::{chain, test, Build};

    #[test]
    fn show_chain_entities_columns() {
        let actual = chain("sui").show_chain_entities_columns("block").build();
        let expected = "SHOW block ENTITY COLUMNS on sui;";
        test(actual, expected);
    }
}
