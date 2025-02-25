use {
    super::{
        data_type::translate_data_type, expr::translate_expr, TranslateError,
    },
    crate::{
        ast::{ ColumnDef,  OperateFunctionArg},
        result::Result,
    },
    sqlparser::ast::{
        ColumnDef as SqlColumnDef,ColumnOption as SqlColumnOption,
        OperateFunctionArg as SqlOperateFunctionArg,
    },
};


pub fn translate_column_def(sql_column_def: &SqlColumnDef) -> Result<ColumnDef> {
    let SqlColumnDef {
        name,
        data_type,
        options,
        ..
    } = sql_column_def;

    let mut nullable = true;
    let mut default = None;
    let mut comment = None;

    for option_def in options {
        match &option_def.option {
            SqlColumnOption::Null => {} // Default is already nullable
            SqlColumnOption::NotNull => nullable = false,
            SqlColumnOption::Default(value) => {
                default = Some(translate_expr(value)?);
            }
            SqlColumnOption::Comment(value) => {
                comment = Some(value.to_string());
            }
            _ => {
                return Err(TranslateError::UnsupportedColumnOption(option_def.option.to_string()).into());
            }
        }
    }

    Ok(ColumnDef {
        name: name.value.to_owned(),
        data_type: translate_data_type(data_type)?,
        nullable,
        default,
        comment,
    })
}


pub fn translate_operate_function_arg(arg: &SqlOperateFunctionArg) -> Result<OperateFunctionArg> {
    let name = arg
        .name
        .as_ref()
        .map(|v| v.value.to_owned())
        .ok_or(TranslateError::UnNamedFunctionArgNotSupported)?;
    let data_type = translate_data_type(&arg.data_type)?;
    let default = arg.default_expr.as_ref().map(translate_expr).transpose()?;
    Ok(OperateFunctionArg {
        name,
        data_type,
        default,
    })
}