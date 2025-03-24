use {
    super::{values::ValuesNode, Prebuild},
    crate::{
        ast::Query,
        ast_builder::{
            ChainFactorNode, ExprNode, FilterNode, GroupByNode, HashJoinNode, HavingNode,
            JoinConstraintNode, JoinNode, OrderByNode, ProjectNode, QueryNode, SelectNode,
        },
        result::Result,
    },
};

#[derive(Clone, Debug)]
pub enum PrevNode<'a> {
    Select(SelectNode<'a>),
    Values(ValuesNode<'a>),
    GroupBy(GroupByNode<'a>),
    Having(HavingNode<'a>),
    Join(Box<JoinNode<'a>>),
    JoinConstraint(Box<JoinConstraintNode<'a>>),
    HashJoin(HashJoinNode<'a>),
    Filter(FilterNode<'a>),
    OrderBy(OrderByNode<'a>),
    ProjectNode(Box<ProjectNode<'a>>),
}

impl<'a> Prebuild<Query> for PrevNode<'a> {
    fn prebuild(self) -> Result<Query> {
        match self {
            Self::Select(node) => node.prebuild(),
            Self::Values(node) => node.prebuild(),
            Self::GroupBy(node) => node.prebuild(),
            Self::Having(node) => node.prebuild(),
            Self::Join(node) => node.prebuild(),
            Self::JoinConstraint(node) => node.prebuild(),
            Self::HashJoin(node) => node.prebuild(),
            Self::Filter(node) => node.prebuild(),
            Self::OrderBy(node) => node.prebuild(),
            Self::ProjectNode(node) => node.prebuild(),
        }
    }
}

impl<'a> From<SelectNode<'a>> for PrevNode<'a> {
    fn from(node: SelectNode<'a>) -> Self {
        PrevNode::Select(node)
    }
}

impl<'a> From<ValuesNode<'a>> for PrevNode<'a> {
    fn from(node: ValuesNode<'a>) -> Self {
        PrevNode::Values(node)
    }
}

impl<'a> From<GroupByNode<'a>> for PrevNode<'a> {
    fn from(node: GroupByNode<'a>) -> Self {
        PrevNode::GroupBy(node)
    }
}

impl<'a> From<HavingNode<'a>> for PrevNode<'a> {
    fn from(node: HavingNode<'a>) -> Self {
        PrevNode::Having(node)
    }
}

impl<'a> From<JoinConstraintNode<'a>> for PrevNode<'a> {
    fn from(node: JoinConstraintNode<'a>) -> Self {
        PrevNode::JoinConstraint(Box::new(node))
    }
}

impl<'a> From<JoinNode<'a>> for PrevNode<'a> {
    fn from(node: JoinNode<'a>) -> Self {
        PrevNode::Join(Box::new(node))
    }
}

impl<'a> From<HashJoinNode<'a>> for PrevNode<'a> {
    fn from(node: HashJoinNode<'a>) -> Self {
        PrevNode::HashJoin(node)
    }
}

impl<'a> From<FilterNode<'a>> for PrevNode<'a> {
    fn from(node: FilterNode<'a>) -> Self {
        PrevNode::Filter(node)
    }
}

impl<'a> From<OrderByNode<'a>> for PrevNode<'a> {
    fn from(node: OrderByNode<'a>) -> Self {
        PrevNode::OrderBy(node)
    }
}

impl<'a> From<ProjectNode<'a>> for PrevNode<'a> {
    fn from(node: ProjectNode<'a>) -> Self {
        PrevNode::ProjectNode(Box::new(node))
    }
}

#[derive(Clone, Debug)]
pub struct LimitNode<'a> {
    prev_node: PrevNode<'a>,
    expr: ExprNode<'a>,
}

impl<'a> LimitNode<'a> {
    pub fn new<N: Into<PrevNode<'a>>, T: Into<ExprNode<'a>>>(prev_node: N, expr: T) -> Self {
        Self {
            prev_node: prev_node.into(),
            expr: expr.into(),
        }
    }

