use {
    // self::function::BreakCase,
    // super::{context::RowContext, select::select},
    crate::{
        ast::{Aggregate, Expr, Function},
        // data::{CustomFunction, Interval, Literal, Row, Value},
        // mock::MockStorage,
        result::{Error, Result}, 
        // store::GStore,
    },
    async_recursion::async_recursion,
    chrono::prelude::Utc,
    futures::{
        future::{ready, try_join_all},
        stream::{self, StreamExt, TryStreamExt},
    },
    im_rc::HashMap,
    std::{borrow::Cow, ops::ControlFlow, rc::Rc},
};



mod aggregate;
mod select;

pub use aggregate::AggregateError;
pub use select::SelectError;

pub async fn evaluate_stateless(expr: Expr) -> Result<()> {
    Ok(())
}