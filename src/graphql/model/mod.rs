mod cart;
mod common;
mod product;

use crate::graphql::model::cart::{Cart, MutationCart};
use crate::graphql::model::product::Product;
use crate::graphql::Resolver;
use crate::infra::error::{Code, Status};
use crate::infra::id::Id;
use crate::infra::mqsrs::*;
use async_graphql::connection::{query, Connection, Edge};
use async_graphql::*;
use volo_gen::common::v1::PaginationOption;

pub struct GraphqlQuery;
pub struct GraphqlMutation;

macro_rules! map_not_found {
    ($res:tt) => {
        match $res {
            Ok(v) => Ok(Some(v.try_into()?)),
            Err(e) => {
                if e.code() == Code::NotFound {
                    return Ok(None);
                }
                Err(Error::from(e))
            }
        }
    };
}

#[Object]
impl GraphqlQuery {
    async fn product<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Product>> {
        let id: Id<Product> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let query = resolver.create_get_product();
        let res = query.execute(id.raw()).await;
        map_not_found!(res)
    }

    async fn products<'ctx>(
        &self,
        cx: &Context<'ctx>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<i64, Product>> {
        let resolver = cx.data::<Resolver>()?;
        let queries = resolver.create_list_product();
        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let res = queries
                    .execute(PaginationOption {
                        after,
                        before,
                        first: first.map(|v| v as i32),
                        last: last.map(|v| v as i32),
                        order_by: None,
                    })
                    .await?;
                let mut conn = Connection::new(res.has_previous_page, res.has_next_page);
                conn.edges.extend(
                    res.products
                        .into_iter()
                        .map(|product| Ok(Edge::new(product.id, product.try_into()?)))
                        .collect::<Result<Vec<_>, Status>>()?,
                );
                Ok::<_, Error>(conn)
            },
        )
        .await
    }

    async fn cart<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Cart>> {
        let id: Id<Cart> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let query = resolver.create_get_cart();
        let res = query.execute(id.raw()).await;
        map_not_found!(res)
    }
}

#[Object]
impl GraphqlMutation {
    async fn create_cart<'ctx>(&self, cx: &Context<'ctx>) -> Result<MutationCart> {
        let resolver = cx.data::<Resolver>()?;
        let mutate = resolver.create_create_cart();
        let cart = mutate.execute(()).await?;
        Ok(MutationCart {
            cart: cart.try_into()?,
        })
    }

    async fn add_to_cart<'ctx>(
        &self,
        cx: &Context<'ctx>,
        cart_id: String,
        variant_id: String,
    ) -> Result<MutationCart> {
        let cart_id: Id<Cart> = cart_id.parse()?;
        let variant_id: Id<Cart> = variant_id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mutate = resolver.create_add_to_cart();
        let cart = mutate.execute((cart_id.raw(), variant_id.raw())).await?;
        Ok(MutationCart {
            cart: cart.try_into()?,
        })
    }

    async fn remove_from_cart<'ctx>(
        &self,
        cx: &Context<'ctx>,
        cart_id: String,
        entry_id: String,
    ) -> Result<MutationCart> {
        let cart_id: Id<Cart> = cart_id.parse()?;
        let entry_id: Id<Cart> = entry_id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mutate = resolver.create_remove_from_cart();
        let cart = mutate.execute((cart_id.raw(), entry_id.raw())).await?;
        Ok(MutationCart {
            cart: cart.try_into()?,
        })
    }
}
