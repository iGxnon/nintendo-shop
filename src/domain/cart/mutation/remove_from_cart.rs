use crate::domain::cart::model::CartDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::cart::v1::Cart;

pub(in crate::domain) fn execute(
    cart_id: i64,
    entry_id: i64,
    conn: &mut PgConnection,
) -> Result<Cart> {
    let mut cart = CartDomain::query(cart_id, conn)?;
    cart.remove_item(entry_id, conn)?;
    Ok(cart.into_cart())
}