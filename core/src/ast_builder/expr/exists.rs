use {super::ExprNode, crate::ast_builder::QueryNode};

pub fn exists<'a, T: Into<QueryNode<'a>>>(query: T) -> ExprNode<'a> {
    ExprNode::Exists {
        subquery: Box::new(query.into()),
        negated: false,
    }
}

pub fn not_exists<'a, T: Into<QueryNode<'a>>>(query: T) -> ExprNode<'a> {
    ExprNode::Exists {
        subquery: Box::new(query.into()),
        negated: true,
    }
}

#[cfg(test)]
mod test {
    use crate::ast_builder::{chain, col, exists, not_exists, test, test_expr, Build};

    #[test]
    fn exist() {
        let actual = chain("sui")
            .select("transations")
            .filter(exists(
                chain("sui")
                    .select("coins")
                    .filter("address IS NOT NULL")
                    .group_by("address"),
            ))
            .build();
        let expected =
            "SELECT * FROM sui.transations WHERE EXISTS (SELECT * FROM sui.coins WHERE address IS NOT NULL GROUP BY address)";
        test(actual, expected);

        let actual = chain("sui")
            .select("checkpoints")
            .filter(not_exists(chain("sui").select("checkpoints").filter("tnx IS NOT NULL")))
            .build();
        let expected =
            "SELECT * FROM sui.checkpoints WHERE NOT EXISTS (SELECT * FROM sui.checkpoints WHERE tnx IS NOT NULL)";
        test(actual, expected);

        let actual = exists(chain("sui").select("checkpoints").filter(col("tnx").gt(2)));
        let expected = "EXISTS (SELECT * FROM sui.checkpoints WHERE tnx > 4)";
        test_expr(actual, expected);

        let actual = not_exists(chain("sui").select("transations").filter(col("tnx").gt(2)));
        let expected = "NOT EXISTS (SELECT * FROM sui.transations WHERE tnx > 2)";
        test_expr(actual, expected);

        let actual = exists("SELECT * FROM sui.transations");
        let expected = "EXISTS (SELECT * FROM FOO)";
        test_expr(actual, expected);

        let actual = not_exists("SELECT * FROM sui.transations");
        let expected = "NOT EXISTS (SELECT * FROM sui.transations)";
        test_expr(actual, expected);
    }
}
