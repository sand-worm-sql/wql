mod build;
mod chain;
mod chain_factor;
mod error;
mod expr;
mod query;
mod select;
mod show_chain_entities;
mod show_chain_entities_columns;

pub use error::AstBuilderError;

/// Available aggregate or normal SQL functions
pub use expr::{
    aggregate::{avg, count, max, min, stdev, sum, variance, AggregateNode},
    function,
};

/// Available expression builder functions
pub use {
    chain::chain,
    expr::{
        bitwise_not, bytes, case, col, date, exists, expr, factorial, minus, nested, not,
        not_exists, null, num, numeric::NumericNode, plus, subquery, text, time, timestamp, uuid,
        ExprNode,
    },
    select::{
        select, values, FilterNode, GroupByNode, HashJoinNode, HavingNode, JoinConstraintNode,
        JoinNode, LimitNode, OffsetLimitNode, OffsetNode, OrderByNode, ProjectNode, SelectNode,
    },
    query::QueryNode,
    expr_with_alias::ExprWithAliasNode,
    show_chain_entities::ShowChainEntitiesNode,
    show_chain_entities_columns::ShowChainEntitiesColumnsNode,
};

pub use build::Build;

#[cfg(test)]
fn test(actual: crate::result::Result<crate::ast::Statement>, expected: &str) {
    use crate::{parse_sql::parse, translate::translate};

    let parsed = &parse(expected).expect(expected)[0];
    let expected = translate(parsed);
    pretty_assertions::assert_eq!(actual, expected);
}

#[cfg(test)]
fn test_expr(actual: crate::ast_builder::ExprNode, expected: &str) {
    use crate::{parse_sql::parse_expr, translate::translate_expr};

    let parsed = &parse_expr(expected).expect(expected);
    let expected = translate_expr(parsed);
    pretty_assertions::assert_eq!(actual.try_into(), expected);
}

#[cfg(test)]
fn test_query(actual: crate::ast_builder::QueryNode, expected: &str) {
    use crate::{parse_sql::parse_query, translate::translate_query};

    let parsed = &parse_query(expected).expect(expected);
    let expected = translate_query(parsed);
    pretty_assertions::assert_eq!(actual.try_into(), expected);
}
