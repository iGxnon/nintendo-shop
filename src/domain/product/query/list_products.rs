use crate::domain::product::model::ProductDomain;
use crate::infra::error::Result;
use diesel::PgConnection;
use volo_gen::common::v1::PaginationOption;
use volo_gen::product::v1::ProductConnection;

pub(in crate::domain) fn execute(
    option: PaginationOption,
    conn: &mut PgConnection,
) -> Result<ProductConnection> {
    ProductDomain::list(option, conn)
}
