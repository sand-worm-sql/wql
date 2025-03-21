use {
    super::Build,
    crate::{ast::Statement, result::Result},
};

#[derive(Clone, Debug)]
pub struct ShowChainEntitiesNode {
    chain_name: String,
}

impl ShowChainEntitiesNode {
    pub fn new(chain_name: String) -> Self {
        Self { chain_name }
    }
}

impl Build for ShowChainEntitiesNode {
    fn build(self) -> Result<Statement> {
        let chain_name = self.chain_name;
        Ok(Statement::ShowChainEntities { chain_name })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_builder::{chain, test, Build};

    #[test]
    fn show_columns() {
        let actual = chain("base").show_chain_entities().build();
        let expected = "SHOW CHAIN ENTITIES FROM base";
        test(actual, expected);
    }
}
