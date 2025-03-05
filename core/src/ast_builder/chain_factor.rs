use {
    super::{ExprNode, IndexItemNode, QueryNode, SelectNode},
    crate::ast::Dictionary,
};

#[derive(Clone, Debug)]
pub enum ChainType<'a> {
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
    pub chain_type: ChainType<'a>,
    pub chain_alias: Option<String>,
    pub index: Option<IndexItemNode<'a>>,
}

impl<'a> ChainFactorNode<'a> {
    pub fn select(self) -> SelectNode<'a> {
        SelectNode::new(self)
    }
}

// pub fn glue_objects() -> TableFactorNode<'static> {
//     TableFactorNode {
//         table_name: "GLUE_OBJECTS".to_owned(),
//         table_type: TableType::Dictionary(Dictionary::GlueObjects),
//         table_alias: None,
//         index: None,
//     }
// }

// pub fn glue_tables() -> TableFactorNode<'static> {
//     TableFactorNode {
//         table_name: "GLUE_TABLES".to_owned(),
//         table_type: TableType::Dictionary(Dictionary::GlueTables),
//         table_alias: None,
//         index: None,
//     }
// }

// pub fn glue_indexes() -> TableFactorNode<'static> {
//     TableFactorNode {
//         table_name: "GLUE_INDEXES".to_owned(),
//         table_type: TableType::Dictionary(Dictionary::GlueIndexes),
//         table_alias: None,
//         index: None,
//     }
// }

// pub fn glue_table_columns() -> TableFactorNode<'static> {
//     TableFactorNode {
//         table_name: "GLUE_TABLE_COLUMNS".to_owned(),
//         table_type: TableType::Dictionary(Dictionary::GlueTableColumns),
//         table_alias: None,
//         index: None,
//     }
// }

// pub fn series<'a, T: Into<ExprNode<'a>>>(args: T) -> TableFactorNode<'a> {
//     TableFactorNode {
//         table_name: "SERIES".to_owned(),
//         table_type: TableType::Series(args.into()),
//         table_alias: None,
//         index: None,
//     }
// }
