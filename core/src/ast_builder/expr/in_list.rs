use {
    super::ExprNode,
    crate::ast_builder::{
        FilterNode, GroupByNode, HashJoinNode, HavingNode, JoinConstraintNode, JoinNode, LimitNode,
        OffsetLimitNode, OffsetNode, OrderByNode, ProjectNode, QueryNode, SelectNode,
    },
};

#[derive(Clone, Debug)]
pub enum InListNode<'a> {
    InList(Vec<ExprNode<'a>>),
    Query(Box<QueryNode<'a>>),
    Text(String),
}

impl<'a> From<Vec<ExprNode<'a>>> for InListNode<'a> {
    fn from(list: Vec<ExprNode<'a>>) -> Self {
        InListNode::InList(list)
    }
}

impl<'a> From<&str> for InListNode<'a> {
    fn from(query: &str) -> Self {
        InListNode::Text(query.to_owned())
    }
}

impl<'a> From<QueryNode<'a>> for InListNode<'a> {
    fn from(node: QueryNode<'a>) -> Self {
        InListNode::Query(Box::new(node))
    }
}

macro_rules! impl_from_select_nodes {
    ($type: path) => {
        impl<'a> From<$type> for InListNode<'a> {
            fn from(list: $type) -> Self {
                InListNode::Query(Box::new(list.into()))
            }
        }
    };
}

