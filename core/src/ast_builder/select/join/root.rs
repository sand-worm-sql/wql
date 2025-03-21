use {
    super::{JoinConstraintData, JoinOperatorType},
    crate::{
        ast::{Join, JoinExecutor, JoinOperator, Select, TableAlias, TableFactor},
        ast_builder::{
            chain_name, select::Prebuild, ChainFactorNode, ExprList, ExprNode, FilterNode,
            GroupByNode, HashJoinNode, JoinConstraintNode, LimitNode, OffsetNode, OrderByExprList,
            OrderByNode, ProjectNode, QueryNode, SelectItemList, SelectNode,
        },
        result::Result,
    },
};

#[derive(Clone, Debug)]
pub enum PrevNode<'a> {
    Select(SelectNode<'a>),
    Join(Box<JoinNode<'a>>),
    JoinConstraint(Box<JoinConstraintNode<'a>>),
    HashJoin(Box<HashJoinNode<'a>>),
}

impl<'a> Prebuild<Select> for PrevNode<'a> {
    fn prebuild(self) -> Result<Select> {
        match self {
            Self::Select(node) => node.prebuild(),
            Self::Join(node) => node.prebuild(),
            Self::JoinConstraint(node) => node.prebuild(),
            Self::HashJoin(node) => node.prebuild(),
        }
    }
}

impl<'a> From<SelectNode<'a>> for PrevNode<'a> {
    fn from(node: SelectNode<'a>) -> Self {
        PrevNode::Select(node)
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
        PrevNode::HashJoin(Box::new(node))
    }
}

#[derive(Clone, Debug)]
pub struct JoinNode<'a> {
    prev_node: PrevNode<'a>,
    relation: TableFactor,
    join_operator_type: JoinOperatorType,
    chain_name: String,
}

impl<'a> JoinNode<'a> {
    pub fn new<N: Into<PrevNode<'a>>>(
        prev_node: N,
        chain_name: String,
        name: String,
        alias: Option<String>,
        join_operator_type: JoinOperatorType,
    ) -> Self {
        Self {
            prev_node: prev_node.into(),
            join_operator_type,
            chain_name: chain_name.clone(),
            relation: match alias {
                Some(alias) => TableFactor::Table {
                    chain_name: chain_name.clone(),
                    name,
                    alias: Some(TableAlias {
                        name: alias,
                        columns: vec![],
                    }),
                    index: None,
                },
                None => TableFactor::Table {
                    chain_name: chain_name.clone(),
                    name,
                    alias: None,
                    index: None,
                },
            },
        }
    }

    pub fn on<T: Into<ExprNode<'a>>>(self, expr: T) -> JoinConstraintNode<'a> {
        JoinConstraintNode::new(self, expr)
    }

    pub fn join(self, chain_name: &str, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            None,
            JoinOperatorType::Inner,
        )
    }

    pub fn join_as(self, chain_name: &str, table_name: &str, alias: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Inner,
        )
    }

    pub fn left_join(self, chain_name: &str, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            None,
            JoinOperatorType::Left,
        )
    }

    pub fn left_join_as(self, chain_name: &str, table_name: &str, alias: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Left,
        )
    }

    pub fn hash_executor<T: Into<ExprNode<'a>>, U: Into<ExprNode<'a>>>(
        self,
        key_expr: T,
        value_expr: U,
    ) -> HashJoinNode<'a> {
        HashJoinNode::new(self, key_expr, value_expr)
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

    pub fn alias_as(self, table_alias: &'a str) -> ChainFactorNode<'a> {
        QueryNode::JoinNode(self).alias_as(table_alias)
    }

    pub fn prebuild_for_constraint(self) -> Result<JoinConstraintData> {
        Ok(JoinConstraintData {
            select: self.prev_node.prebuild()?,
            relation: self.relation,
            operator_type: self.join_operator_type,
            executor: JoinExecutor::NestedLoop,
        })
    }

    pub fn prebuild_for_hash_join(self) -> Result<(Select, TableFactor, JoinOperator)> {
        let select_data = self.prev_node.prebuild()?;
        let join_operator = JoinOperator::from(self.join_operator_type);

        Ok((select_data, self.relation, join_operator))
    }
}

impl<'a> Prebuild<Select> for JoinNode<'a> {
    fn prebuild(self) -> Result<Select> {
        let mut select: Select = self.prev_node.prebuild()?;

        select.from.joins.push(Join {
            relation: self.relation,
            join_operator: JoinOperator::from(self.join_operator_type),
            join_executor: JoinExecutor::NestedLoop,
        });

        Ok(select)
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::ast_builder::{chain, test, Build},
        pretty_assertions::assert_eq,
    };

