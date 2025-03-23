use {
    super::{join::JoinOperatorType, Prebuild},
    crate::{
        ast::{
            AstLiteral, Expr, Query, Select, SelectItem, TableAlias, TableFactor, TableWithJoins,
        },
        ast_builder::{
            chain_factor::ChainQueryType, ChainFactorNode, ExprList, ExprNode, FilterNode,
            GroupByNode, JoinNode, LimitNode, OffsetNode, OrderByExprList, OrderByNode,
            ProjectNode, QueryNode, SelectItemList,
        },
        result::Result,
        translate::alias_or_name,
    },
};

#[derive(Clone, Debug)]
pub struct SelectNode<'a> {
    chain_node: ChainFactorNode<'a>,
}

impl<'a> SelectNode<'a> {
    pub fn new(chain_node: ChainFactorNode<'a>) -> Self {
        Self { chain_node }
    }

    pub fn filter<T: Into<ExprNode<'a>>>(self, expr: T) -> FilterNode<'a> {
        FilterNode::new(self, expr)
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

    pub fn project<T: Into<SelectItemList<'a>>>(self, select_items: T) -> ProjectNode<'a> {
        ProjectNode::new(self, select_items)
    }

    pub fn order_by<T: Into<OrderByExprList<'a>>>(self, order_by_exprs: T) -> OrderByNode<'a> {
        OrderByNode::new(self, order_by_exprs)
    }

    pub fn join(self, chain_name: Option<&str>, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.?to_owned(),
            table_name.to_owned(),
            None,
            JoinOperatorType::Inner,
        )
    }

    pub fn join_as(self, chain_name: Option<&str>,  table_name: &str, alias: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Inner,
        )
    }

    pub fn left_join(self, chain_name: Option<&str>, table_name: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            None,
            JoinOperatorType::Left,
        )
    }

    pub fn left_join_as(self, chain_name: Option<&str>, table_name: &str, alias: &str) -> JoinNode<'a> {
        JoinNode::new(
            self,
            chain_name.to_owned(),
            table_name.to_owned(),
            Some(alias.to_owned()),
            JoinOperatorType::Left,
        )
    }

    pub fn alias_as(self, alias: &'a str) -> ChainFactorNode<'a> {
        QueryNode::SelectNode(self).alias_as(alias)
    }
}

impl<'a> Prebuild<Select> for SelectNode<'a> {
    fn prebuild(self) -> Result<Select> {
        let alias = self.chain_node.table_alias.map(|name| TableAlias {
            name,
            columns: Vec::new(),
        });

        let index = match self.chain_node.index {
            Some(index) => Some(index.prebuild()?),
            None => None,
        };

        let relation = match self.chain_node.chain_query_type {
            ChainQueryType::Table => TableFactor::Table {
                chain_name: self.chain_node.chain_name,
                name: self.chain_node.entity_name.unwrap(),
                alias,
                index,
            },
            ChainQueryType::Dictionary(dict) => TableFactor::Dictionary {
                dict,
                alias: alias_or_name(alias, self.chain_node.chain_name),
            },
            ChainQueryType::Series(args) => TableFactor::Series {
                alias: alias_or_name(alias, self.chain_node.chain_name),
                size: args.try_into()?,
            },
            ChainQueryType::Derived { subquery, alias } => TableFactor::Derived {
                subquery: Query::try_from(*subquery)?,
                alias: TableAlias {
                    name: alias,
                    columns: Vec::new(),
                },
            },
        };

        let from = TableWithJoins {
            relation,
            joins: Vec::new(),
        };

        Ok(Select {
            projection: vec![SelectItem::Wildcard],
            from,
            selection: None,
            group_by: Vec::new(),
            having: None,
        })
    }
}

pub fn select<'a>() -> SelectNode<'a> {
    SelectNode {
        chain_node: ChainFactorNode {
            chain_name: "Series".to_owned(),
            chain_query_type: ChainQueryType::Series(
                Expr::Literal(AstLiteral::Number(1.into())).into(),
            ),
            entity_name: None,
            table_alias: None,
            index: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_builder::{chain, select, test, Build};

    #[test]
    fn select_root() {
        // select node -> build
        let actual = chain("sui").select("transations").build();
        let expected = "SELECT * FROM sui.transations";
        test(actual, expected);

        // select -> derived subquery
        let actual = chain("sui")
            .select("transations")
            .alias_as("transations")
            .select()
            .build();
        let expected = "SELECT * FROM (SELECT * FROM sui.transations) transations";
        test(actual, expected);

        // select without table
        let actual = select().project("1 + 1").build();
        let expected = "SELECT 1 + 1";
        test(actual, expected);
    }
}