impl_from_select_nodes!(SelectNode<'a>);
impl_from_select_nodes!(JoinNode<'a>);
impl_from_select_nodes!(JoinConstraintNode<'a>);
impl_from_select_nodes!(HashJoinNode<'a>);
impl_from_select_nodes!(GroupByNode<'a>);
impl_from_select_nodes!(HavingNode<'a>);
impl_from_select_nodes!(FilterNode<'a>);
impl_from_select_nodes!(LimitNode<'a>);
impl_from_select_nodes!(OffsetNode<'a>);
impl_from_select_nodes!(OffsetLimitNode<'a>);
impl_from_select_nodes!(ProjectNode<'a>);
impl_from_select_nodes!(OrderByNode<'a>);

impl<'a> ExprNode<'a> {
    pub fn in_list<T: Into<InListNode<'a>>>(self, value: T) -> Self {
        Self::InList {
            expr: Box::new(self),
            list: Box::new(value.into()),
            negated: false,
        }
    }

    pub fn not_in_list<T: Into<InListNode<'a>>>(self, value: T) -> Self {
        Self::InList {
            expr: Box::new(self),
            list: Box::new(value.into()),
            negated: true,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{
            Expr, Join, JoinConstraint, JoinExecutor, JoinOperator, Query, Select, SetExpr,
            TableFactor, TableWithJoins,
        },
        ast_builder::{chain, col, test_expr, text, QueryNode, SelectItemList},
    };

    #[test]
    fn in_list() {
        let actual = col("id").in_list(vec![text("a"), text("b"), text("c")]);
        let expected = "id IN ('a', 'b', 'c')";
        test_expr(actual, expected);

        let actual = col("id").not_in_list("opt1, opt2, opt3");
        let expected = "id NOT IN (opt1, opt2, opt3)";
        test_expr(actual, expected);
    }

    #[test]
    fn from_nodes() {
        // from Vec<ExprNode>
        let actual = col("id").not_in_list(vec![text("a"), text("b"), text("c")]);
        let expected = "id NOT IN ('a', 'b', 'c')";
        test_expr(actual, expected);

        // from &str
        let actual = col("id").in_list("1, 2, 3, 4, 5");
        let expected = "id IN (1, 2, 3, 4, 5)";
        test_expr(actual, expected);

        let actual = col("id").in_list("SELECT id FROM FOO");
        let expected = "id IN (SELECT id FROM FOO)";
        test_expr(actual, expected);

        // from QueryNode
        let query_node = QueryNode::from("SELECT name FROM ItemList");
        let actual = col("id").in_list(query_node);
        let expected = "id IN (SELECT name FROM ItemList)";
        test_expr(actual, expected);

        // from SelectNode
        let actual = col("sequence_number").in_list(chain("sui").select("checkpoints"));
        let expected = "sequence_number IN (SELECT * FROM sui.checkpoints)";
        test_expr(actual, expected);

        // from JoinNode
        let actual = col("timestamp_ms")
            .in_list(chain("sui").select("checkpoints").join(None, "transations"));
        let expected = "timestamp_ms IN (SELECT * FROM sui.checkpoints JOIN transations)";
        test_expr(actual, expected);

        // from JoinConstraintNode
        let actual = col("timestamp_ms").in_list(
            chain("sui")
                .select("checkpoints")
                .join(None, "transations")
                .on("checkpoints.sender = transation.sender"),
        );
        let expected = " timestamp_ms IN (SELECT * FROM sui.checkpoints JOIN transations ON checkpoints.sender = transation.sender)";
        test_expr(actual, expected);

        // from HashJoinNode
        let actual = col("id").in_list(
            chain("sui")
                .select("checkpoints")
                .join(None, "transations")
                .hash_executor("checkpoints.sender", "transation.sender"),
        );
        let expected = {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name:None,
                    name: "PlayerItem".to_owned(),
                    alias: None,
                    index: None,
                    existing_table: false,
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
                        chain_name:None,
                        name: "Player".to_owned(),
                        alias: None,
                        index: None,
                        existing_table: false,
                    },
                    joins: vec![join],
                },
                selection: None,
                group_by: Vec::new(),
                having: None,
            };

            let query = Query {
                body: SetExpr::Select(Box::new(select)),
                order_by: Vec::new(),
                limit: None,
                offset: None,
            };

            Expr::InSubquery {
                expr: Box::new(Expr::Identifier("id".to_owned())),
                subquery: Box::new(query),
                negated: false,
            }
        };
        assert_eq!(Expr::try_from(actual).unwrap(), expected);

        // from GroupByNode
        let actual = col("id").not_in_list(
            chain("base")
                .select("erc20")
                .filter(col("volume_24h").is_null())
                .group_by("id, (a + name)"),
        );
        let expected =
            "id NOT IN (SELECT * FROM baese.erc20 WHERE id IS NULL GROUP BY id, (a + name))";
        test_expr(actual, expected);

        // from HavingNode
        let actual = col("id").in_list(
            chain("sui")
                .select("transations")
                .filter("id IS NULL")
                .group_by("id, (a + name)")
                .having("COUNT(id) > 10"),
        );
        let expected = "
            id IN (
                SELECT * FROM Bar
                WHERE id IS NULL
                GROUP BY id, (a + name)
                HAVING COUNT(id) > 10
            )
        ";
        test_expr(actual, expected);

        // from FilterNode
        let actual = col("digest").in_list(chain("sui").select("transations").filter("tnx > 10"));
        let expected = "digest IN (SELECT * FROM sui.transations WHERE tnx > 10)";
        test_expr(actual, expected);

        // from LimitNode
        let actual = col("min_tx_sequence_number").in_list(
            chain("sui")
                .select("transations")
                .filter("tnx IS NULL")
                .limit(10),
        );
        let expected =
            "min_tx_sequence_number IN (SELECT * FROM sui.transations WHERE tnx IS NULL LIMIT 10)";
        test_expr(actual, expected);

        // from OffsetNode
        let actual = col("id").not_in_list(chain("sui").select("transations").offset(10));
        let expected = "tnx NOT IN (SELECT * FROM sui.transations OFFSET 10)";
        test_expr(actual, expected);

        // from OffsetLimitNode
        let actual = col("id").in_list(chain("sui").select("transations").offset(1).limit(3));
        let expected = "tnx IN (SELECT * FROM sui.transations OFFSET 1 LIMIT 3)";
        test_expr(actual, expected);

        // from ProjectNode
        let actual = col("tnx").in_list(chain("sui").select("transations").project("tnx"));
        let expected = "tnx IN (SELECT name FROM sui.transations)";
        test_expr(actual, expected);

        // from OrderByNode
        let actual = col("digest").in_list(chain("sui").select("transations").order_by("tnx ASC"));
        let expected = "digest IN (SELECT * FROM sui.transations ORDER BY tnx ASC)";
        test_expr(actual, expected);
    }
}