    #[test]
    fn inner_join() {
        // select node -> join node -> join constraint node
        let actual = chain("sui")
            .select("transactions")
            .join_as("blocks", "b")
            .on("b.digest = transactions.digest")
            .filter("p.id = 1")
            .build();
        let expected = "
        SELECT * FROM sui.transactions INNER JOIN blocks AS b ON b.digest = transactions.digest WHERE b.transaction = 1;
        ";
        test(actual, expected);

        // select node -> join node ->  join constraint node
        let actual = chain("sui")
            .select("transactions")
            .join_as("Player", "p")
            .on("p.id = Item.player_id")
            .filter("p.id = 1")
            .project(vec!["p.id", "p.name", "Item.id"])
            .build();
        let expected = "
        SELECT p.id, p.name, Item.id FROM Item INNER JOIN Player AS p ON p.id = Item.player_id WHERE p.id = 1;
        ";
        test(actual, expected);

        // select node -> join node ->  build
        let actual = chain("sui")
            .select("transactions")
            .join_as("trades", "t")
            .build();
        let expected = "
        SELECT * FROM sui.transactions INNER JOIN trades AS t;
        ";
        test(actual, expected);

        // join node -> join constraint node -> join node -> join constraint node
        let actual = chain("sui")
            .select("transactions")
            .join("marks")
            .on("transactions.id = marks.id")
            .join("sui", "attendance")
            .on("marks.id = attendance.id")
            .filter("attendance.attendance >= 75")
            .project(vec![
                "transactions.checkpoint",
                "transactions.digest",
                "marks.rank",
                "attendance.attendance",
            ])
            .build();
        let expected = "
            SELECT transactions.checkpoint, transactions.digest, marks.rank, attendance.attendance
            FROM  sui.transactions
            INNER JOIN marks ON transactions.id=marks.id
            INNER JOIN sui.attendance on marks.id=attendance.id
            WHERE attendance.attendance >= 75;
        ";
        test(actual, expected);

        // select node -> join node -> project node
        let actual = chain("arb")
            .select("Orders")
            .join("Customers")
            .project(vec![
                "Orders.OrderID",
                "Customers.CustomerName",
                "Orders.OrderDate",
            ])
            .build();
        let expected = "
            SELECT Orders.OrderID, Customers.CustomerName, Orders.OrderDate 
            FROM arb.Orders INNER JOIN Customers
        ";
        test(actual, expected);
    }

