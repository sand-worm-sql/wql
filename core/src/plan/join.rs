use {
    super::{context::Context, evaluable::check_expr as check_evaluable, planner::Planner},
    crate::{
        ast::{
            BinaryOperator, Expr, Join, JoinConstraint, JoinExecutor, JoinOperator, Query, Select,
            SetExpr, Statement, TableWithJoins,
        },
        data::Schema,
    },
    std::{collections::HashMap, rc::Rc},
    utils::Vector,
};

pub fn plan(schema_map: &HashMap<String, Schema>, statement: Statement) -> Statement {
    let planner = JoinPlanner { schema_map };

    match statement {
        Statement::Query(query) => {
            let query = planner.query(None, query);

            Statement::Query(query)
        }
        _ => statement,
    }
}

struct JoinPlanner<'a> {
    schema_map: &'a HashMap<String, Schema>,
}

impl<'a> Planner<'a> for JoinPlanner<'a> {
    fn query(&self, outer_context: Option<Rc<Context<'a>>>, query: Query) -> Query {
        let Query {
            body,
            order_by,
            limit,
            offset,
        } = query;

        let body = match body {
            SetExpr::Select(select) => {
                let select = self.select(outer_context, *select);

                SetExpr::Select(Box::new(select))
            }
            SetExpr::Values(_) => body,
        };

        Query {
            body,
            order_by,
            limit,
            offset,
        }
    }

    fn get_schema(&self, name: &str) -> Option<&'a Schema> {
        self.schema_map.get(name)
    }
}

impl<'a> JoinPlanner<'a> {
    fn select(&self, outer_context: Option<Rc<Context<'a>>>, select: Select) -> Select {
        let Select {
            projection,
            from,
            selection,
            group_by,
            having,
        } = select;

        let (outer_context, from) = self.table_with_joins(outer_context, from);
        let selection = selection.map(|expr| self.subquery_expr(outer_context, expr));

        Select {
            projection,
            from,
            selection,
            group_by,
            having,
        }
    }

    fn table_with_joins(
        &self,
        outer_context: Option<Rc<Context<'a>>>,
        table_with_joins: TableWithJoins,
    ) -> (Option<Rc<Context<'a>>>, TableWithJoins) {
        let TableWithJoins { relation, joins } = table_with_joins;
        let init_context = self.update_context(None, &relation);
        let (context, joins) =
            joins
                .into_iter()
                .fold((init_context, Vector::new()), |(context, joins), join| {
                    let outer_context = outer_context.as_ref().map(Rc::clone);
                    let (context, join) = self.join(outer_context, context, join);
                    let joins = joins.push(join);

                    (context, joins)
                });
        let joins = joins.into();
        let context = Context::concat(context, outer_context);

        (context, TableWithJoins { relation, joins })
    }

    fn join(
        &self,
        outer_context: Option<Rc<Context<'a>>>,
        inner_context: Option<Rc<Context<'a>>>,
        join: Join,
    ) -> (Option<Rc<Context<'a>>>, Join) {
        let Join {
            relation,
            join_operator,
            join_executor,
        } = join;

        if matches!(join_executor, JoinExecutor::Hash { .. }) {
            let context = self.update_context(inner_context, &relation);
            let join = Join {
                relation,
                join_operator,
                join_executor,
            };

            return (context, join);
        }

        enum JoinOp {
            Inner,
            LeftOuter,
        }

        let (join_op, expr) = match join_operator {
            JoinOperator::Inner(JoinConstraint::On(expr)) => (JoinOp::Inner, expr),
            JoinOperator::LeftOuter(JoinConstraint::On(expr)) => (JoinOp::LeftOuter, expr),
            JoinOperator::Inner(JoinConstraint::None)
            | JoinOperator::LeftOuter(JoinConstraint::None) => {
                let context = self.update_context(inner_context, &relation);
                let join = Join {
                    relation,
                    join_operator,
                    join_executor,
                };

                return (context, join);
            }
        };

        let current_context = self.update_context(None, &relation);
        let (join_executor, expr) = self.join_expr(
            outer_context,
            inner_context.as_ref().map(Rc::clone),
            current_context,
            expr,
        );

        let join_operator = match (join_op, expr) {
            (JoinOp::Inner, Some(expr)) => JoinOperator::Inner(JoinConstraint::On(expr)),
            (JoinOp::Inner, None) => JoinOperator::Inner(JoinConstraint::None),
            (JoinOp::LeftOuter, Some(expr)) => JoinOperator::LeftOuter(JoinConstraint::On(expr)),
            (JoinOp::LeftOuter, None) => JoinOperator::LeftOuter(JoinConstraint::None),
        };

        let context = self.update_context(inner_context, &relation);
        let join = Join {
            relation,
            join_operator,
            join_executor,
        };

        (context, join)
    }

