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
use async_graphql::*;
use bigdecimal::BigDecimal;
use diesel::data_types::PgTime;
use diesel::prelude::*;
use diesel::sql_types::{Time, Timestamp};
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
    let products = products
        .into_iter()
        .zip(images)
        .zip(variants)
        .map(|((product, images), variants)| {
            let code = match &*product.currency_code.to_uppercase() {
                "USD" => CurrencyCode::USD,
                "CNY" => CurrencyCode::CNY,
                _ => panic!("unexpect currency_code"),
            };
            (
                product.id,
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
                },
            )
        })
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

            let (eid, quantity) = t_cart_entries::table
                .filter(
                    t_cart_entries::cid
                        .eq(cart.id)
                        .and(t_cart_entries::pid.eq(variant.pid))
                        .and(t_cart_entries::variant.eq(variant.order_idx)),
                )
                .select((t_cart_entries::id, t_cart_entries::quantity))
                .get_result::<(i64, i32)>(conn)?;

            if quantity == 1 {
                diesel::delete(t_cart_entries::table)
                    .filter(t_cart_entries::id.eq(eid))
                    .execute(conn)?;
                return get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() });
            }

            diesel::update(t_cart_entries::table)
                .filter(t_cart_entries::id.eq(eid))
                .set(t_cart_entries::quantity.eq(quantity - 1))
                .execute(conn)?;
            get_cart(cid, conn).map(|v| MutationCart { cart: v.unwrap() })
        })
    }
}
