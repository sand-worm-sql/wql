mod ast_literal;
mod data_type;
mod ddl;
mod error;
mod expr;
mod function;
mod operator;
mod query;

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
        CreateFunctionBody as SqlCreateFunctionBody, CreateIndex as SqlCreateIndex,
        Ident as SqlIdent, ObjectName as SqlObjectName, ObjectType as SqlObjectType,
        ReferentialAction as SqlReferentialAction, Statement as SqlStatement,
        TableConstraint as SqlTableConstraint, TableFactor, TableWithJoins,
    },
};

pub fn translate(sql_statement: &SqlStatement) -> Result<Statement> {
    match sql_statement {
        SqlStatement::Query(query) => translate_query(query).map(Statement::Query),
        SqlStatement::DropFunction {
            if_exists,
            func_desc,
            ..
        } => Ok(Statement::DropFunction {
            if_exists: *if_exists,
            names: func_desc
                .iter()
                .map(|v| translate_object_name(&v.name))
                .collect::<Result<Vec<_>>>()?,
        }),
        SqlStatement::CreateIndex(SqlCreateIndex {
            name,
            table_name,
            columns,
            ..
        }) => {
            if columns.len() > 1 {
                return Err(TranslateError::CompositeIndexNotSupported.into());
            }

            let Some(name) = name else {
                return Err(TranslateError::UnsupportedUnnamedIndex.into());
            };

            let name = translate_object_name(name)?;

            if name.to_uppercase() == "PRIMARY" {
                return Err(TranslateError::ReservedIndexName(name).into());
            };

            Ok(Statement::CreateIndex {
                name,
                table_name: translate_object_name(table_name)?,
                column: translate_order_by_expr(&columns[0])?,
            })
        }
        SqlStatement::Drop {
            object_type: SqlObjectType::Index,
            names,
            ..
        } => {
            if names.len() > 1 {
                return Err(TranslateError::TooManyParamsInDropIndex.into());
            }

            let object_name = &names[0].0;
            if object_name.len() != 2 {
                return Err(TranslateError::InvalidParamsInDropIndex.into());
            }

            let table_name = object_name[0].value.to_owned();
            let name = object_name[1].value.to_owned();

            if name.to_uppercase() == "PRIMARY" {
                return Err(TranslateError::CannotDropPrimary.into());
            };

            Ok(Statement::DropIndex { name, table_name })
        }
        SqlStatement::StartTransaction { .. } => Ok(Statement::StartTransaction),
        SqlStatement::Commit { .. } => Ok(Statement::Commit),
        SqlStatement::Rollback { .. } => Ok(Statement::Rollback),
        SqlStatement::ShowTables {
            filter: None,
            db_name: None,
            ..
        } => Ok(Statement::ShowVariable(Variable::Chains)),
        SqlStatement::ShowFunctions { filter: None } => {
            Ok(Statement::ShowVariable(Variable::Functions))
        }
        SqlStatement::ShowVariable { variable } => match (variable.len(), variable.first()) {
            (1, Some(keyword)) => match keyword.value.to_uppercase().as_str() {
                "VERSION" => Ok(Statement::ShowVariable(Variable::Version)),
                v => Err(TranslateError::UnsupportedShowVariableKeyword(v.to_owned()).into()),
            },
            (3, Some(keyword)) => match keyword.value.to_uppercase().as_str() {
                "INDEXES" => match variable.get(2) {
                    Some(tablename) => Ok(Statement::ShowIndexes(tablename.value.to_owned())),
                    _ => Err(TranslateError::UnsupportedShowVariableStatement(
                        sql_statement.to_string(),
                    )
                    .into()),
                },
                _ => Err(TranslateError::UnsupportedShowVariableStatement(
                    sql_statement.to_string(),
                )
                .into()),
            },
            _ => Err(
                TranslateError::UnsupportedShowVariableStatement(sql_statement.to_string()).into(),
            ),
        },
        // SqlStatement::ShowColumns { table_name, .. } => Ok(Statement::ShowColumns {
        //     table_name: translate_object_name(table_name)?,
        // }),
        SqlStatement::CreateFunction {
            or_replace,
            name,
            args,
            function_body: Some(SqlCreateFunctionBody::Return(return_)),
            ..
        } => {
            let args = args
                .as_ref()
                .map(|args| {
                    args.iter()
                        .map(translate_operate_function_arg)
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?;
            Ok(Statement::CreateFunction {
                or_replace: *or_replace,
                name: translate_object_name(name)?,
                args: args.unwrap_or_default(),
                return_: translate_expr(return_)?,
            })
        }
        SqlStatement::CreateFunction { .. } => {
            Err(TranslateError::UnsupportedEmptyFunctionBody.into())
        }

        _ => Err(TranslateError::UnsupportedStatement(sql_statement.to_string()).into()),
    }
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
mod tests {}
