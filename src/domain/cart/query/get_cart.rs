use crate::domain::cart::model::*;
use crate::domain::product::model::*;
use crate::graphql::Resolver;
use crate::infra::mqsrs::Query;
use crate::infra::resolver::BaseResolver;
use crate::schema::t_cart_entries;
use crate::schema::t_products;
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::{
    BelongingToDsl, ExpressionMethods, GroupedBy, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::ops::Div;
use volo_gen::cart::v1::{Cart, CartEntry, GetCartReq, GetCartRes};
use volo_gen::common::v1::{CurrencyCode, Image};
use volo_gen::product::v1::{Product, ProductVariant};

fn execute(req: GetCartReq, conn: &mut PgConnection) -> Result<GetCartRes> {
    let entries: Vec<QueryCartEntry> = t_cart_entries::table
        .select(QueryCartEntry::as_select())
        .filter(t_cart_entries::cid.eq(req.id))
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
    let mut products = products
        .into_iter()
        .zip(images)
        .zip(variants)
        .map(|v| {
            let code = match &*v.0 .0.currency_code.to_uppercase() {
                "USD" => CurrencyCode::Usd,
                "CNY" => CurrencyCode::Cny,
                _ => panic!("unexpect currency_code"),
            };
            (
                v.0 .0.id,
                Product {
                    id: v.0 .0.id,
                    title: v.0 .0.title.into(),
                    sub_title: v.0 .0.sub_title.into(),
                    description: v.0 .0.description.into(),
                    currency_code: code,
                    images: v
                        .0
                         .1
                        .into_iter()
                        .map(|v| Image {
                            url: v.url.into(),
                            alt_text: v.alt_text.into(),
                        })
                        .collect(),
                    variants: v
                        .1
                        .into_iter()
                        .map(|v| ProductVariant {
                            id: v.id,
                            price: volo_gen::common::v1::Money {
                                amount: (BigDecimal::from(v.price.0).div(100) as BigDecimal)
                                    .to_string()
                                    .into(),
                                currency_code: code,
                            },
                            title: v.title.into(),
                            inventory_count: v.inventory_count,
                        })
                        .collect(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let entries = entries
        .into_iter()
        .map(|v| CartEntry {
            id: v.id,
            quantity: v.quantity,
            product: products.remove(&v.pid).unwrap(), // instead of clone, using remove to take ownership
            variants_idx: v.variant,
        })
        .collect::<Vec<_>>();
    Ok(GetCartRes {
        cart: Some(Cart {
            id: req.id,
            entries,
        }),
    })
}

impl Resolver {
    pub fn create_get_cart(&self) -> impl Query<GetCartReq, Result<GetCartRes>> + '_ {
        move |req: GetCartReq| async move { execute(req, self.resolve(&self.pgsql).get()?.deref_mut()) }
    }
}
