use crate::graphql::model::common::{CurrencyCode, Money};
use crate::graphql::model::product::Product;
use crate::infra::id::Id;
use async_graphql::*;
use bigdecimal::BigDecimal;
use std::ops::Mul;

pub struct Cart {
    pub id: Id<Cart>,
    pub entries: Vec<CartEntry>,
}

#[derive(SimpleObject)]
pub struct CreateCart {
    pub cart: Cart,
}

pub struct CartEntry {
    pub id: Id<CartEntry>,
    pub quantity: i32,
    pub product: Product,
    pub variants_idx: usize, // the selected variant in product, default 0
}

#[Object]
impl Cart {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn entries(&self) -> &[CartEntry] {
        self.entries.as_slice()
    }

    async fn total_amount(&self) -> Money {
        self.entries.iter().map(|v| v.calculate_amount()).sum()
    }
}

impl CartEntry {
    fn calculate_amount(&self) -> Money {
        if self.product.variants.is_empty() {
            return Money {
                amount: BigDecimal::from(0),
                currency_code: CurrencyCode::USD,
            };
        }
        let price = &self.product.variants[self.variants_idx].price;
        let total = price.amount.clone().mul(BigDecimal::from(self.quantity));
        Money {
            amount: total,
            currency_code: price.currency_code,
        }
    }
}

#[Object]
impl CartEntry {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn quantity(&self) -> i32 {
        self.quantity
    }

    async fn product(&self) -> &Product {
        &self.product
    }

    async fn total_amount(&self) -> Money {
        self.calculate_amount()
    }
}
