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
    ShowChainEntitiesColumns {
        chain_name: String,
        entity_name: String,
    },
    ShowChainEntities {
        chain_name: String,
    },
    /// SELECT, VALUES
    Query(Query),

    /// CREATE FUNCTION
    CreateFunction {
        or_replace: bool,
        name: String,
        /// Optional schema
        args: Vec<OperateFunctionArg>,
        return_: Expr,
    },

    /// DROP FUNCTION
    DropFunction {
        /// An optional `IF EXISTS` clause. (Non-standard.)
        if_exists: bool,
        /// One or more objects to drop. (ANSI SQL requires exactly one.)
        names: Vec<String>,
    },
    /// CREATE INDEX
    CreateIndex {
        name: String,
        table_name: String,
        column: OrderByExpr,
    },
    /// DROP INDEX
    DropIndex {
        name: String,
        table_name: String,
    },
    /// START TRANSACTION, BEGIN
    StartTransaction,
    /// COMMIT
    Commit,
    /// ROLLBACK
    Rollback,
    /// SHOW VARIABLE
    ShowVariable(Variable),
    ShowIndexes(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Variable {
    Chains,
    Functions,
    Version,
}

impl ToSql for Statement {
    fn to_sql(&self) -> String {
        match self {
            Statement::ShowChainEntities { chain_name } => {
                format!("SHOW CHAIN ENTITIES FROM {chain_name};")
            }
            Statement::CreateFunction {
                or_replace,
                name,
                args,
                return_,
                ..
            } => {
                let or_replace = or_replace.then_some(" OR REPLACE").unwrap_or("");
                let args = args
                    .iter()
                    .map(ToSql::to_sql)
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_ = format!(" RETURN {}", return_.to_sql());
                format!("CREATE{or_replace} FUNCTION {name}({args}){return_};")
            }
            Statement::DropFunction { if_exists, names } => {
                let names = names.join(", ");
                match if_exists {
                    true => format!("DROP FUNCTION IF EXISTS {};", names),
                    false => format!("DROP FUNCTION {};", names),
                }
            }
            Statement::CreateIndex {
                name,
                table_name,
                column,
            } => {
                format!(
                    r#"CREATE INDEX "{name}" ON "{table_name}" ({});"#,
                    column.to_sql()
                )
            }
            Statement::DropIndex { name, table_name } => {
                format!("DROP INDEX {table_name}.{name};")
            }
            Statement::StartTransaction => "START TRANSACTION;".to_owned(),
            Statement::Commit => "COMMIT;".to_owned(),
            Statement::Rollback => "ROLLBACK;".to_owned(),
            Statement::ShowVariable(variable) => match variable {
                Variable::Chains => "SHOW CHAINS;".to_owned(),
                Variable::Functions => "SHOW FUNCTIONS;".to_owned(),
                Variable::Version => "SHOW VERSIONS;".to_owned(),
            },
            Statement::ShowIndexes(object_name) => {
                format!(r#"SHOW INDEXES FROM "{object_name}";"#)
            }
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
        crate::ast::{
            Assignment, AstLiteral, BinaryOperator, ColumnDef, DataType, Expr,
            OperateFunctionArg, OrderByExpr, Query, ReferentialAction, Select, SelectItem, SetExpr,
            Statement, TableFactor, TableWithJoins, ToSql, Values, Variable,
        },
        bigdecimal::BigDecimal,
        std::str::FromStr,
    };

    #[test]
    fn to_sql_show_columns() {
        assert_eq!(
            "SHOW CHAIN ENTITIES FROM base;",
            Statement::ShowChainEntities {
                chain_name: "base".into()
            }
            .to_sql()
        )
    }

    #[test]
    fn to_sql_create_index() {
        assert_eq!(
            r#"CREATE INDEX "idx_name" ON "Test" ("LastName");"#,
            Statement::CreateIndex {
                name: "idx_name".into(),
                table_name: "Test".into(),
                column: OrderByExpr {
                    expr: Expr::Identifier("LastName".to_owned()),
                    asc: None
                }
            }
            .to_sql()
        );
    }

    #[test]
    fn to_sql_drop_index() {
        assert_eq!(
            "DROP INDEX Test.idx_id;",
            Statement::DropIndex {
                name: "idx_id".into(),
                table_name: "Test".into(),
            }
            .to_sql()
        )
    }

    #[test]
    fn to_sql_transaction() {
        assert_eq!("START TRANSACTION;", Statement::StartTransaction.to_sql());
        assert_eq!("COMMIT;", Statement::Commit.to_sql());
        assert_eq!("ROLLBACK;", Statement::Rollback.to_sql());
    }

    #[test]
    fn to_sql_show_variable() {
        assert_eq!(
            "SHOW CHAINS;",
            Statement::ShowVariable(Variable::Chains).to_sql()
        );
        assert_eq!(
            "SHOW FUNCTIONS;",
            Statement::ShowVariable(Variable::Functions).to_sql()
        );
        assert_eq!(
            "SHOW VERSIONS;",
            Statement::ShowVariable(Variable::Version).to_sql()
        );
    }

    #[test]
    fn to_sql_show_indexes() {
        assert_eq!(
            r#"SHOW INDEXES FROM "Test";"#,
            Statement::ShowIndexes("Test".into()).to_sql()
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
