use crate::domain::product::model::*;
use crate::graphql::Resolver;
use crate::infra::mqsrs::Query;
use crate::infra::resolver::BaseResolver;
use crate::rpc::Resolver as RpcResolver;
use crate::schema::t_products;
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use diesel::{BelongingToDsl, PgConnection, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper};
use std::ops::{DerefMut, Div};
use volo_gen::common::v1::{CurrencyCode, Image, Money};
use volo_gen::product::v1::{GetProductReq, GetProductRes, Product, ProductVariant};

fn execute(req: GetProductReq, conn: &mut PgConnection) -> Result<GetProductRes> {
    let product: QueryProduct = match t_products::table
        .find(req.id)
        .select(QueryProduct::as_select())
        .get_result(conn)
    {
        Ok(product) => product,
        Err(_) => return Ok(GetProductRes { product: None }),
    };
    let currency_code = match &*product.currency_code.to_uppercase() {
        "USD" => CurrencyCode::Usd,
        "CNY" => CurrencyCode::Cny,
        _ => return Err(anyhow!("error parsing currency_code")),
    };
    let images = QueryProductImage::belonging_to(&product)
        .select(QueryProductImage::as_select())
        .load(conn)?;
    let variants = QueryProductVariant::belonging_to(&product)
        .select(QueryProductVariant::as_select())
        .load(conn)?;
    Ok(GetProductRes {
        product: Some(Product {
            id: product.id,
            title: product.title.into(),
            sub_title: product.sub_title.into(),
            description: product.description.into(),
            images: images
                .into_iter()
                .map(|v| Image {
                    url: v.url.into(),
                    alt_text: v.alt_text.into(),
                })
                .collect(),
            variants: variants
                .into_iter()
                .map(|v| ProductVariant {
                    id: v.id,
                    price: Money {
                        amount: (BigDecimal::from(v.price.0).div(100) as BigDecimal)
                            .to_string()
                            .into(),
                        currency_code,
                    },
                    title: v.title.into(),
                    inventory_count: v.inventory_count,
                })
                .collect(),
            currency_code,
        }),
    })
}

impl RpcResolver {
    pub fn create_get_product(&self) -> impl Query<GetProductReq, Result<GetProductRes>> + '_ {
        move |_: GetProductReq| async { todo!() }
    }
}

impl Resolver {
    pub fn create_get_product(&self) -> impl Query<GetProductReq, Result<GetProductRes>> + '_ {
        move |req: GetProductReq| async {
            execute(req, self.resolve(&self.pgsql).get()?.deref_mut())
        }
    }
}
