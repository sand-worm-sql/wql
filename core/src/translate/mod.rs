mod ast_literal;
mod data_type;
mod ddl;
mod error;
mod expr;
mod function;
mod operator;
mod query;

use crate::ast::Show;

pub use self::{
    data_type::translate_data_type,
    ddl::{translate_column_def, translate_operate_function_arg},
    error::TranslateError,
    expr::{translate_expr, translate_order_by_expr},
    query::{alias_or_name, translate_query, translate_select_item},
};

use {
    crate::{
        ast::{Assignment, ReferentialAction, Statement, Variable},
        result::Result,
    },
    sqlparser::ast::{
        Assignment as SqlAssignment, AssignmentTarget as SqlAssignmentTarget,
        Ident as SqlIdent, ObjectName as SqlObjectName,
        ReferentialAction as SqlReferentialAction, Statement as SqlStatement,TableFactor, TableWithJoins,
    },
};

pub fn translate(sql_statement: &SqlStatement) -> Result<Statement> {
    match sql_statement {
        SqlStatement::Query(query) => translate_query(query).map(Statement::Query),
        SqlStatement::ShowTables {
            filter: None,
            db_name: None,
            ..
        } => Ok(Statement::Show(Show::Variable(Variable::Tables))),
        SqlStatement::ShowVariable { variable } => translate_show_variable(variable, sql_statement),
        _ => Err(TranslateError::UnsupportedStatement(sql_statement.to_string()).into()),
    }
}

fn translate_show_variable(
    variable: &[SqlIdent],
    sql_statement: &SqlStatement,
) -> Result<Statement> {
    match (variable.len(), variable.first()) {
        (1, Some(keyword)) => match keyword.value.to_uppercase().as_str() {
            "VERSION" => Ok(Statement::Show(Show::Variable(Variable::Version))),
            "CHAINS" => Ok(Statement::Show(Show::Variable(Variable::Chains))),
            "TABLES" => Ok(Statement::Show(Show::Variable(Variable::Tables))),
            v => Err(TranslateError::UnsupportedShowVariableKeyword(v.to_owned()).into()),
        },
        (4, Some(keyword)) if keyword.value.eq_ignore_ascii_case("CHAIN") => {
            let entity_keyword = variable.get(1).map(|v| v.value.to_uppercase());
            let from_keyword = variable.get(2).map(|v| v.value.to_uppercase());
            let chain_name = variable.get(3).map(|v| v.value.clone());

            match (
                entity_keyword.as_deref(),
                from_keyword.as_deref(),
                chain_name,
            ) {
                (Some("ENTITIES"), Some("FROM"), Some(chain)) => {
                    Ok(Statement::Show(Show::ChainEntities { chain_name: chain }))
                }
                _ => unsupported_show_variable(sql_statement),
            }
        }
        (6, Some(keyword)) if keyword.value.eq_ignore_ascii_case("CHAIN") => {
            let subcommand = variable.get(1).map(|v| v.value.to_uppercase()); // ENTITIES or COLUMNS
            let in_keyword = variable.get(2).map(|v| v.value.to_uppercase()); // IN
            let entity_name = variable.get(3).map(|v| v.value.clone());
            let from_keyword = variable.get(4).map(|v| v.value.to_uppercase()); // FROM
            let chain_name = variable.get(5).map(|v| v.value.clone()); // block

            match (
                subcommand.as_deref(),
                in_keyword.as_deref(),
                from_keyword.as_deref(),
                chain_name,
                entity_name,
            ) {
                (Some("ENTITIES"), Some("IN"), Some("FROM"), Some(chain), Some(entity)) => {
                    Ok(Statement::Show(Show::ChainEntitiesColumns {
                        chain_name: chain,
                        entity_name: entity,
                    }))
                }
                _ => unsupported_show_variable(sql_statement),
            }
        }
        _ => unsupported_show_variable(sql_statement),
    }
}

fn unsupported_show_variable(sql_statement: &SqlStatement) -> Result<Statement> {
    Err(TranslateError::UnsupportedShowVariableStatement(sql_statement.to_string()).into())
}

pub fn translate_assignment(sql_assignment: &SqlAssignment) -> Result<Assignment> {
    let SqlAssignment { target, value } = sql_assignment;

    let id = match target {
        SqlAssignmentTarget::Tuple(_) => {
            return Err(TranslateError::TupleAssignmentOnUpdateNotSupported(
                sql_assignment.to_string(),
            )
            .into());
        }
        SqlAssignmentTarget::ColumnName(SqlObjectName(id)) => id,
    };

    if id.len() > 1 {
        return Err(
            TranslateError::CompoundIdentOnUpdateNotSupported(sql_assignment.to_string()).into(),
        );
    }

    Ok(Assignment {
        id: id
            .first()
            .ok_or(TranslateError::UnreachableEmptyIdent)?
            .value
            .to_owned(),
        value: translate_expr(value)?,
    })
}

fn translate_table_with_join(table: &TableWithJoins) -> Result<String> {
    if !table.joins.is_empty() {
        return Err(TranslateError::JoinOnUpdateNotSupported.into());
    }
    match &table.relation {
        TableFactor::Table { name, .. } => translate_object_name(name),
        t => Err(TranslateError::UnsupportedTableFactor(t.to_string()).into()),
    }
}

fn translate_object_name(sql_object_name: &SqlObjectName) -> Result<String> {
    let sql_object_name = &sql_object_name.0;
    if sql_object_name.len() > 2 {
        let compound_object_name = translate_idents(sql_object_name).join(".");
        return Err(TranslateError::CompoundObjectNotSupported(compound_object_name).into());
    }

    sql_object_name
        .first()
        .map(|v| v.value.to_owned())
        .ok_or_else(|| TranslateError::UnreachableEmptyObject.into())
}

fn translate_chain_and_table(sql_object_name: &SqlObjectName) -> Result<(Option<String>, String)> {
    let sql_object_name = &sql_object_name.0;
    if sql_object_name.len() > 3 {
        let compound_object_name = translate_idents(sql_object_name).join(".");
        return Err(TranslateError::CompoundObjectNotSupported(compound_object_name).into());
    }

    match (sql_object_name.first(), sql_object_name.get(1)) {
        (Some(chain), Some(table)) => Ok((Some(chain.value.to_owned()), table.value.to_owned())),
        (Some(table), None) => Ok((None, table.value.to_owned())),
        _ => Err(TranslateError::UnreachableEmptyObject.into()),
    }
}

pub fn translate_idents(idents: &[SqlIdent]) -> Vec<String> {
    idents.iter().map(|v| v.value.to_owned()).collect()
}

pub fn translate_referential_action(
    action: &Option<SqlReferentialAction>,
) -> Result<ReferentialAction> {
    use SqlReferentialAction::*;

    let action = action.unwrap_or(NoAction);

    match action {
        NoAction | Restrict => Ok(ReferentialAction::NoAction),
        _ => Err(TranslateError::UnsupportedConstraint(action.to_string()).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;

    #[test]
    fn test_translate_show_tables() {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, "SHOW TABLES").unwrap();
        let stmt = translate(&ast[0]);
        assert!(matches!(
            stmt,
            Ok(Statement::Show(Show::Variable(Variable::Tables)))
        ));
    }
}
