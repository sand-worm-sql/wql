mod ast_literal;
mod data_type;
mod ddl;
mod expr;
mod function;
mod operator;
mod query;

pub use {
    ast_literal::{AstLiteral, DateTimeField, TrimWhereField},
    data_type::DataType,
    ddl::*,
    expr::Expr,
    function::{Aggregate, CountArgExpr, Function},
    operator::*,
    query::*,
};

use {
    serde::{Deserialize, Serialize},
    strum_macros::Display,
};

pub trait ToSql {
    fn to_sql(&self) -> String;
}

pub trait ToSqlUnquoted {
    fn to_sql_unquoted(&self) -> String;
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize, Display)]
pub enum ReferentialAction {
    #[strum(to_string = "NO ACTION")]
    NoAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Statement {
    /// SELECT, VALUES
    Query(Query),

    /// SHOW VARIABLE
    Show(Show),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Show {
    ChainEntitiesColumns {
        chain_name: String,
        entity_name: String,
    },
    ChainEntities {
        chain_name: String,
    },
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Variable {
    Tables,
    Chains,
    Version,
}

impl ToSql for Statement {
    fn to_sql(&self) -> String {
        match self {
            Statement::Show(show_stmt) => match show_stmt {
                Show::ChainEntities { chain_name } => {
                    format!("SHOW CHAIN ENTITIES FROM {chain_name};")
                }
                Show::ChainEntitiesColumns {
                    chain_name,
                    entity_name,
                } => {
                    format!("SHOW CHAIN ENTITIES IN {entity_name} FROM {chain_name};")
                }
                Show::Variable(var) => match var {
                    Variable::Tables => "SHOW TABLES;".to_owned(),
                    Variable::Chains => "SHOW CHAINS;".to_owned(),
                    Variable::Version => "SHOW VERSIONS;".to_owned(),
                },
            },
            _ => "(..statement..)".to_owned(),
        }
    }
}

impl ToSql for Assignment {
    fn to_sql(&self) -> String {
        format!(r#""{}" = {}"#, self.id, self.value.to_sql())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Array {
    pub elem: Vec<Expr>,
    pub named: bool,
}

#[cfg(test)]
mod tests {
    use {
        crate::ast::{Assignment, AstLiteral, Expr, Show, Statement, ToSql, Variable},
        bigdecimal::BigDecimal,
        std::str::FromStr,
    };

    #[test]
    fn to_sql_show_columns() {
        assert_eq!(
            "SHOW CHAIN ENTITIES FROM base;",
            Statement::Show(Show::ChainEntities {
                chain_name: "base".into()
            })
            .to_sql()
        )
    }

    #[test]
    fn to_sql_show_variable() {
        assert_eq!(
            "SHOW CHAINS;",
            Statement::Show(Show::Variable(Variable::Chains)).to_sql()
        );
        assert_eq!(
            "SHOW VERSIONS;",
            Statement::Show(Show::Variable(Variable::Version)).to_sql()
        );
    }

    #[test]
    fn to_sql_assignment() {
        assert_eq!(
            r#""count" = 5"#,
            Assignment {
                id: "count".to_owned(),
                value: Expr::Literal(AstLiteral::Number(BigDecimal::from_str("5").unwrap()))
            }
            .to_sql()
        )
    }
}
