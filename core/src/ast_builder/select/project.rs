use {
    super::Prebuild,
    crate::{
        ast::Select,
        ast_builder::{
            ChainFactorNode, ExprNode, FilterNode, GroupByNode, HashJoinNode, HavingNode,
            JoinConstraintNode, JoinNode, LimitNode, OffsetNode, OrderByExprList, OrderByNode,
            QueryNode, SelectItemList, SelectNode,
        },
        result::Result,
    },
};

#[derive(Clone, Debug)]
pub enum PrevNode<'a> {
    Select(SelectNode<'a>),
    GroupBy(GroupByNode<'a>),
    Having(HavingNode<'a>),
    Join(Box<JoinNode<'a>>),
    JoinConstraint(Box<JoinConstraintNode<'a>>),
    HashJoin(HashJoinNode<'a>),
    Filter(FilterNode<'a>),
}

impl<'a> Prebuild<Select> for PrevNode<'a> {
    fn prebuild(self) -> Result<Select> {
        match self {
            Self::Select(node) => node.prebuild(),
            Self::GroupBy(node) => node.prebuild(),
            Self::Having(node) => node.prebuild(),
            Self::Join(node) => node.prebuild(),
            Self::JoinConstraint(node) => node.prebuild(),
            Self::HashJoin(node) => node.prebuild(),
            Self::Filter(node) => node.prebuild(),
        }
    }
}

impl<'a> From<SelectNode<'a>> for PrevNode<'a> {
    fn from(node: SelectNode<'a>) -> Self {
        PrevNode::Select(node)
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

impl<'a> From<JoinNode<'a>> for PrevNode<'a> {
    fn from(node: JoinNode<'a>) -> Self {
        PrevNode::Join(Box::new(node))
    }
}

impl<'a> From<JoinConstraintNode<'a>> for PrevNode<'a> {
    fn from(node: JoinConstraintNode<'a>) -> Self {
        PrevNode::JoinConstraint(Box::new(node))
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

#[derive(Clone, Debug)]
pub struct ProjectNode<'a> {
    prev_node: PrevNode<'a>,
    select_items_list: Vec<SelectItemList<'a>>,
}

impl<'a> ProjectNode<'a> {
    pub fn new<N: Into<PrevNode<'a>>, T: Into<SelectItemList<'a>>>(
        prev_node: N,
        select_items: T,
    ) -> Self {
        Self {
            prev_node: prev_node.into(),
            select_items_list: vec![select_items.into()],
        }
    }

    pub fn project<T: Into<SelectItemList<'a>>>(mut self, select_items: T) -> Self {
        self.select_items_list.push(select_items.into());

        self
    }

    pub fn alias_as(self, table_alias: &'a str) -> ChainFactorNode<'a> {
        QueryNode::ProjectNode(self).alias_as(table_alias)
    }

    pub fn order_by<T: Into<OrderByExprList<'a>>>(self, order_by_exprs: T) -> OrderByNode<'a> {
        OrderByNode::new(self, order_by_exprs)
    }

    pub fn offset<T: Into<ExprNode<'a>>>(self, expr: T) -> OffsetNode<'a> {
        OffsetNode::new(self, expr)
    }

    pub fn limit<T: Into<ExprNode<'a>>>(self, expr: T) -> LimitNode<'a> {
        LimitNode::new(self, expr)
    }
}

impl<'a> Prebuild<Select> for ProjectNode<'a> {
    fn prebuild(self) -> Result<Select> {
        let mut query: Select = self.prev_node.prebuild()?;
        query.projection = self
            .select_items_list
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Vec<_>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(query)
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
    fn project() {
        // select node -> project node -> build
        let actual = chain("sui").select("transactions").project("id").build();
        let expected = "SELECT  id FROM sui.transactions";
        test(actual, expected);

        // select node -> project node -> build
        let actual = chain("base")
            .select("accounts")
            .project("*, accounts.*, address")
            .build();
        let expected = "SELECT *, accounts.*, address FROM base.accounts";
        test(actual, expected);

        // project node -> project node -> build
        let actual = chain("sui")
            .select("transactions")
            .project(vec!["col1", "col2"])
            .project("col3")
            .project(vec!["col4".into(), col("col5")])
            .project(col("col6"))
            .project("col7 as hello")
            .build();
        let expected = "
            SELECT
                col1, col2, col3,
                col4, col5, col6,
                col7 as hello
            FROM
                sui.transactions
        ";
        test(actual, expected);

        // select node -> project node -> build
        let actual = chain("sui")
            .select("transactions")
            .project("1 + 1 as col1, col2")
            .build();
        let expected = "SELECT 1 + 1 as col1, col2 FROM  sui.transactions";
        test(actual, expected);
    }

    #[test]
    fn prev_nodes() {
        // select node -> project node -> build
        let actual = chain("base").select("transactions").project("*").build();
        let expected = "SELECT * FROM base.transactions";
        test(actual, expected);

        // group by node -> project node -> build
        let actual = chain("base")
            .select("transactions")
            .group_by("address_from")
            .project("address_from, COUNT(hash) as unique_id")
            .build();
        let expected = "
            SELECT address_from, COUNT(hash) as unique_id
            FROM base.transactions
            GROUP BY address_from; 
        ";
        test(actual, expected);

        // // having node -> project node -> build
        let actual = chain("base")
            .select("accounts")
            .filter(r#"address = "0x00000000219ab540356cbb839cbe05303d7705fa""#)
            .group_by("balance")
            .having("SUM(length) < 1000")
            .project(col("age"))
            .project("SUM(length)")
            .build();
        let expected = r#"
            SELECT age, SUM(length)
            FROM base.accounts
            WHERE address = "0x00000000219ab540356cbb839cbe05303d7705fa"
            GROUP BY balance
            HAVING SUM(length) < 1000;
        "#;
        test(actual, expected);

        // // hash join node -> project node -> build
        let actual = chain("sui")
            .select("Player")
            .join(None, "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .project("Player.name, PlayerItem.name")
            .build();
        let expected = {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name:None,
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
                projection: SelectItemList::from("Player.name, PlayerItem.name")
                    .try_into()
                    .unwrap(),
                from: TableWithJoins {
                    relation: TableFactor::Table {
                        chain_name:None,
                        name: "Player".to_owned(),
                        alias: None,
                        index: None,
                        existing_table: true
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
        assert_eq!(actual, expected);

        // select -> project -> derived subquery
        let actual = chain("base")
            .select("transactions")
            .project("hash")
            .alias_as("transations_table")
            .select()
            .build();
        let expected = "SELECT * FROM (SELECT hash FROM base.transactions) transations_table";
        test(actual, expected);
    }
}
