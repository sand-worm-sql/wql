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
    use crate::ast_builder::{col, exists, not_exists, chain, test, test_expr, Build};

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
            .select("checkouts")
            .filter(not_exists(table("BAR").select().filter("id IS NOT NULL")))
            .build();
        let expected =
            "SELECT * FROM FOO WHERE NOT EXISTS (SELECT * FROM BAR WHERE id IS NOT NULL)";
        test(actual, expected);

        let actual = exists(table("FOO").select().filter(col("id").gt(2)));
        let expected = "EXISTS (SELECT * FROM FOO WHERE id > 2)";
        test_expr(actual, expected);

        let actual = not_exists(table("FOO").select().filter(col("id").gt(2)));
        let expected = "NOT EXISTS (SELECT * FROM FOO WHERE id > 2)";
        test_expr(actual, expected);

        let actual = exists("SELECT * FROM ");
        let expected = "EXISTS (SELECT * FROM FOO)";
        test_expr(actual, expected);

        let actual = not_exists("SELECT * FROM sui.transations");
        let expected = "NOT EXISTS (SELECT * FROM sui.transations)";
        test_expr(actual, expected);
    }
}
