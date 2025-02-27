use futures_core::future::BoxFuture;

use crate::Conn;

pub use self::{exec::*, next_set::*, ping::*, prepare::*, query::*, reset::*};

mod exec;
mod load_data;
mod next_set;
mod ping;
mod prepare;
mod query;
mod reset;

mod helpers;

/// Connection will be broken if this operation isn't finished.
pub trait Routine<T> {
    fn call<'a>(&'a mut self, conn: &'a mut Conn) -> BoxFuture<'a, crate::Result<T>>;
}