    pub fn alias_as(self, table_alias: &'a str) -> ChainFactorNode<'a> {
        QueryNode::LimitNode(self).alias_as(table_alias)
    }
}

impl<'a> Prebuild<Query> for LimitNode<'a> {
    fn prebuild(self) -> Result<Query> {
        let mut node_data = self.prev_node.prebuild()?;
        node_data.limit = Some(self.expr.try_into()?);

        Ok(node_data)
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
            ast_builder::{chain, col, num, test, Build, SelectItemList},
        },
        pretty_assertions::assert_eq,
    };

    #[test]
    fn limit() {
        // select node -> limit node -> build
        let actual = chain("base").select("transations").limit(10).build();
        let expected = "SELECT * FROM base.transations LIMIT 10";
        test(actual, expected);

        // group by node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .group_by("to")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations GROUP BY to LIMIT 10";
        test(actual, expected);

        // having node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .group_by("type")
            .having("type = 2")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations GROUP BY type HAVING type = 2 LIMIT 10";
        test(actual, expected);

        // join node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .join(Some("base"), "blocks")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations JOIN base.blocks LIMIT 10";
        test(actual, expected);

        // join node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .join_as(Some("base"), "blocks", "B")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations JOIN base.blocks AS B LIMIT 10";
        test(actual, expected);

        // join node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .left_join(Some("base"), "blocks")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations LEFT JOIN base.blocks LIMIT 10";
        test(actual, expected);

        // join node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .left_join_as(Some("base"), "blocks", "B")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations LEFT JOIN blocks AS B LIMIT 10";
        test(actual, expected);

        // group by node -> limit node -> build
        let actual = chain("base")
            .select("transations")
            .group_by("to")
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.transations GROUP BY to LIMIT 10";
        test(actual, expected);

        // having node -> limit node -> build
        let actual = chain("base")
            .select("foo")
            .group_by("id")
            .having(col("id").gt(10))
            .limit(10)
            .build();
        let expected = "SELECT * FROM base.foo GROUP BY id HAVING id > 10 LIMIT 10";
        test(actual, expected);

        // join constraint node -> limit node -> build
        let actual = chain("mina")
            .select("Foo")
            .join(Some("mina"), "Bar")
            .on("Foo.id = Bar.id")
            .limit(10)
            .build();
        let expected = "SELECT * FROM mina.Foo JOIN mina.Bar ON Foo.id = Bar.id LIMIT 10";
        test(actual, expected);

        // filter node -> limit node -> build
        let actual = chain("mina")
            .select("World")
            .filter(col("id").gt(2))
            .limit(100)
            .build();
        let expected = "SELECT * FROM mina.World WHERE id > 2 LIMIT 100";
        test(actual, expected);

        // order by node -> limit node -> build
        let actual = chain("mina")
            .select("Hello")
            .order_by("score")
            .limit(3)
            .build();
        let expected = "SELECT * FROM mina.Hello ORDER BY score LIMIT 3";
        test(actual, expected);

        // project node -> limit node -> build
        let actual = chain("base").select("Item").project("*").limit(10).build();
        let expected = "SELECT * FROM base.Item LIMIT 10";
        test(actual, expected);

        // hash join node -> limit node -> build
        let actual = chain("base")
            .select("Player")
            .join(Some("base"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .limit(100)
            .build();
        let expected = {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name: Some("base".to_owned()),
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
                        chain_name: Some("base".to_owned()),
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
                limit: Some(num(100).try_into().unwrap()),
                offset: None,
            }))
        };
        assert_eq!(actual, expected);

        // select node -> limit node -> derived subquery
        let actual = chain("base")
            .select("transations")
            .limit(10)
            .alias_as("Tnx")
            .select()
            .build();
        let expected = "SELECT * FROM (SELECT * FROM base.transations LIMIT 10) Tnx";
        test(actual, expected);
    }
}
