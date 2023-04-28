pub mod model;
pub mod mutation;
pub mod query;

use crate::infra::error::Result;
use crate::infra::mqsrs::Mutation;
use crate::infra::mqsrs::Query;
use std::ops::DerefMut;
use volo_gen::cart::v1::Cart;

pub mod graphql {
    use super::*;
    use crate::graphql::Resolver;

    impl Resolver {
        pub fn create_get_cart(&self) -> impl Query<i64, Result<Cart>> + '_ {
            use crate::domain::cart::query::get_cart::execute;

            move |req: i64| async move { execute(req, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_create_cart(&self) -> impl Mutation<(), Result<Cart>> + '_ {
            use crate::domain::cart::mutation::create_cart::execute;

            move |_: ()| async move { execute(self.pg_conn()?.deref_mut()) }
        }

        pub fn create_add_to_cart(&self) -> impl Mutation<(i64, i64), Result<Cart>> + '_ {
            use crate::domain::cart::mutation::add_to_cart::execute;

            move |req: (i64, i64)| async move { execute(req.0, req.1, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_remove_from_cart(&self) -> impl Mutation<(i64, i64), Result<Cart>> + '_ {
            use crate::domain::cart::mutation::remove_from_cart::execute;

            move |req: (i64, i64)| async move { execute(req.0, req.1, self.pg_conn()?.deref_mut()) }
        }
    }
}
