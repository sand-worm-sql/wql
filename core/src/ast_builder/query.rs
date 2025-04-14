use {
    super::{
        chain_factor::ChainQueryType,
        select::{Prebuild, ValuesNode},
        ChainFactorNode, ExprList, FilterNode, GroupByNode, HashJoinNode, HavingNode,
        JoinConstraintNode, JoinNode, LimitNode, OffsetLimitNode, OffsetNode, OrderByNode,
        ProjectNode, SelectNode,
    },
    crate::{
        ast::{Expr, Query, SetExpr, Values},
        parse_sql::parse_query,
        result::{Error, Result},
        translate::translate_query,
    },
};

#[derive(Clone, Debug)]
pub enum QueryNode<'a> {
    Text(String),
    Values(Vec<ExprList<'a>>),
    SelectNode(SelectNode<'a>),
    ValuesNode(ValuesNode<'a>),
    JoinNode(JoinNode<'a>),
    JoinConstraintNode(JoinConstraintNode<'a>),
    HashJoinNode(HashJoinNode<'a>),
    GroupByNode(GroupByNode<'a>),
    HavingNode(HavingNode<'a>),
    LimitNode(LimitNode<'a>),
    OffsetNode(OffsetNode<'a>),
    OffsetLimitNode(OffsetLimitNode<'a>),
    FilterNode(FilterNode<'a>),
    ProjectNode(ProjectNode<'a>),
    OrderByNode(OrderByNode<'a>),
}

impl<'a> QueryNode<'a> {
    pub fn alias_as(self, chain_alias: &'a str) -> ChainFactorNode<'a> {
        ChainFactorNode {
            chain_name: chain_alias.to_owned(),
            chain_query_type: ChainQueryType::Derived {
                subquery: Box::new(self),
                alias: chain_alias.to_owned(),
            },
            entity_name: None,
            table_alias: None,
            index: None,
        }
    }
}

impl<'a> From<&str> for QueryNode<'a> {
    fn from(query: &str) -> Self {
        Self::Text(query.to_owned())
    }
}

impl<'a> From<SelectNode<'a>> for QueryNode<'a> {
    fn from(node: SelectNode<'a>) -> Self {
        QueryNode::SelectNode(node)
    }
}

macro_rules! impl_from_select_nodes {
    ($type: ident) => {
        impl<'a> From<$type<'a>> for QueryNode<'a> {
            fn from(node: $type<'a>) -> Self {
                QueryNode::$type(node)
            }
        }
    };
}

impl_from_select_nodes!(JoinNode);
impl_from_select_nodes!(JoinConstraintNode);
impl_from_select_nodes!(HashJoinNode);
impl_from_select_nodes!(GroupByNode);
impl_from_select_nodes!(HavingNode);
impl_from_select_nodes!(FilterNode);
impl_from_select_nodes!(LimitNode);
impl_from_select_nodes!(OffsetNode);
impl_from_select_nodes!(OffsetLimitNode);
impl_from_select_nodes!(ProjectNode);
impl_from_select_nodes!(OrderByNode);

impl<'a> TryFrom<QueryNode<'a>> for Query {
    type Error = Error;

    fn try_from(query_node: QueryNode<'a>) -> Result<Self> {
        match query_node {
            QueryNode::Text(query_node) => {
                parse_query(query_node).and_then(|item| translate_query(&item))
            }
            QueryNode::Values(values) => {
                let values: Vec<Vec<Expr>> = values
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?;

                Ok(Query {
                    body: SetExpr::Values(Values(values)),
                    order_by: Vec::new(),
                    limit: None,
                    offset: None,
                })
            }
            QueryNode::SelectNode(node) => node.prebuild(),
            QueryNode::ValuesNode(node) => node.prebuild(),
            QueryNode::JoinNode(node) => node.prebuild(),
            QueryNode::JoinConstraintNode(node) => node.prebuild(),
            QueryNode::HashJoinNode(node) => node.prebuild(),
            QueryNode::GroupByNode(node) => node.prebuild(),
            QueryNode::HavingNode(node) => node.prebuild(),
            QueryNode::FilterNode(node) => node.prebuild(),
            QueryNode::LimitNode(node) => node.prebuild(),
            QueryNode::OffsetNode(node) => node.prebuild(),
            QueryNode::OffsetLimitNode(node) => node.prebuild(),
            QueryNode::ProjectNode(node) => node.prebuild(),
            QueryNode::OrderByNode(node) => node.prebuild(),
        }
    }
}

#[cfg(test)]
mod test {
    use {
        super::QueryNode,
        crate::{
            ast::{
                Join, JoinConstraint, JoinExecutor, JoinOperator, Query, Select, SetExpr,
                TableFactor, TableWithJoins,
            },
            ast_builder::{
                chain, chain_query_objects, chain_table_columns, chain_tables, col, series,
                test_query, SelectItemList,
            },
        },
        pretty_assertions::assert_eq,
    };

    #[test]
    fn query() {
        let actual = chain("sui").select("checkpoints").into();
        let expected = "SELECT * FROM sui.checkpoints";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("blocks")
            .join(None, "chackpoints")
            .into();
        let expected = "SELECT * FROM sui.blocks JOIN chackpoints";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transations")
            .join(None, "checkpoints")
            .on("transations.digest = checkpoints.digest")
            .into();
        let expected = "SELECT * FROM sui.transations JOIN checkpoints ON transations.digest = checkpoints.digest";
        test_query(actual, expected);

        let actual: QueryNode = chain("sui")
            .select("Player")
            .join(None, "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .into();
        let expected = {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name: None,
                    name: "PlayerItem".to_owned(),
                    alias: None,
                    index: None,
                    existing_table: true,
                },
                join_operator: JoinOperator::Inner(JoinConstraint::None),
                join_executor: JoinExecutor::Hash {
                    key_expr: col("PlayerItem.user_id").try_into().unwrap(),
                    value_expr: col("Player.id").try_into().unwrap(),
                    where_clause: None,
                },
            };
            let select = Select {
                projection: SelectItemList::from("*").try_into().unwrap(),
                from: TableWithJoins {
                    relation: TableFactor::Table {
                        chain_name: None,
                        name: "Player".to_owned(),
                        alias: None,
                        index: None,
                        existing_table: true,
                    },
                    joins: vec![join],
                },
                selection: None,
                group_by: Vec::new(),
                having: None,
            };

            Query {
                body: SetExpr::Select(Box::new(select)),
                order_by: Vec::new(),
                limit: None,
                offset: None,
            }
        };
        assert_eq!(Query::try_from(actual).unwrap(), expected);

        let actual = chain("sui")
            .select("transations")
            .group_by("transation_kind")
            .into();
        let expected = "SELECT * FROM sui.transations GROUP BY  transation_kind";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transactions")
            .group_by("tnx")
            .having("COUNT(tnx) > 10")
            .into();
        let expected = "SELECT * FROM sui.transactions GROUP BY tnx HAVING COUNT(tnx) > 10";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transactions")
            .group_by("transactions_kind")
            .having("COUNT() < 100")
            .limit(3)
            .into();
        let expected = "SELECT * FROM FOO GROUP BY city HAVING COUNT(name) < 100 LIMIT 3";
        test_query(actual, expected);

        let actual = chain("sui").select("transations").offset(10).into();
        let expected = "SELECT * FROM sui.transations OFFSET 10";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transactions")
            .group_by("transaction_kind")
            .having("COUNT(tnx) < 100")
            .offset(1)
            .limit(3)
            .into();
        let expected = "SELECT * FROM sui.transactions GROUP BY transaction_kind HAVING COUNT(tnx) < 100 OFFSET 1 LIMIT 3";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transactions")
            .project("digest, tnx")
            .limit(10)
            .into();
        let expected = r#"SELECT digest, tnx FROM sui.transactions LIMIT 10"#;
        test_query(actual, expected);

        let actual = chain("sui")
            .select("tranastions")
            .order_by("tnx DESC")
            .into();
        let expected = "SELECT * FROM sui.tranastions ORDER BY tnx DESC";
        test_query(actual, expected);

        let actual = chain_query_objects().select().into();
        let expected = "SELECT * FROM chain.query_objects";
        test_query(actual, expected);

        let actual = chain_tables().select().into();
        let expected = "SELECT * FROM chain.entity_tables";
        test_query(actual, expected);

        let actual = chain_table_columns().select().into();
        let expected = "SELECT * FROM chain.entity_columns";
        test_query(actual, expected);

        let actual = series("1 + 2").select().into();
        let expected = "SELECT * FROM SERIES(1 + 2)";
        test_query(actual, expected);

        let actual = chain("sui")
            .select("transations")
            .alias_as("transations")
            .select()
            .into();
        let expected = "SELECT * FROM (SELECT * FROM sui.transations) AS transations";
        test_query(actual, expected);
    }
}
