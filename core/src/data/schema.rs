use {
    crate::ast::{ColumnDef, Expr},
    chrono::NaiveDateTime,
    serde::{Deserialize, Serialize},
    std::fmt::Debug,
    strum_macros::Display,
    thiserror::Error as ThisError,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaIndexOrd {
    Asc,
    Desc,
    Both,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaIndex {
    pub name: String,
    pub expr: Expr,
    pub order: SchemaIndexOrd,
    pub created: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Schema {
    pub table_name: String,
    pub column_defs: Option<Vec<ColumnDef>>,
    pub indexes: Vec<SchemaIndex>,
    pub comment: Option<String>,
    pub chain_name: Option<String>,
}

#[derive(ThisError, Debug, PartialEq, Serialize)]
pub enum SchemaParseError {
    #[error("cannot parse ddl")]
    CannotParseDDL,
}

// #[cfg(test)]
// mod tests {
//     use {
//         super::SchemaParseError,
//         crate::{
//             ast::{AstLiteral, ColumnDef, ColumnUniqueOption, Expr},
//             chrono::Utc,
//             data::{Schema, SchemaIndex, SchemaIndexOrd},
//             prelude::DataType,
//         },
//     };

//     fn assert_schema(actual: Schema, expected: Schema) {
//         let Schema {
//             table_name,
//             column_defs,
//             indexes,
//             engine,
//             foreign_keys,
//             comment,
//         } = actual;

//         let Schema {
//             table_name: table_name_e,
//             column_defs: column_defs_e,
//             indexes: indexes_e,
//             engine: engine_e,
//             foreign_keys: foreign_keys_e,
//             comment: comment_e,
//         } = expected;

//         assert_eq!(table_name, table_name_e);
//         assert_eq!(column_defs, column_defs_e);
//         assert_eq!(engine, engine_e);
//         assert_eq!(foreign_keys, foreign_keys_e);
//         assert_eq!(comment, comment_e);
//         indexes
//             .into_iter()
//             .zip(indexes_e)
//             .for_each(|(actual, expected)| assert_index(actual, expected));
//     }

//     fn assert_index(actual: SchemaIndex, expected: SchemaIndex) {
//         let SchemaIndex {
//             name, expr, order, ..
//         } = actual;
//         let SchemaIndex {
//             name: name_e,
//             expr: expr_e,
//             order: order_e,
//             ..
//         } = expected;

//         assert_eq!(name, name_e);
//         assert_eq!(expr, expr_e);
//         assert_eq!(order, order_e);
//     }

//     #[test]
//     fn table_basic() {
//         let schema = Schema {
//             table_name: "User".to_owned(),
//             column_defs: Some(vec![
//                 ColumnDef {
//                     name: "id".to_owned(),
//                     data_type: DataType::Int,
//                     nullable: false,
//                     default: None,
//                     unique: None,
//                     comment: None,
//                 },
//                 ColumnDef {
//                     name: "name".to_owned(),
//                     data_type: DataType::Text,
//                     nullable: true,
//                     default: Some(Expr::Literal(AstLiteral::QuotedString("worm".to_owned()))),
//                     unique: None,
//                     comment: None,
//                 },
//             ]),
//             indexes: Vec::new(),
//             engine: None,
//             foreign_keys: Vec::new(),
//             comment: None,
//         };

//         let ddl = r#"CREATE TABLE "User" ("id" INT NOT NULL, "name" TEXT NULL DEFAULT 'worm');"#;
//         assert_eq!(schema.to_ddl(), ddl);

//         let actual = Schema::from_ddl(ddl).unwrap();
//         assert_schema(actual, schema);

//         let schema = Schema {
//             table_name: "Test".to_owned(),
//             column_defs: None,
//             indexes: Vec::new(),
//             engine: None,
//             foreign_keys: Vec::new(),
//             comment: None,
//         };
//         let ddl = r#"CREATE TABLE "Test";"#;
//         assert_eq!(schema.to_ddl(), ddl);

//         let actual = Schema::from_ddl(ddl).unwrap();
//         assert_schema(actual, schema);
//     }

//     #[test]
//     fn table_primary() {
//         let schema = Schema {
//             table_name: "User".to_owned(),
//             column_defs: Some(vec![ColumnDef {
//                 name: "id".to_owned(),
//                 data_type: DataType::Int,
//                 nullable: false,
//                 default: None,
//                 unique: Some(ColumnUniqueOption { is_primary: true }),
//                 comment: None,
//             }]),
//             indexes: Vec::new(),
//             engine: None,
//             foreign_keys: Vec::new(),
//             comment: None,
//         };

//         let ddl = r#"CREATE TABLE "User" ("id" INT NOT NULL PRIMARY KEY);"#;
//         assert_eq!(schema.to_ddl(), ddl);

//         let actual = Schema::from_ddl(ddl).unwrap();
//         assert_schema(actual, schema);
//     }

//     #[test]
//     fn invalid_ddl() {
//         // Only Statement::CreateTable is supported
//         let invalid_ddl = r#"DROP TABLE "Users";"#;
//         let actual = Schema::from_ddl(invalid_ddl);
//         assert_eq!(actual, Err(SchemaParseError::CannotParseDDL.into()));
//     }

//     #[test]
//     fn table_with_index() {
//         let schema = Schema {
//             table_name: "User".to_owned(),
//             column_defs: Some(vec![
//                 ColumnDef {
//                     name: "id".to_owned(),
//                     data_type: DataType::Int,
//                     nullable: false,
//                     default: None,
//                     unique: None,
//                     comment: None,
//                 },
//                 ColumnDef {
//                     name: "name".to_owned(),
//                     data_type: DataType::Text,
//                     nullable: false,
//                     default: None,
//                     unique: None,
//                     comment: None,
//                 },
//             ]),
//             indexes: vec![
//                 SchemaIndex {
//                     name: "User_id".to_owned(),
//                     expr: Expr::Identifier("id".to_owned()),
//                     order: SchemaIndexOrd::Both,
//                     created: Utc::now().naive_utc(),
//                 },
//                 SchemaIndex {
//                     name: "User_name".to_owned(),
//                     expr: Expr::Identifier("name".to_owned()),
//                     order: SchemaIndexOrd::Both,
//                     created: Utc::now().naive_utc(),
//                 },
//             ],
//             engine: None,
//             foreign_keys: Vec::new(),
//             comment: None,
//         };
//         let ddl = r#"CREATE TABLE "User" ("id" INT NOT NULL, "name" TEXT NOT NULL);
// CREATE INDEX "User_id" ON "User" ("id");
// CREATE INDEX "User_name" ON "User" ("name");"#;
//         assert_eq!(schema.to_ddl(), ddl);

//         let actual = Schema::from_ddl(ddl).unwrap();
//         assert_schema(actual, schema);

//         let index_should_not_be_first = r#"CREATE INDEX "User_id" ON "User" ("id");
// CREATE TABLE "User" ("id" INT NOT NULL, "name" TEXT NOT NULL);"#;
//         let actual = Schema::from_ddl(index_should_not_be_first);
//         assert_eq!(actual, Err(SchemaParseError::CannotParseDDL.into()));
//     }

//     #[test]
//     fn non_word_identifier() {
//         let schema = Schema {
//             table_name: 1.to_string(),
//             column_defs: Some(vec![
//                 ColumnDef {
//                     name: 2.to_string(),
//                     data_type: DataType::Int,
//                     nullable: true,
//                     default: None,
//                     unique: None,
//                     comment: None,
//                 },
//                 ColumnDef {
//                     name: ";".to_owned(),
//                     data_type: DataType::Int,
//                     nullable: true,
//                     default: None,
//                     unique: None,
//                     comment: None,
//                 },
//             ]),
//             indexes: vec![SchemaIndex {
//                 name: ".".to_owned(),
//                 expr: Expr::Identifier(";".to_owned()),
//                 order: SchemaIndexOrd::Both,
//                 created: Utc::now().naive_utc(),
//             }],
//             engine: None,
//             foreign_keys: Vec::new(),
//             comment: None,
//         };
//         let ddl = r#"CREATE TABLE "1" ("2" INT NULL, ";" INT NULL);
// CREATE INDEX "." ON "1" (";");"#;
//         assert_eq!(schema.to_ddl(), ddl);

//         let actual = Schema::from_ddl(ddl).unwrap();
//         assert_schema(actual, schema);
//     }
// }