    fn join_expr(
        &self,
        outer_context: Option<Rc<Context<'a>>>,
        inner_context: Option<Rc<Context<'a>>>,
        current_context: Option<Rc<Context<'a>>>,
        expr: Expr,
    ) -> (JoinExecutor, Option<Expr>) {
        match expr {
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Eq,
                right,
            } => {
                let key_context = {
                    let current = current_context.as_ref().map(Rc::clone);
                    let outer = outer_context.as_ref().map(Rc::clone);

                    Context::concat(current, outer)
                };
                let value_context = {
                    let context = Context::concat(current_context, inner_context);

                    Context::concat(context, outer_context)
                };

                let left_as_key = check_evaluable(key_context.as_ref().map(Rc::clone), &left);
                let right_as_value = check_evaluable(value_context.as_ref().map(Rc::clone), &right);

                if left_as_key && right_as_value {
                    let join_executor = JoinExecutor::Hash {
                        key_expr: *left,
                        value_expr: *right,
                        where_clause: None,
                    };

                    return (join_executor, None);
                }

                let right_as_key = check_evaluable(key_context, &right);
                let left_as_value = left_as_key || check_evaluable(value_context, &left);

                if right_as_key && left_as_value {
                    let join_executor = JoinExecutor::Hash {
                        key_expr: *right,
                        value_expr: *left,
                        where_clause: None,
                    };

                    return (join_executor, None);
                }

                let expr = Expr::BinaryOp {
                    left,
                    op: BinaryOperator::Eq,
                    right,
                };

                (JoinExecutor::NestedLoop, Some(expr))
            }
            Expr::BinaryOp {
                left,
                op: BinaryOperator::And,
                right,
            } => {
                let (join_executor, left) = self.join_expr(
                    outer_context.as_ref().map(Rc::clone),
                    inner_context.as_ref().map(Rc::clone),
                    current_context.as_ref().map(Rc::clone),
                    *left,
                );

                if let JoinExecutor::Hash {
                    key_expr,
                    value_expr,
                    where_clause,
                } = join_executor
                {
                    let context = {
                        let current = current_context.as_ref().map(Rc::clone);
                        let outer = outer_context.as_ref().map(Rc::clone);

                        Context::concat(current, outer)
                    };

                    let expr = match left {
                        Some(left) => Expr::BinaryOp {
                            left: Box::new(left),
                            op: BinaryOperator::And,
                            right,
                        },
                        None => *right,
                    };

                    let (evaluable_expr, expr) = find_evaluable(context, expr);

                    let where_clause = match (where_clause, evaluable_expr) {
                        (Some(expr), Some(expr2)) => Some(Expr::BinaryOp {
                            left: Box::new(expr),
                            op: BinaryOperator::And,
                            right: Box::new(expr2),
                        }),
                        (Some(expr), None) | (None, Some(expr)) => Some(expr),
                        (None, None) => None,
                    };

                    let join_executor = JoinExecutor::Hash {
                        key_expr,
                        value_expr,
                        where_clause,
                    };

                    return (join_executor, expr);
                }

                let (join_executor, right) = self.join_expr(
                    outer_context.as_ref().map(Rc::clone),
                    inner_context,
                    current_context.as_ref().map(Rc::clone),
                    *right,
                );

                let expr = match (left, right) {
                    (Some(left), Some(right)) => Some(Expr::BinaryOp {
                        left: Box::new(left),
                        op: BinaryOperator::And,
                        right: Box::new(right),
                    }),
                    (expr @ Some(_), None) | (None, expr @ Some(_)) => expr,
                    // (None,None) -> unreachable
                    // To resolve this,
                    // join_expr should return an enum of
                    //   Consumed(Option<Expr>) or NotConsumed(Expr)
                    (None, None) => None,
                };

                match join_executor {
                    JoinExecutor::NestedLoop => (join_executor, expr),
                    JoinExecutor::Hash {
                        key_expr,
                        value_expr,
                        where_clause,
                    } => {
                        let context = Context::concat(current_context, outer_context);
                        let (evaluable_expr, expr) = expr
                            .map(|expr| find_evaluable(context, expr))
                            .unwrap_or((None, None));

                        let where_clause = match (where_clause, evaluable_expr) {
                            (Some(expr), Some(expr2)) => Some(Expr::BinaryOp {
                                left: Box::new(expr),
                                op: BinaryOperator::And,
                                right: Box::new(expr2),
                            }),
                            (Some(expr), None) | (None, Some(expr)) => Some(expr),
                            (None, None) => None,
                        };

                        let join_executor = JoinExecutor::Hash {
                            key_expr,
                            value_expr,
                            where_clause,
                        };

                        (join_executor, expr)
                    }
                }
            }
            Expr::Nested(expr) => {
                self.join_expr(outer_context, inner_context, current_context, *expr)
            }
            Expr::Subquery(query) => {
                let context = Context::concat(current_context, inner_context);
                let context = Context::concat(context, outer_context);

                let query = self.query(context, *query);
                let expr = Some(Expr::Subquery(Box::new(query)));

                (JoinExecutor::NestedLoop, expr)
            }
            Expr::InSubquery {
                expr,
                subquery,
                negated,
            } => {
                let context = Context::concat(current_context, inner_context);
                let context = Context::concat(context, outer_context);

                let subquery = self.query(context, *subquery);
                let expr = Some(Expr::InSubquery {
                    expr,
                    subquery: Box::new(subquery),
                    negated,
                });
                (JoinExecutor::NestedLoop, expr)
            }
            Expr::Exists { subquery, negated } => {
                let context = Context::concat(current_context, inner_context);
                let context = Context::concat(context, outer_context);

                let subquery = self.query(context, *subquery);
                let expr = Some(Expr::Exists {
                    subquery: Box::new(subquery),
                    negated,
                });
                (JoinExecutor::NestedLoop, expr)
            }
            _ => (JoinExecutor::NestedLoop, Some(expr)),
        }
    }
}

