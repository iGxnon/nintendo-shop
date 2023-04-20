mod cart;
mod common;
mod product;

use crate::graphql::model::cart::{Cart, CartEntry};
use crate::graphql::model::common::{CurrencyCode, Money};
use crate::graphql::model::product::{Product, ProductVariant};
use async_graphql::*;
use bigdecimal::BigDecimal;

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    async fn product(&self, id: String) -> Option<Product> {
        Some(Product {
            id: 1.into(),
            title: "Mr. DZ".to_string(),
            sub_title: "Electron Smoke".to_string(),
            description: "Smoke everyday".to_string(),
            images: vec![],
            variants: vec![
                ProductVariant {
                    id: 2.into(),
                    price: Money {
                        amount: BigDecimal::from(8),
                        currency_code: CurrencyCode::USD,
                    },
                    title: "l".to_string(),
                    inventory_count: 3,
                },
                ProductVariant {
                    id: 9.into(),
                    price: Money {
                        amount: BigDecimal::from(10),
                        currency_code: CurrencyCode::USD,
                    },
                    title: "xl".to_string(),
                    inventory_count: 3,
                },
                ProductVariant {
                    id: 11.into(),
                    price: Money {
                        amount: BigDecimal::from(12),
                        currency_code: CurrencyCode::USD,
                    },
                    title: "xxl".to_string(),
                    inventory_count: 3,
                },
            ],
        })
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
