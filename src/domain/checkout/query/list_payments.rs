use crate::domain::checkout::model::PaymentDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::checkout::v1::Payment;

pub(in crate::domain) fn execute(conn: &mut PgConnection) -> Result<Vec<Payment>> {
    Ok(PaymentDomain::list(conn)?
        .into_iter()
        .map(|v| v.into_payment())
        .collect())
}
