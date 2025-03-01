use {
    super::{DataType, Expr},
    crate::ast::ToSql,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    /// `DEFAULT <restricted-expr>`
    pub default: Option<Expr>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperateFunctionArg {
    pub name: String,
    pub data_type: DataType,
    /// `DEFAULT <restricted-expr>`
    pub default: Option<Expr>,
}

impl ToSql for ColumnDef {
    fn to_sql(&self) -> String {
        let ColumnDef {
            name,
            data_type,
            nullable,
            default,
            comment,
        } = self;
        {
            let nullable = match nullable {
                true => "NULL",
                false => "NOT NULL",
            };
            let column_def = format!(r#""{name}" {data_type} {nullable}"#);
            let default = default
                .as_ref()
                .map(|expr| format!("DEFAULT {}", expr.to_sql()));
            let comment = comment
                .as_ref()
                .map(|comment| format!("COMMENT '{}'", comment));

            [Some(column_def), default, comment]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

impl ToSql for OperateFunctionArg {
    fn to_sql(&self) -> String {
        let OperateFunctionArg {
            name,
            data_type,
            default,
        } = self;
        let default = default
            .as_ref()
            .map(|expr| format!(" DEFAULT {}", expr.to_sql()))
            .unwrap_or_else(|| "".to_owned());
        format!(r#""{name}" {data_type}{default}"#)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{AstLiteral, ColumnDef, DataType, Expr, OperateFunctionArg, ToSql};

    #[test]
    fn to_sql_column_def() {
        assert_eq!(
            r#""name" TEXT NOT NULL"#,
            ColumnDef {
                name: "name".to_owned(),
                data_type: DataType::Text,
                nullable: false,
                default: None,
                comment: None,
            }
            .to_sql()
        );

        assert_eq!(
            r#""accepted" BOOLEAN NULL"#,
            ColumnDef {
                name: "accepted".to_owned(),
                data_type: DataType::Boolean,
                nullable: true,
                default: None,
                comment: None,
            }
            .to_sql()
        );

        assert_eq!(
            r#""accepted" BOOLEAN NOT NULL DEFAULT FALSE"#,
            ColumnDef {
                name: "accepted".to_owned(),
                data_type: DataType::Boolean,
                nullable: false,
                default: Some(Expr::Literal(AstLiteral::Boolean(false))),
                comment: None,
            }
            .to_sql()
        );

        assert_eq!(
            r#""accepted" BOOLEAN NOT NULL COMMENT 'this is comment'"#,
            ColumnDef {
                name: "accepted".to_owned(),
                data_type: DataType::Boolean,
                nullable: false,
                default: None,
                comment: Some("this is comment".to_owned()),
            }
            .to_sql()
        );
    }

    #[test]
    fn to_sql_operate_function_arg() {
        assert_eq!(
            r#""name" TEXT"#,
            OperateFunctionArg {
                name: "name".to_owned(),
                data_type: DataType::Text,
                default: None,
            }
            .to_sql()
        );

        assert_eq!(
            r#""accepted" BOOLEAN DEFAULT FALSE"#,
            OperateFunctionArg {
                name: "accepted".to_owned(),
                data_type: DataType::Boolean,
                default: Some(Expr::Literal(AstLiteral::Boolean(false))),
            }
            .to_sql()
        );
    }
}
