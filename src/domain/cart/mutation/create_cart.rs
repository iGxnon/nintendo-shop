use crate::{
    graphql::Resolver,
    infra::{mqsrs::Mutation, resolver::BaseResolver},
    schema::t_carts::dsl::*,
};
use anyhow::Result;
use diesel::{data_types::PgTimestamp, PgConnection, RunQueryDsl};
use std::ops::DerefMut;
use volo_gen::cart::v1::{Cart, CreateCartReq, CreateCartRes};

fn execute(_: CreateCartReq, conn: &mut PgConnection) -> Result<CreateCartRes> {
    let data = diesel::insert_into(t_carts)
        .default_values()
        .get_result::<(i64, PgTimestamp, PgTimestamp)>(conn)?;
    Ok(CreateCartRes {
        cart: Cart {
            id: data.0,
            entries: vec![],
        },
    })
}

impl Resolver {
    pub fn create_cart(&self) -> impl Mutation<CreateCartReq, Result<CreateCartRes>> + '_ {
        move |_: CreateCartReq| async move {
            execute(
                CreateCartReq {},
                self.resolve(&self.pgsql).get()?.deref_mut(),
            )
        }
    }
}
