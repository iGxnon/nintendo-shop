pub mod model;
pub mod mutation;
pub mod query;

use crate::infra::error::Result;
use crate::infra::mqsrs::Mutation;
use crate::infra::mqsrs::Query;
use std::ops::DerefMut;
use volo_gen::checkout::v1::Checkout;

pub mod graphql {
    use super::*;
    use crate::graphql::Resolver;
    use volo_gen::checkout::v1::PutCheckout;

    impl Resolver {
        pub fn create_get_checkout(&self) -> impl Query<i64, Result<Checkout>> + '_ {
            use crate::domain::checkout::query::get_checkout::execute;

            move |id: i64| async move { execute(id, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_get_checkout_by_cart_id(&self) -> impl Query<i64, Result<Checkout>> + '_ {
            use crate::domain::checkout::query::get_checkout_by_cart_id::execute;

            move |cid: i64| async move { execute(cid, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_create_checkout(&self) -> impl Mutation<i64, Result<Checkout>> + '_ {
            use crate::domain::checkout::mutation::create_checkout::execute;

            move |cid: i64| async move { execute(cid, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_submit_information(
            &self,
        ) -> impl Mutation<(i64, PutCheckout), Result<Checkout>> + '_ {
            use crate::domain::checkout::mutation::submit_information::execute;

            move |(id, put): (i64, PutCheckout)| async move {
                execute(id, put, self.pg_conn()?.deref_mut())
            }
        }
    }
}