type EvaluableExpr = Option<Expr>;
type RemainderExpr = Option<Expr>;

fn find_evaluable(context: Option<Rc<Context<'_>>>, expr: Expr) -> (EvaluableExpr, RemainderExpr) {
    match expr {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            let (evaluable, remainder) = find_evaluable(context.as_ref().map(Rc::clone), *left);
            let (evaluable2, remainder2) = find_evaluable(context, *right);

            let merge = |expr, expr2| match (expr, expr2) {
                (Some(expr), Some(expr2)) => Some(Expr::BinaryOp {
                    left: Box::new(expr),
                    op: BinaryOperator::And,
                    right: Box::new(expr2),
                }),
                (Some(expr), None) | (None, Some(expr)) => Some(expr),
                (None, None) => None,
            };

            let evaluable_expr = merge(evaluable, evaluable2);
            let remainder_expr = merge(remainder, remainder2);

            (evaluable_expr, remainder_expr)
        }
        _ if check_evaluable(context, &expr) => (Some(expr), None),
        _ => (None, Some(expr)),
    }
}

#[cfg(test)]
mod tests {
    use {
        super::plan,
        crate::{
            ast::{DateTimeField, Statement},
            ast_builder::{chain, col, exists, num, subquery, Build, QueryNode},
            mock::{run, MockStorage},
            parse_sql::parse,
            plan::fetch_schema_map,
            translate::translate,
        },
        futures::executor::block_on,
    };

    fn plan_join(storage: &MockStorage, sql: &str) -> Statement {
        let parsed = parse(sql).expect(sql).into_iter().next().unwrap();
        let statement = translate(&parsed).unwrap();
        let schema_map = block_on(fetch_schema_map(storage, &statement)).unwrap();

        plan(&schema_map, statement)
    }

