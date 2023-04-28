use crate::domain::cart::model::CartDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::cart::v1::Cart;

pub(in crate::domain) fn execute(id: i64, conn: &mut PgConnection) -> Result<Cart> {
    CartDomain::query(id, conn).map(|v| v.into_cart())
}
