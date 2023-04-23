// modelv1 mod is different from modelv2, it does not use idl modelv1 to
// communicate with domain layer. It is just a graphql modelv1 wrapper
// upon a dao modelv1

mod cart;
mod common;
mod product;

use crate::domain::cart::model::{NewCartEntry, QueryCart, QueryCartEntry};
use crate::domain::product::model::{QueryProduct, QueryProductImage, QueryProductVariant};
use crate::graphql::modelv1::cart::{Cart, CartEntry, MutationCart};
use crate::graphql::modelv1::common::{CurrencyCode, Image, Money};
use crate::graphql::modelv1::product::{Product, ProductVariant};
use crate::graphql::Resolver;
use crate::infra::id::Id;
use crate::infra::resolver::BaseResolver;
use crate::schema::{t_cart_entries, t_carts, t_product_variants, t_products};
use anyhow::anyhow;
use async_graphql::connection::{query, Connection as GraphqlConnection, Edge};
use async_graphql::*;
use bigdecimal::BigDecimal;
use diesel::data_types::PgTime;
use diesel::prelude::*;
use diesel::sql_types::{Time, Timestamp};
use diesel::Connection;
use std::collections::HashMap;
use std::ops::{Add, DerefMut, Div};

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    async fn product<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Product>> {
        let id: Id<Product> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;
        get_product(id, conn.deref_mut())
    }

    async fn must_product<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Product>> {
        let id: Id<Product> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;
        conn.deref_mut().transaction(|conn| get_product(id, conn))
    }

    async fn products<'ctx>(
        &self,
        cx: &Context<'ctx>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<GraphqlConnection<i64, Product>> {
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;
        query_products(after, before, first, last, conn.deref_mut()).await
    }

    async fn must_products<'ctx>(
        &self,
        cx: &Context<'ctx>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<GraphqlConnection<i64, Product>> {
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;
        must_query_products(after, before, first, last, conn.deref_mut()).await
    }

    async fn cart<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Cart>> {
        let id: Id<Cart> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;

        get_cart(id, conn.deref_mut())
    }

    async fn must_cart<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Cart>> {
        let id: Id<Cart> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;
        conn.deref_mut().transaction(|conn| get_cart(id, conn))
    }
}

#[inline]
fn merge_products(
    raw: Vec<QueryProduct>,
    images: Vec<Vec<QueryProductImage>>,
    variants: Vec<Vec<QueryProductVariant>>,
) -> Vec<Product> {
    raw.into_iter()
        .zip(images)
        .zip(variants)
        .map(|((product, images), variants)| {
            let code = match &*product.currency_code.to_uppercase() {
                "USD" => CurrencyCode::USD,
                "CNY" => CurrencyCode::CNY,
                _ => panic!("unexpect currency_code"),
            };
            Product {
                id: product.id.into(),
                title: product.title,
                sub_title: product.sub_title,
                description: product.description,
                images: images
                    .into_iter()
                    .map(|v| Image {
                        url: v.url,
                        alt_text: v.alt_text,
                        order_idx: v.order_idx,
                    })
                    .collect(),
                variants: variants
                    .into_iter()
                    .map(|v| ProductVariant {
                        id: v.id.into(),
                        price: Money {
                            amount: BigDecimal::from(v.price.0).div(100),
                            currency_code: code,
                        },
                        title: v.title,
                        inventory_count: v.inventory_count,
                        order_idx: v.order_idx,
                    })
                    .collect(),
            }
        })
        .collect()
}

