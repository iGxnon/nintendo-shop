use crate::domain::checkout::model::ShippingDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::checkout::v1::Shipping;

pub(in crate::domain) fn execute(conn: &mut PgConnection) -> Result<Vec<Shipping>> {
    Ok(ShippingDomain::list(conn)?
        .into_iter()
        .map(|v| v.into_shipping())
        .collect())
}