    #[test]
    fn left_join() {
        // select node -> left join node -> join constraint node
        let actual = chain("mina")
            .select("player")
            .left_join("item")
            .on("player.id = item.id")
            .project(vec!["player.id", "item.id"])
            .build();
        let expected = "
            SELECT player.id, item.id
            FROM mina.player
            LEFT JOIN item
            ON player.id = item.id
        ";
        test(actual, expected);

        // select node -> left join node -> join constraint node -> left join node
        let actual = chain("sui")
            .select("Item")
            .left_join("Player")
            .on("Player.id = Item.player_id")
            .left_join_as("sui", "Player", "p1")
            .on("p1.id = Item.player_id")
            .left_join_as("sui", "Player", "p2")
            .on("p2.id = Item.player_id")
            .left_join_as("sui", "Player", "p3")
            .on("p3.id = Item.player_id")
            .left_join_as("sui", "Player", "p4")
            .on("p4.id = Item.player_id")
            .left_join_as("sui", "Player", "p5")
            .on("p5.id = Item.player_id")
            .left_join_as("sui", "Player", "p6")
            .on("p6.id = Item.player_id")
            .left_join_as("sui", "Player", "p7")
            .on("p7.id = Item.player_id")
            .left_join_as("sui", "Player", "p8")
            .on("p8.id = Item.player_id")
            .left_join_as("sui", "Player", "p9")
            .on("p9.id = Item.player_id")
            .filter("Player.id = 1")
            .build();
        let expected = "
            SELECT * FROM sui.Item
            LEFT JOIN Player ON Player.id = Item.player_id
            LEFT JOIN Player p1 ON p1.id = Item.player_id
            LEFT JOIN Player p2 ON p2.id = Item.player_id
            LEFT JOIN Player p3 ON p3.id = Item.player_id
            LEFT JOIN Player p4 ON p4.id = Item.player_id
            LEFT JOIN Player p5 ON p5.id = Item.player_id
            LEFT JOIN Player p6 ON p6.id = Item.player_id
            LEFT JOIN Player p7 ON p7.id = Item.player_id
            LEFT JOIN Player p8 ON p8.id = Item.player_id
            LEFT JOIN Player p9 ON p9.id = Item.player_id
            WHERE Player.id = 1;
        ";
        test(actual, expected);

        // select node -> left join node -> join constraint node -> left join node
        let actual = chain("base")
            .select("item")
            .left_join("Player")
            .on("Player.id = Item.player_id")
            .left_join("sui", "Player")
            .on("p1.id = Item.player_id")
            .build();
        let expected = "
            SELECT * FROM base.Item
            LEFT JOIN Player ON Player.id = Item.player_id
            LEFT JOIN Player ON p1.id = Item.player_id";
        test(actual, expected);

        //     let actual = chain("base")
        //         .select("T tem")
        //         .left_join("Player")
        //         .on("Player.id = Item.player_id")
        //         .left_join_as("Player", "p1")
        //         .on("p1.id = Item.player_id")
        //         .left_join_as("Player", "p2")
        //         .on("p2.id = Item.player_id")
        //         .left_join_as("Player", "p3")
        //         .on("p3.id = Item.player_id")
        //         .join_as("Player", "p4")
        //         .on("p4.id = Item.player_id AND Item.id > 101")
        //         .filter("Player.id = 1")
        //         .build();
        //     let expected = "
        //         SELECT * FROM base.Item
        //         LEFT JOIN Player ON Player.id = Item.player_id
        //         LEFT JOIN Player p1 ON p1.id = Item.player_id
        //         LEFT JOIN Player p2 ON p2.id = Item.player_id
        //         LEFT JOIN Player p3 ON p3.id = Item.player_id
        //         INNER JOIN Player p4 ON p4.id = Item.player_id AND Item.id > 101
        //         WHERE Player.id = 1;
        //     ";
        //     test(actual, expected);
        // }

        // #[test]
        // fn join_join() {
        //     // join - join
        //     let actual = chain("sui").select("Foo").join("Bar").join("Baz").build();
        //     let expected = "
        //         SELECT * FROM sui.Foo
        //         INNER JOIN Bar
        //         INNER JOIN Baz
        //         ";
        //     test(actual, expected);

        //     // join - join as
        //     let actual = chain("sui")
        //         .select("Foo")
        //         .join("Bar")
        //         .join_as("sui", "Baz", "B")
        //         .build();
        //     let expected = "
        //         SELECT * FROM sui.Foo
        //         INNER JOIN sui.Bar
        //         INNER JOIN sui.Baz  B
        //         ";
        //     test(actual, expected);

        //     // join - left join
        //     let actual = chain("sui")
        //         .select("Foo")
        //         .join("Bar")
        //         .left_join("Baz")
        //         .build();
        //     let expected = "
        //         SELECT * FROM sui.Foo
        //         INNER JOIN Bar
        //         LEFT JOIN Baz
        //         ";
        //     test(actual, expected);

        //     // join - left join as
        //     let actual = chain("sui")
        //         .select("Foo")
        //         .join("Bar")
        //         .left_join_as("Baz", "B")
        //         .build();
        //     let expected = "
        //         SELECT * FROM sui.Foo
        //         INNER JOIN Bar
        //         LEFT JOIN Baz B
        //         ";
        //     test(actual, expected);

        // join as - join
        let actual = chain("sui")
            .select("Foo")
            .join_as("Bar", "B")
            .join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            INNER JOIN Bar B
            INNER JOIN sui.Baz
            ";
        test(actual, expected);

        // join as - join as
        let actual = chain("sui")
            .select("Foo")
            .join_as("Bar", "B")
            .join_as("sui", "Baz", "C")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            INNER JOIN Bar B
            INNER JOIN sui.Baz C
            ";
        test(actual, expected);

        // join as - left join
        let actual = chain("base")
            .select("transactions")
            .join_as("Bar", "B")
            .left_join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM base.transactions
            INNER JOIN Bar B
            LEFT JOIN sui.Baz
            ";
        test(actual, expected);

        // join as - left join as
        let actual = chain("sui")
            .select("Foo")
            .join_as("Bar", "B")
            .left_join_as("sui", "Baz", "C")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            INNER JOIN Bar B
            LEFT JOIN sui.Baz C
            ";
        test(actual, expected);

        // left join - join
        let actual = chain("sui")
            .select("Foo")
            .left_join("Bar")
            .join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar
            INNER JOIN sui.Baz
            ";
        test(actual, expected);

        // left join - join as
        let actual = chain("sui")
            .select("Foo")
            .left_join("Bar")
            .join_as("sui", "Baz", "B")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar
            INNER JOIN sui.Baz B
            ";
        test(actual, expected);

        // left join - left join
        let actual = chain("sui")
            .select("Foo")
            .left_join("Bar")
            .left_join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar
            LEFT JOIN sui.Baz
            ";
        test(actual, expected);

        // left join - left join as
        let actual = chain("sui")
            .select("Foo")
            .left_join("Bar")
            .left_join_as("sui", "Baz", "B")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar
            LEFT JOIN sui.Baz B
            ";
        test(actual, expected);

        // left join as - join
        let actual = chain("sui")
            .select("Foo")
            .left_join_as("Bar", "B")
            .join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar B
            INNER JOIN sui.Baz
            ";
        test(actual, expected);

        // left join as - join as
        let actual = chain("sui")
            .select("Foo")
            .left_join_as("Bar", "B")
            .join_as("sui", "Baz", "C")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar B
            INNER JOIN sui.Baz C
            ";
        test(actual, expected);

        // left join as - left join
        let actual = chain("sui")
            .select("Foo")
            .left_join_as("Bar", "B")
            .left_join("sui", "Baz")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar B
            LEFT JOIN sui.Baz
            ";
        test(actual, expected);

        // left join as - left join as
        let actual = chain("sui")
            .select("Foo")
            .left_join_as("Bar", "B")
            .left_join_as("sui", "Baz", "C")
            .build();
        let expected = "
            SELECT * FROM sui.Foo
            LEFT JOIN Bar B
            LEFT JOIN sui.Baz C
            ";
        test(actual, expected);
    }

