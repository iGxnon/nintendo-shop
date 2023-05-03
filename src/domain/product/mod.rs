pub mod model;
pub mod mutation;
pub mod query;

use crate::infra::error::*;
use crate::infra::mqsrs::Query;
use std::ops::DerefMut;
use volo_gen::common::v1::PaginationOption;
use volo_gen::product::v1::Product;
use volo_gen::product::v1::ProductConnection;

pub mod graphql {
    use super::*;
    use crate::graphql::Resolver;

    impl Resolver {
        pub fn create_get_product(&self) -> impl Query<i64, Result<Product>> + '_ {
            use crate::domain::product::query::get_product::execute;

            move |req: i64| async move { execute(req, self.pg_conn()?.deref_mut()) }
        }

        pub fn create_list_product(
            &self,
        ) -> impl Query<PaginationOption, Result<ProductConnection>> + '_ {
            use crate::domain::product::query::list_products::execute;

            move |req: PaginationOption| async move { execute(req, self.pg_conn()?.deref_mut()) }
        }
    }
}
