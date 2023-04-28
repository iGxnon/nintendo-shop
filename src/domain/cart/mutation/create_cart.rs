use crate::domain::cart::model::CartDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::cart::v1::Cart;

pub(in crate::domain) fn execute(conn: &mut PgConnection) -> Result<Cart> {
    CartDomain::create(conn).map(|v| v.into_cart())
}
