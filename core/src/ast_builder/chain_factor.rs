use {
    super::{ExprNode, IndexItemNode, QueryNode, SelectNode},
    crate::ast::Dictionary,
};

#[derive(Clone, Debug)]
pub enum ChainQueryType<'a> {
    Table,
    Series(ExprNode<'a>),
    Dictionary(Dictionary),
    Derived {
        subquery: Box<QueryNode<'a>>,
        alias: String,
    },
}

#[derive(Clone, Debug)]
pub struct ChainFactorNode<'a> {
    pub chain_name: String,
    pub chain_query_type: ChainQueryType<'a>,
    pub table_alias: Option<String>,
    pub entity_name: Option<String>,
    pub index: Option<IndexItemNode<'a>>,
}

impl<'a> ChainFactorNode<'a> {
    pub fn select(self) -> SelectNode<'a> {
        SelectNode::new(self)
    }
}

pub fn chain_query_objects() -> ChainFactorNode<'static> {
    ChainFactorNode {
        chain_name: "chain".to_owned(),
        chain_query_type: ChainQueryType::Dictionary(Dictionary::GlueObjects),
        entity_name: Some("enitity_objects".to_owned()),
        table_alias: None,
        index: None,
    }
}

pub fn chain_tables() -> ChainFactorNode<'static> {
    ChainFactorNode {
        chain_name: "chain".to_owned(),
        chain_query_type: ChainQueryType::Dictionary(Dictionary::GlueTables),
        entity_name: Some("enitity_tables".to_owned()),
        table_alias: None,
        index: None,
    }
}

pub fn chain_table_indexes() -> ChainFactorNode<'static> {
    ChainFactorNode {
        chain_name: "chain".to_owned(),
        chain_query_type: ChainQueryType::Dictionary(Dictionary::GlueIndexes),
        entity_name: Some("enitity_indexes".to_owned()),
        table_alias: None,
        index: None,
    }
}

pub fn chain_table_columns() -> ChainFactorNode<'static> {
    ChainFactorNode {
        chain_name: "chain".to_owned(),
        chain_query_type: ChainQueryType::Dictionary(Dictionary::GlueTableColumns),
        entity_name: Some("enitity_columns".to_owned()),
        table_alias: None,
        index: None,
    }
}

pub fn series<'a, T: Into<ExprNode<'a>>>(args: T) -> ChainFactorNode<'a> {
    ChainFactorNode {
        chain_name: "SERIES".to_owned(),
        chain_query_type: ChainQueryType::Series(args.into()),
        entity_name: None,
        table_alias: None,
        index: None,
    }
}
