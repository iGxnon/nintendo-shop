use crate::domain::checkout::model::CheckoutDomain;
use crate::infra::error::Result;
use diesel::{Connection, PgConnection};
use volo_gen::checkout::v1::{Checkout, PutCheckout};

pub(in crate::domain) fn execute(
    id: i64,
    put: PutCheckout,
    conn: &mut PgConnection,
) -> Result<Checkout> {
    conn.transaction(|conn| {
        let mut checkout = CheckoutDomain::query(id, conn)?;
        checkout.submit_information(put, conn)?;
        Ok(checkout.into_checkout())
    })
}
