use {
    super::{JoinConstraintData, JoinOperatorType},
    crate::{
        ast::{Join, JoinConstraint, JoinOperator, Select},
        ast_builder::{
            select::Prebuild, ChainFactorNode, ExprList, ExprNode, FilterNode, GroupByNode,
            HashJoinNode, JoinNode, LimitNode, OffsetNode, OrderByExprList, OrderByNode,
            ProjectNode, QueryNode, SelectItemList,
        },
        result::Result,
    },
};

#[derive(Clone, Debug)]
pub enum PrevNode<'a> {
    Join(Box<JoinNode<'a>>),
    HashJoin(Box<HashJoinNode<'a>>),
}

impl<'a> PrevNode<'a> {
    fn prebuild_for_constraint(self) -> Result<JoinConstraintData> {
        match self {
            PrevNode::Join(node) => node.prebuild_for_constraint(),
            PrevNode::HashJoin(node) => node.prebuild_for_constraint(),
        }
    }
}

impl<'a> From<JoinNode<'a>> for PrevNode<'a> {
    fn from(node: JoinNode<'a>) -> Self {
        PrevNode::Join(Box::new(node))
    }
}

impl<'a> From<HashJoinNode<'a>> for PrevNode<'a> {
    fn from(node: HashJoinNode<'a>) -> Self {
        PrevNode::HashJoin(Box::new(node))
    }
}

#[derive(Clone, Debug)]
pub struct JoinConstraintNode<'a> {
    prev_node: PrevNode<'a>,
    expr: ExprNode<'a>,
}

impl<'a> JoinConstraintNode<'a> {
    pub fn new<N: Into<PrevNode<'a>>, T: Into<ExprNode<'a>>>(prev_node: N, expr: T) -> Self {
        Self {
            prev_node: prev_node.into(),
            expr: expr.into(),
        }
    }

    pub fn join(self, chain_name: Option<&str>, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.map(String::from),
            table_name.to_owned(),
            None,
            JoinOperatorType::Inner,
        )
    }

    pub fn join_as(self, chain_name: Option<&str>, table_name: &str, alias: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.map(String::from),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Inner,
        )
    }

    pub fn left_join(self, chain_name: Option<&str>, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.map(String::from),
            table_name.to_owned(),
            None,
            JoinOperatorType::Left,
        )
    }

    pub fn left_join_as(
        self,
        chain_name: Option<&str>,
        table_name: &str,
        alias: &str,
    ) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.map(String::from),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Left,
        )
    }

    pub fn project<T: Into<SelectItemList<'a>>>(self, select_items: T) -> ProjectNode<'a> {
        ProjectNode::new(self, select_items)
    }

    pub fn group_by<T: Into<ExprList<'a>>>(self, expr_list: T) -> GroupByNode<'a> {
        GroupByNode::new(self, expr_list)
    }

    pub fn offset<T: Into<ExprNode<'a>>>(self, expr: T) -> OffsetNode<'a> {
        OffsetNode::new(self, expr)
    }

    pub fn limit<T: Into<ExprNode<'a>>>(self, expr: T) -> LimitNode<'a> {
        LimitNode::new(self, expr)
    }

    pub fn filter<T: Into<ExprNode<'a>>>(self, expr: T) -> FilterNode<'a> {
        FilterNode::new(self, expr)
    }

    pub fn order_by<T: Into<OrderByExprList<'a>>>(self, order_by_exprs: T) -> OrderByNode<'a> {
        OrderByNode::new(self, order_by_exprs)
    }

    pub fn alias_as(self, chain_alias: &'a str) -> ChainFactorNode<'a> {
        QueryNode::JoinConstraintNode(self).alias_as(chain_alias)
    }
}

impl<'a> Prebuild<Select> for JoinConstraintNode<'a> {
    fn prebuild(self) -> Result<Select> {
        let JoinConstraintData {
            mut select,
            relation,
            operator_type,
            executor: join_executor,
        } = self.prev_node.prebuild_for_constraint()?;
        select.from.joins.push(Join {
            relation,
            join_operator: match operator_type {
                JoinOperatorType::Inner => {
                    JoinOperator::Inner(JoinConstraint::On(self.expr.try_into()?))
                }
                JoinOperatorType::Left => {
                    JoinOperator::LeftOuter(JoinConstraint::On(self.expr.try_into()?))
                }
            },
            join_executor,
        });

        Ok(select)
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::{
            ast::{
                Join, JoinConstraint, JoinExecutor, JoinOperator, Query, Select, SetExpr,
                Statement, TableFactor, TableWithJoins,
            },
            ast_builder::{chain, col, test, Build, SelectItemList},
        },
        pretty_assertions::assert_eq,
    };

    #[test]
    fn join_constraint() {
        // join node ->  join constarint node -> build
        let actual = chain("sui")
            .select("checkpoints")
            .join(None, "transations")
            .on("checkpoints.digest = transations.digest")
            .build();
        let expected = "SELECT * FROM sui.checkpoints INNER JOIN  transations ON checkpoints.digest = transations.digest";
        test(actual, expected);

        // join node ->  join constraint node -> build
        let actual = chain("sui")
            .select("checkpoints")
            .join_as(None, "transations", "tnx")
            .on("checkpoints.digest = tnx.digest")
            .build();
        let expected = "SELECT * FROM sui.checkpoints INNER JOIN transations AS tnx ON checkpoints.digest = tnx.digest";
        test(actual, expected);

        // join node -> join constraint node -> build
        let actual = chain("sui")
            .select("checkpoints")
            .left_join(None, "transations")
            .on("checkpoints.id = transations.id")
            .build();
        let expected = "SELECT * FROM  sui.checkpoints LEFT OUTER JOIN transations ON checkpoints.id = transations.id";
        test(actual, expected);

        // join node -> join constraint node -> build
        let actual = chain("sui")
            .select("checkpoints")
            .left_join_as(None, "transations", "t")
            .on("checkpoints.digest = t.digest")
            .build();
        let expected = "SELECT * FROM sui.checkpoints LEFT OUTER JOIN transations AS t ON checkpoints.digest = t.digest";
        test(actual, expected);

        // hash join node -> join constraint node -> build
        let actual = chain("sui")
            .select("Player")
            .join(None, "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .on("PlayerItem.flag IS NOT NULL")
            .build();
        let expected = {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name: None,
                    name: "PlayerItem".to_owned(),
                    alias: None,
                    index: None,
                    existing_table: true,
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(
                    col("PlayerItem.flag").is_not_null().try_into().unwrap(),
                )),
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
                        chain_name: Some("sui".to_owned()),
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

            Ok(Statement::Query(Query {
                body: SetExpr::Select(Box::new(select)),
                order_by: Vec::new(),
                limit: None,
                offset: None,
            }))
        };
        assert_eq!(actual, expected, "hash join -> join constraint");

        // join -> on -> derived subquery
        let actual = chain("sui")
            .select("checkpoints")
            .join(None, "transations")
            .on("checkpoints.digest = transations.digest")
            .alias_as("Transationx")
            .select()
            .build();
        let expected = "
            SELECT * FROM (
                SELECT * FROM sui.checkpoints
                INNER JOIN transations ON checkpoints.digest = transations.digest
            ) Transationx
            ";
        test(actual, expected);
    }
}
