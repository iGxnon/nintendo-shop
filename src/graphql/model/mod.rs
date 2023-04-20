mod cart;
mod common;
mod product;

use crate::graphql::model::cart::{Cart, CartEntry};
use crate::graphql::model::common::{CurrencyCode, Image, Money};
use crate::graphql::model::product::{Product, ProductVariant};
use crate::graphql::Resolver;
use crate::infra::id::Id;
use crate::infra::mqsrs;
use async_graphql::*;
use bigdecimal::BigDecimal;
use volo_gen::product::v1::GetProductReq;

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    async fn product<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Product>> {
        let id: Id<Product> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let query = resolver.create_get_product();
        let res = mqsrs::Query::execute(&query, GetProductReq { id: id.raw() }).await?;
        if let Some(product) = res.product {
            return Ok(Some(Product {
                id,
                title: product.title.into_string(),
                sub_title: product.sub_title.into_string(),
                description: product.description.into_string(),
                images: product
                    .images
                    .into_iter()
                    .map(|v| Image {
                        url: v.url.parse().unwrap(),
                        alt_text: v.alt_text.into_string(),
                    })
                    .collect(),
                variants: product
                    .variants
                    .into_iter()
                    .map(|v| ProductVariant {
                        id: v.id.into(),
                        price: Money {
                            amount: v.price.amount.parse().unwrap(),
                            currency_code: product.currency_code.into(),
                        },
                        title: v.title.into_string(),
                        inventory_count: v.inventory_count,
                    })
                    .collect(),
            }));
        }
        Ok(None)
    }

    async fn cart(&self, id: String) -> Option<Cart> {
        Some(Cart {
            id: 1.into(),
            entries: vec![CartEntry {
                id: 2.into(),
                quantity: 3,
                product: Product {
                    id: 4.into(),
                    title: "Mr. DZ".to_string(),
                    sub_title: "Yan".to_string(),
                    description: "Smoke".to_string(),
                    images: vec![],
                    variants: vec![ProductVariant {
                        id: 9.into(),
                        price: Money {
                            amount: BigDecimal::from(10),
                            currency_code: CurrencyCode::USD,
                        },
                        title: "xl".to_string(),
                        inventory_count: 0,
                    }],
                },
                variants_idx: 0,
            }],
        })
    }
}