    macro_rules! test {
        ($actual: expr, $expected: expr, $name: literal) => {
            let expected = $expected.build().unwrap();

            assert_eq!($actual, expected, $name);
        };
    }

    #[test]
    fn basic() {
        let storage = run("
            CREATE TABLE Player (
                id INTEGER,
                name TEXT
            );
            CREATE TABLE PlayerItem (
                user_id INTEGER,
                item_id INTEGER,
                amount INTEGER
            );
        ");

        let sql = "SELECT * FROM base.Player;";
        let actual = plan_join(&storage, sql);
        let expected = chain("base").select("Player");
        test!(actual, expected, "basic select:\n{sql}");

        let sql = "
            SELECT *
            FROM base.Player
            JOIN base.PlayerItem ON PlayerItem.user_id != Player.id
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("base")
            .select("Player")
            .join(Some("base"), "PlayerItem")
            .on("PlayerItem.user_id != Player.id");
        test!(actual, expected, "basic nested loop join:\n{sql}");

        let sql = "
            SELECT *
            FROM base.Player
            LEFT JOIN base.PlayerItem ON PlayerItem.amount > 2
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("base")
            .select("Player")
            .left_join(Some("base"), "PlayerItem")
            .on("PlayerItem.amount > 2");
        test!(actual, expected, "basic nested loop join 2:\n{sql}");

        let sql = "
            SELECT *
            FROM base.Player
            JOIN Empty u2
            LEFT JOIN Player u3;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("base")
            .select("Player")
            .join_as(None, "Empty", "u2")
            .left_join_as(None, "Player", "u3");
        test!(actual, expected, "self multiple joins:\n{sql}");

        let sql = "
            SELECT *
            FROM base.Player
            JOIN PlayerItem ON PlayerItem.user_id = Player.id
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("base")
            .select("Player")
            .join(None, "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id");
        test!(actual, expected, "basic hash join query:\n{sql}");

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON PlayerItem.user_id = Player.id
        ";
        let actual = plan_join(&storage, sql);
        let actual = {
            let schema_map = block_on(fetch_schema_map(&storage, &actual)).unwrap();

            plan(&schema_map, actual)
        };
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id");
        test!(
            actual,
            expected,
            "redundant plan does not change the plan result:\n{sql}"
        );

        let sql = "
            SELECT * FROM sui.Player
            JOIN PlayerItem ON (SELECT * FROM Player u2)
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(None, "PlayerItem")
            .on("(SELECT * FROM Player u2)");
        test!(actual, expected, "subquery in join_constraint:\n{sql}");
    }

    #[test]
    fn hash_join() {
        let storage = run("
            CREATE TABLE Player (
                id INTEGER,
                name TEXT
            );
            CREATE TABLE Item (
                id INTEGER,
                name TEXT
            );
            CREATE TABLE PlayerItem (
                user_id INTEGER,
                item_id INTEGER,
                amount INTEGER
            );
        ");

        let sql = "
            SELECT *
            FROM sui.Player
            LEFT JOIN sui.PlayerItem ON
                PlayerItem.amount > 10 AND
                PlayerItem.user_id = Player.id
            WHERE True;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .left_join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .hash_filter("PlayerItem.amount > 10")
            .filter(true);
        test!(actual, expected, "where_clause AND hash_join expr:\n{sql}");

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN PlayerItem ON
                (PlayerItem.user_id = Player.id) AND
                Player.name = 'abcd' AND
                Player.name != 'barcode'
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(None, "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .on("Player.name = 'abcd' AND Player.name != 'barcode'");
        test!(
            actual,
            expected,
            "nested expr & remaining join constraint:\n{sql}"
        );

        let sql = "
            SELECT *
            FROM sui.Player
            LEFT JOIN sui.PlayerItem ON
                PlayerItem.amount > 10 AND
                PlayerItem.amount * 3 <= 2 AND
                PlayerItem.user_id = Player.id
            WHERE True;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .left_join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .hash_filter("PlayerItem.amount > 10 AND PlayerItem.amount * 3 <= 2")
            .filter(true);
        test!(actual, expected, "complex where_clause:\n{sql}");

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON
                Player.id = PlayerItem.user_id AND
                PlayerItem.amount > 10
            WHERE True;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .hash_filter("PlayerItem.amount > 10")
            .filter(true);
        test!(actual, expected, "hash_join expr AND where_clause:\n{sql}");

        let sql = "
            SELECT *
            FROM sui.Player u1
            LEFT OUTER JOIN sui.Player u2
            WHERE u2.id = (
                SELECT u3.id
                FROM Player u3
                JOIN Player u4 ON
                    u4.id = u3.id AND
                    u4.id = u1.id
            );
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .alias_as("Player", "u1")
            .select()
            .left_join_as(Some("sui"), "Player", "u2")
            .filter(
                col("u2.id").eq(subquery(
                    chain("sui")
                        .alias_as("Player", "u3")
                        .select()
                        .join_as(Some("sui"), "Player", "u4")
                        .hash_executor("u4.id", "u3.id")
                        .hash_filter("u4.id = u1.id")
                        .project("u3.id"),
                )),
            );
        test!(actual, expected, "hash join in subquery:\n{sql}");

        let sql = "
            SELECT * FROM sui.Player u1
            WHERE u1.id = (
                SELECT * FROM sui.Player u2
                WHERE u2.id = (
                    SELECT * FROM sui.Player u3
                    JOIN sui.Player u4 ON
                        u4.id = u3.id + u1.id
                )
            );
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui").alias_as("Player", "u1").select().filter(
            col("u1.id").eq(subquery(
                chain("sui").alias_as("Player", "u2").select().filter(
                    col("u2.id").eq(subquery(
                        chain("sui")
                            .alias_as("Player", "u3")
                            .select()
                            .join_as(Some("sui"), "Player", "u4")
                            .hash_executor(col("u4.id"), col("u3.id").add("u1.id")),
                    )),
                ),
            )),
        );
        test!(actual, expected, "hash join in nested subquery:\n{sql}");

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON
                Player.id = PlayerItem.user_id AND
                Player.id > 10 AND
                PlayerItem.item_id IS NOT NULL AND
                PlayerItem.amount > 10
            WHERE True;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .hash_filter("PlayerItem.item_id IS NOT NULL")
            .hash_filter("PlayerItem.amount > 10")
            .on("Player.id > 10")
            .filter(true);
        test!(
            actual,
            expected,
            "hash join with join_constraint AND where_clause:\n{sql}"
        );

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON
                Player.id > Player.id + PlayerItem.user_id AND
                Player.id = PlayerItem.user_id AND
                PlayerItem.item_id IS NOT NULL AND
                PlayerItem.amount > 10
            WHERE True;
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .hash_executor("PlayerItem.user_id", "Player.id")
            .hash_filter("PlayerItem.item_id IS NOT NULL")
            .hash_filter("PlayerItem.amount > 10")
            .on("Player.id > Player.id + PlayerItem.user_id")
            .filter(true);
        test!(
            actual,
            expected,
            "hash join with join_constraint AND where_clause 2:\n{sql}"
        );

        let sql = "
            SELECT * 
            FROM sui.Player
            JOIN sui.PlayerItem ON 
                (SELECT * FROM sui.Player JOIN sui.PlayerItem ON Player.id = PlayerItem.user_id)
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .on(subquery(
                chain("Player")
                    .select("Player")
                    .join(Some("sui"), "PlayerItem")
                    .hash_executor("Player.id", "PlayerItem.user_id"),
            ));
        test!(
            actual,
            expected,
            "hash join with join_constraint subquery:\n{sql}"
        );

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON
                1 IN (SELECT * FROM sui.PlayerItem JOIN sui.Player ON PlayerItem.user_id = Player.id)
            WHERE True    
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .on(num(1).in_list(
                chain("sui")
                    .select("PlayerItem")
                    .join(Some("sui"), "Player")
                    .hash_executor("PlayerItem.user_id", "Player.id"),
            ))
            .filter(true);
        test!(
            actual,
            expected,
            "hash join with join constraint in subquery"
        );

        let sql = "
            SELECT *
            FROM sui.Player
            JOIN sui.PlayerItem ON
                EXISTS (SELECT * FROM sui.PlayerItem JOIN sui.Player ON PlayerItem.user_id = Player.id WHERE Player.id > 3)
            WHERE True    
        ";
        let actual = plan_join(&storage, sql);
        let expected = chain("sui")
            .select("Player")
            .join(Some("sui"), "PlayerItem")
            .on(exists(
                chain("sui")
                    .select("PlayerItem")
                    .join(Some("sui"), "Player")
                    .hash_executor("PlayerItem.user_id", "Player.id")
                    .filter("Player.id > 3"),
            ))
            .filter(true);
        test!(
            actual,
            expected,
            "hash join with join constraint in subquery"
        );
    }

    #[test]
    fn hash_join_in_subquery() {
        let storage = run("
            CREATE TABLE Player (
                id INTEGER,
                name TEXT
            );
            CREATE TABLE Flag (
                id INTEGER,
                user_id INTEGER,
                name TEXT
            );
        ");

        let subquery_sql = "
            SELECT u.id
            FROM sui.Player u
            JOIN sui.Flag f ON f.user_id = u.id
        ";
        let subquery_node = || -> QueryNode {
            chain("sui")
                .alias_as("Player", "u")
                .select()
                .join_as(Some("sui"), "Flag", "f")
                .hash_executor("f.user_id", "u.id")
                .project("u.id")
                .into()
        };

        let sql = format!("SELECT * FROM sui.Player WHERE id = ({subquery_sql})");
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui")
            .select("Player")
            .filter(col("id").eq(subquery_node()));
        test!(actual, expected, "binary operator:\n{sql}");

        let sql = format!("SELECT * FROM sui.Player WHERE -({subquery_sql}) IN ({subquery_sql})");
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui")
            .select("Player")
            .filter(subquery(subquery_node()).minus().in_list(subquery_node()));
        test!(actual, expected, "unary operator and in subquery:\n{sql}");

        let sql = format!(
            "
            SELECT * FROM sui.Player
            WHERE
                CAST(({subquery_sql}) AS INTEGER) IN (1, 2, 3)
        "
        );
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui")
            .select("Player")
            .filter(subquery(subquery_node()).cast("INTEGER").in_list("1, 2, 3"));
        test!(actual, expected, "cast and in list:\n{sql}");

        let sql = format!(
            "
            SELECT * FROM Player
            WHERE 
                ({subquery_sql}) IS NULL
                OR
                ({subquery_sql}) IS NOT NULL
        "
        );
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui").select("Player").filter(
            subquery(subquery_node())
                .is_null()
                .or(subquery(subquery_node()).is_not_null()),
        );
        test!(actual, expected, "is null and is not null:\n{sql}");

        let sql = format!("SELECT * FROM sui.Player WHERE EXISTS({subquery_sql})");
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui")
            .select("Player")
            .filter(exists(subquery_node()));
        test!(actual, expected, "exists:\n{sql}");

        let sql = format!(
            "
            SELECT * FROM sui.Player
            WHERE ({subquery_sql}) BETWEEN ({subquery_sql}) AND 100;
        "
        );
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui")
            .select("Player")
            .filter(subquery(subquery_node()).between(subquery_node(), num(100)));
        test!(actual, expected, "between:\n{sql}");

        let sql = format!(
            "
            SELECT * FROM sui.Player
            WHERE EXTRACT(HOUR FROM (({subquery_sql}))) IS NULL
        "
        );
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui").select("Player").filter(
            subquery(subquery_node())
                .nested()
                .extract(DateTimeField::Hour)
                .is_null(),
        );
        test!(actual, expected, "extract and nested:\n{sql}");

        let sql = format!(
            "
            SELECT * FROM sui.Player
            WHERE
                CASE ({subquery_sql})
                    WHEN 10 THEN True
                    WHEN 20 THEN ({subquery_sql}) IS NULL
                    ELSE col3
                END
        "
        );
        let actual = plan_join(&storage, &sql);
        let expected = chain("sui").select("Player").filter(
            subquery(subquery_node())
                .case()
                .when_then(10, true)
                .when_then(20, subquery(subquery_node()).is_null())
                .or_else("col3"),
        );
        test!(actual, expected, "case expr:\n{sql}");
    }
}
