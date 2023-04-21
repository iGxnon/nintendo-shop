mod cart;
mod common;
mod product;

use crate::graphql::model::cart::{Cart, CartEntry};
use crate::graphql::model::common::{Image, Money};
use crate::graphql::model::product::{Product, ProductVariant};
use crate::graphql::Resolver;
use crate::infra::id::Id;
use crate::infra::mqsrs;
use async_graphql::*;
use volo_gen::cart::v1::{CreateCartReq, GetCartReq};
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

    async fn cart<'ctx>(&self, cx: &Context<'ctx>, id: String) -> Result<Option<Cart>> {
        let id: Id<Cart> = id.parse()?;
        let resolver = cx.data::<Resolver>()?;
        let query = resolver.create_get_cart();
        let res = mqsrs::Query::execute(&query, GetCartReq { id: id.raw() }).await?;
        if let Some(cart) = res.cart {
            return Ok(Some(Cart {
                id: cart.id.into(),
                entries: cart
                    .entries
                    .into_iter()
                    .map(|v| CartEntry {
                        id: v.id.into(),
                        quantity: v.quantity,
                        variants_idx: v.variants_idx as usize,
                        product: Product {
                            id: v.product.id.into(),
                            title: v.product.title.into_string(),
                            sub_title: v.product.sub_title.into_string(),
                            description: v.product.description.into_string(),
                            images: v
                                .product
                                .images
                                .into_iter()
                                .map(|i| Image {
                                    url: i.url.parse().unwrap(),
                                    alt_text: i.alt_text.into_string(),
                                })
                                .collect(),
                            variants: v
                                .product
                                .variants
                                .into_iter()
                                .map(|a| ProductVariant {
                                    id: a.id.into(),
                                    price: Money {
                                        amount: a.price.amount.parse().unwrap(),
                                        currency_code: v.product.currency_code.into(),
                                    },
                                    title: a.title.into_string(),
                                    inventory_count: a.inventory_count,
                                })
                                .collect(),
                        },
                    })
                    .collect(),
            }));
        }
        Ok(None)
    }
}

#[Object]
impl Mutation {
    async fn create_cart<'ctx>(&self, cx: &Context<'ctx>) -> Result<Cart> {
        let resolver = cx.data::<Resolver>()?;
        let mutation = resolver.create_cart();
        let res = mqsrs::Mutation::execute(mutation, CreateCartReq {}).await?;
        Ok(Cart {
            id: res.cart.id.into(),
            entries: vec![],
        })
    }

    async fn add_item_to_cart<'ctx>(&self, cx: &Context<'ctx>) -> Result<bool> {
        todo!()
    }

    async fn get_or_init_checkout<'ctx>(&self, cx: &Context<'ctx>) -> Result<bool> {
        todo!()
    }

    async fn checkout<'ctx>(&self, cx: &Context<'ctx>) -> Result<bool> {
        todo!()
    }
}
