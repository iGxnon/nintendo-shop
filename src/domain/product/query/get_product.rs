use crate::domain::product::model::ProductDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::product::v1::Product;

pub(in crate::domain) fn execute(id: i64, conn: &mut PgConnection) -> Result<Product> {
    ProductDomain::query(id, conn).map(|v| v.into_product())
}