    #[test]
    fn hash_join() {
        use crate::{
            ast::{
                Join, JoinConstraint, JoinExecutor, JoinOperator, Query, Select, SetExpr,
                Statement, TableAlias, TableFactor, TableWithJoins,
            },
            ast_builder::{col, SelectItemList},
        };

        let gen_expected = |other_join| {
            let join = Join {
                relation: TableFactor::Table {
                    chain_name: "sui".to_owned(),
                    name: "PlayerItem".to_owned(),
                    alias: None,
                    index: None,
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
                        chain_name: "sui".to_owned(),
                        name: "Player".to_owned(),
                        alias: None,
                        index: None,
                    },
                    joins: vec![join, other_join],
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

        let actual = chain("sui")
            .select("Player")
            .join("PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .join("sui", "OtherItem")
            .build();
        let expected = {
            let other_join = Join {
                relation: TableFactor::Table {
                    chain_name: "sui".to_owned(),
                    name: "OtherItem".to_owned(),
                    alias: None,
                    index: None,
                },
                join_operator: JoinOperator::Inner(JoinConstraint::None),
                join_executor: JoinExecutor::NestedLoop,
            };

            gen_expected(other_join)
        };
        assert_eq!(actual, expected, "inner join");

        let actual = chain("sui")
            .select("Player")
            .join("PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .join_as("sui", "OtherItem", "Ot")
            .build();
        let expected = {
            let other_join = Join {
                relation: TableFactor::Table {
                    chain_name: "sui".to_owned(),
                    name: "OtherItem".to_owned(),
                    alias: Some(TableAlias {
                        name: "Ot".to_owned(),
                        columns: Vec::new(),
                    }),
                    index: None,
                },
                join_operator: JoinOperator::Inner(JoinConstraint::None),
                join_executor: JoinExecutor::NestedLoop,
            };

            gen_expected(other_join)
        };
        assert_eq!(actual, expected, "inner join with alias");

        let actual = chain("sui")
            .select("Player")
            .join("PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .left_join("sui", "OtherItem")
            .build();
        let expected = {
            let other_join = Join {
                relation: TableFactor::Table {
                    chain_name: "sui".to_owned(),
                    name: "OtherItem".to_owned(),
                    alias: None,
                    index: None,
                },
                join_operator: JoinOperator::LeftOuter(JoinConstraint::None),
                join_executor: JoinExecutor::NestedLoop,
            };

            gen_expected(other_join)
        };
        assert_eq!(actual, expected, "left join");

        let actual = chain("sui")
            .select("Player")
            .join("PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .left_join_as("sui", "OtherItem", "Ot")
            .build();
        let expected = {
            let other_join = Join {
                relation: TableFactor::Table {
                    chain_name: "sui".to_owned(),
                    name: "OtherItem".to_owned(),
                    alias: Some(TableAlias {
                        name: "Ot".to_owned(),
                        columns: Vec::new(),
                    }),
                    index: None,
                },
                join_operator: JoinOperator::LeftOuter(JoinConstraint::None),
                join_executor: JoinExecutor::NestedLoop,
            };

            gen_expected(other_join)
        };
        assert_eq!(actual, expected, "left join with alias");

        let actual = chain("sui").select("App").alias_as("Sub").select().build();
        let expected = "SELECT * FROM (SELECT * FROM sui.App) Sub";
        test(actual, expected);

        // join -> derived subquery
        let actual = chain("sui")
            .select("Foo")
            .join("Bar")
            .alias_as("Sub")
            .select()
            .build();
        let expected = "
            SELECT * FROM (
                SELECT * FROM sui.Foo
                INNER JOIN Bar
            ) Sub
            ";
        test(actual, expected);
    }
}
