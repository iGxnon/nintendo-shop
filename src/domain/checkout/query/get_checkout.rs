use crate::domain::checkout::model::CheckoutDomain;
use crate::infra::error::Result;
use diesel::{Connection, PgConnection};
use volo_gen::checkout::v1::Checkout;

pub(in crate::domain) fn execute(id: i64, conn: &mut PgConnection) -> Result<Checkout> {
    conn.transaction(|conn| Ok(CheckoutDomain::query(id, conn)?.into_checkout()))
}
