pub mod product;

use crate::infra::resolver::{Register, Target};
use crate::infra::*;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::PooledConnection;
use std::path::Path;

pub struct Config {}

pub struct Resolver {
    pub redis: Register<PooledConnection<redis::Client>>,
    pub pgsql: Register<PooledConnection<ConnectionManager<PgConnection>>>,
}

impl resolver::BaseResolver for Resolver {
    const TARGET: Target = Target::THRIFT;
}

impl resolver::NamedResolver for Resolver {
    const UUID: &'static str = "product-thrift";
}

impl Resolver {
    fn new(conf: impl AsRef<Path>) -> Self {
        todo!()
    }
}
