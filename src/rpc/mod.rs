pub mod product;

use crate::infra::resolver::*;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::PooledConnection;

pub struct Config {}

pub struct Resolver {
    pub redis: Register<PooledConnection<redis::Client>>,
    pub pgsql: Register<PooledConnection<ConnectionManager<PgConnection>>>,
}

impl BaseResolver for Resolver {
    const TARGET: Target = Target::THRIFT;
}

impl NamedResolver for Resolver {
    const SID: &'static str = "product-thrift";
}