#[inline]
fn query_products_fn(
    after: Option<i64>,
    before: Option<i64>,
    first: Option<usize>,
    last: Option<usize>,
    conn: &mut PgConnection,
) -> Result<GraphqlConnection<i64, Product>> {
    let mut start = 0;
    let total = t_products::table.count().get_result::<i64>(conn)?;
    let mut end = total;

    if let Some(after) = after {
        if after >= total {
            return Ok(GraphqlConnection::new(false, false));
        }
        start = after + 1;
    };

    if let Some(before) = before {
        if before == 0 {
            return Ok(GraphqlConnection::new(false, false));
        }
        end = before;
    };

    let products = t_products::table
        .filter(t_products::id.between(start, end))
        .select(QueryProduct::as_select())
        .load(conn)?;
    let images = QueryProductImage::belonging_to(&products)
        .select(QueryProductImage::as_select())
        .load(conn)?
        .grouped_by(&products);
    let variants = QueryProductVariant::belonging_to(&products)
        .select(QueryProductVariant::as_select())
        .load(conn)?
        .grouped_by(&products);
    let mut products = merge_products(products, images, variants);

    if let Some(first) = first {
        products.truncate(first.min(products.len()));
        end -= first.min(products.len()) as i64;
    } else if let Some(last) = last {
        products = products
            .drain(products.len() - last.min(products.len())..)
            .collect();
        start = end - last.min(products.len()) as i64;
    }

    let mut connection = GraphqlConnection::new(start > 0, end < products.len() as i64);
    connection.edges.extend(
        products
            .into_iter()
            .enumerate()
            .map(|(idx, item)| Edge::new(start + (idx as i64), item)),
    );
    Ok::<_, Error>(connection)
}

#[inline]
async fn query_products(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    conn: &mut PgConnection,
) -> Result<GraphqlConnection<i64, Product>> {
    query(
        after,
        before,
        first,
        last,
        |after, before, first, last| async move {
            query_products_fn(after, before, first, last, conn)
        },
    )
    .await
}

#[inline]
async fn must_query_products(
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
    conn: &mut PgConnection,
) -> Result<GraphqlConnection<i64, Product>> {
    query(
        after,
        before,
        first,
        last,
        |after, before, first, last| async move {
            conn.transaction(|conn| query_products_fn(after, before, first, last, conn))
        },
    )
    .await
}

#[inline]
fn get_product(id: Id<Product>, conn: &mut PgConnection) -> Result<Option<Product>> {
    let product = match t_products::table
        .find(id.raw())
        .select(QueryProduct::as_select())
        .get_result(conn)
    {
        Ok(product) => product,
        Err(diesel::NotFound) => return Ok(None),
        _ => return Err(Error::from(anyhow!("Internal error."))),
    };

    let currency_code: CurrencyCode = product.currency_code.as_str().parse()?;

    let images = QueryProductImage::belonging_to(&product)
        .select(QueryProductImage::as_select())
        .load(conn)?;
    let variants = QueryProductVariant::belonging_to(&product)
        .select(QueryProductVariant::as_select())
        .load(conn)?;

    Ok(Some(Product {
        id,
        title: product.title,
        sub_title: product.sub_title,
        description: product.description,
        images: images
            .into_iter()
            .map(|v| Image {
                url: v.url,
                alt_text: v.alt_text,
                order_idx: v.order_idx,
            })
            .collect(),
        variants: variants
            .into_iter()
            .map(|v| ProductVariant {
                id: v.id.into(),
                price: Money {
                    amount: BigDecimal::from(v.price.0).div(100),
                    currency_code,
                },
                title: v.title,
                inventory_count: v.inventory_count,
                order_idx: v.order_idx,
            })
            .collect(),
    }))
}

#[inline]
fn get_cart(id: Id<Cart>, conn: &mut PgConnection) -> Result<Option<Cart>> {
    let cart = match t_carts::table
        .find(id.raw())
        .select(QueryCart::as_select())
        .get_result(conn)
    {
        Ok(cart) => cart,
        Err(diesel::NotFound) => return Ok(None),
        _ => return Err(Error::from(anyhow!("Internal error."))),
    };
    let entries: Vec<QueryCartEntry> = t_cart_entries::table
        .select(QueryCartEntry::as_select())
        .filter(t_cart_entries::cid.eq(cart.id))
        .load(conn)?;
    let pids: Vec<i64> = entries.iter().map(|v| v.pid).collect();
    let products: Vec<QueryProduct> = t_products::table
        .filter(t_products::id.eq_any(pids))
        .select(QueryProduct::as_select())
        .load(conn)?;
    let images = QueryProductImage::belonging_to(&products)
        .select(QueryProductImage::as_select())
        .load::<QueryProductImage>(conn)?
        .grouped_by(&products);
    let variants = QueryProductVariant::belonging_to(&products)
        .select(QueryProductVariant::as_select())
        .load::<QueryProductVariant>(conn)?
        .grouped_by(&products);
    let pids = products.iter().map(|v| v.id).collect::<Vec<_>>();
    let products = merge_products(products, images, variants)
        .into_iter()
        .zip(pids)
        .map(|(v, k)| (k, v))
        .collect::<HashMap<_, _>>();
    let entries = entries
        .into_iter()
        .map(|v| CartEntry {
            id: v.id.into(),
            quantity: v.quantity,
            product: products[&v.pid].clone(),
            variant_at: v.variant,
        })
        .collect::<Vec<_>>();
    Ok(Some(Cart { id, entries }))
}

#[Object]
impl Mutation {
    async fn create_cart<'ctx>(&self, cx: &Context<'ctx>) -> Result<MutationCart> {
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;

        let id = diesel::insert_into(t_carts::table)
            .default_values()
            .returning(t_carts::id)
            .get_result::<i64>(conn.deref_mut())?;

        return Ok(MutationCart {
            cart: Cart {
                id: id.into(),
                entries: vec![],
            },
        });
    }

    async fn add_to_cart<'ctx>(
        &self,
        cx: &Context<'ctx>,
        cart_id: String,
        variant_id: String,
    ) -> Result<MutationCart> {
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;

        let id: Id<ProductVariant> = variant_id.parse()?;
        let cid: Id<Cart> = cart_id.parse()?;

        conn.deref_mut().transaction(|conn| {
            let cart = t_carts::table
                .find(cid.raw())
                .select(QueryCart::as_select())
                .get_result(conn)?;

            let variant = t_product_variants::table
                .find(id.raw())
                .select(QueryProductVariant::as_select())
                .get_result(conn)?;

            let res = t_cart_entries::table
                .filter(
                    t_cart_entries::cid
                        .eq(cart.id)
                        .and(t_cart_entries::pid.eq(variant.pid))
                        .and(t_cart_entries::variant.eq(variant.order_idx)),
                )
                .select((t_cart_entries::id, t_cart_entries::quantity))
                .get_result::<(i64, i32)>(conn);

            if let Ok((eid, quantity)) = res {
                diesel::update(t_cart_entries::table)
                    .filter(t_cart_entries::id.eq(eid))
                    .set(t_cart_entries::quantity.eq(quantity + 1))
                    .execute(conn)?;
                return get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() });
            };

            diesel::insert_into(t_cart_entries::table)
                .values(&NewCartEntry {
                    cid: cart.id,
                    pid: variant.pid,
                    quantity: 1,
                    variant: variant.order_idx,
                })
                .execute(conn)?;

            return get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() });
        })
    }

    async fn remove_from_cart<'ctx>(
        &self,
        cx: &Context<'ctx>,
        cart_id: String,
        entry_id: String,
    ) -> Result<MutationCart> {
        let resolver = cx.data::<Resolver>()?;
        let mut conn = resolver.pg_conn()?;

        let eid: Id<CartEntry> = entry_id.parse()?;
        let cid: Id<Cart> = cart_id.parse()?;

        conn.deref_mut().transaction(|conn| {
            let _ = t_carts::table
                .find(cid.raw())
                .select(QueryCart::as_select())
                .get_result(conn)?;

            let cart_entry = t_cart_entries::table
                .find(eid.raw())
                .select(QueryCartEntry::as_select())
                .get_result(conn)?;

            if cart_entry.quantity == 1 {
                diesel::delete(t_cart_entries::table)
                    .filter(t_cart_entries::id.eq(eid.raw()))
                    .execute(conn)?;
                return get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() });
            }

            diesel::update(t_cart_entries::table)
                .filter(t_cart_entries::id.eq(eid.raw()))
                .set(t_cart_entries::quantity.eq(cart_entry.quantity - 1))
                .execute(conn)?;
            get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() })
        })
    }
}
